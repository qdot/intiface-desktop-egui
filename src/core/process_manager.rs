use super::{process_messages::*, util, IntifaceConfiguration};
use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  io
};
use thiserror::Error;
#[cfg(not(target_os = "windows"))]
use tokio::net::unix::{UnixListener, UnixStream};
#[cfg(target_os = "windows")]
use tokio::net::windows::named_pipe;
use tokio::{select, process::Command, sync::{broadcast, mpsc}, io::Interest};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use dashmap::{DashMap, DashSet};

pub struct ProcessManager {
  process_running: Arc<AtomicBool>,
  process_events: Arc<broadcast::Sender<EngineMessage>>,
  process_stop_sender: Option<mpsc::Sender<()>>,
  client_name: Arc<DashSet<String>>,
  client_devices: Arc<DashMap<u32, String>>
}

impl Default for ProcessManager {
  fn default() -> Self {
    let (process_events, _) = broadcast::channel(256);
    Self {
      process_running: Arc::new(AtomicBool::new(false)),
      process_events: Arc::new(process_events),
      process_stop_sender: None,
      client_name: Arc::new(DashSet::new()),
      client_devices: Arc::new(DashMap::new())
    }
  }
}

#[derive(Error, Debug)]
pub enum ProcessError {
  #[error("Could not start process: {0}")]
  ProcessStartupError(String),
}

fn translate_buffer(data: &mut Vec<u8>) -> Vec<EngineMessage> {
  let de = serde_json::Deserializer::from_slice(data);
  let mut stream = de.into_iter::<EngineMessage>();
  let mut messages = vec![];
  while let Some(msg) = stream.next() {
    match msg {
      Ok(msg) => {
        info!("{:?}", msg);
        messages.push(msg);
      },
      Err(e) => {
        //error!("{:?}", e);
        break;
      }
    }
  }
  *data = data[stream.byte_offset()..].to_vec();
  messages
}

async fn run_windows_named_pipe(pipe_name: &str, sender: Arc<broadcast::Sender<EngineMessage>>, mut stop_receiver: mpsc::Receiver<()>, process_ended_token: CancellationToken, client_name: Arc<DashSet<String>>, client_devices: Arc<DashMap<u32, String>>) {
  info!("Starting named pipe server at {}", pipe_name);
  let server = named_pipe::ServerOptions::new()
    .first_pipe_instance(true)
    .create(pipe_name)
    .unwrap();
  server.connect().await.unwrap();
  let mut stopped = false;
  loop {
    select! {
      ready = server.ready(Interest::READABLE) => {
        match ready {
          Ok(status) => {
            if status.is_readable() {
              let mut data = vec![0; 1024];
              match server.try_read(&mut data) {
                Ok(n) => {
                  let msgs = translate_buffer(&mut data);
                  for msg in msgs {
                    match &msg {
                      EngineMessage::ClientConnected(name) => {
                        client_name.clear();
                        client_name.insert(name.clone());
                      }
                      EngineMessage::ClientDisconnected => {
                        client_name.clear();
                      }
                      EngineMessage::DeviceConnected(name, index) => {
                        client_devices.insert(*index, name.clone());
                      }
                      EngineMessage::DeviceDisconnected(index) => {
                        client_devices.remove(index);
                      }
                      _ => {}
                    }
                    let _ = sender.send(msg);
                  }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                  continue;
                }
                Err(e) => {
                  //return Err(e.into());
                }
              }              
            }
          },
          Err(e) => {
            warn!("Error receiving info from named pipe, closing and returning.");
            break;
          }
        }
      },
      _ = stop_receiver.recv() => {
        info!("Waiting to write to channel");
        match server.ready(Interest::WRITABLE).await {
          Ok(status) => {
            if status.is_writable() {
              info!("Writing to channel and breaking.");
              server.try_write(&serde_json::to_vec(&IntifaceMessage::Stop).unwrap());
              stopped = true;
            }
          },
          Err(e) => {
            warn!("Error receiving info from named pipe, closing and returning.");
          }
        };
      },
      _ = process_ended_token.cancelled() => {
        if !stopped {
          error!("Process ended without sending Stop message.");
        } else {
          info!("Process ended cleanly, exiting loop.");
        }
        break;
      }
    }
  }
  server.disconnect();
  client_name.clear();
  client_devices.clear();
  info!("Pipe {} disconnected.", pipe_name);
}

impl ProcessManager {
  fn build_arguments<'a>(&self, pipe_name: &str, config: &IntifaceConfiguration) -> Vec<String> {
    let mut args = vec![];
    args.push("--servername".to_owned());
    args.push(config.server_name().clone());
    args.push("--stayopen".to_owned());
    args.push("--frontendpipe".to_owned());
    args.push(pipe_name.to_owned());

    if util::device_config_file_path().exists() {
      args.push("--deviceconfig".to_owned());
      args.push(util::device_config_file_path().to_str().unwrap().to_owned());
    }
    if util::user_device_config_file_path().exists() {
      args.push("--userdeviceconfig".to_owned());
      args.push(
        util::user_device_config_file_path()
          .to_str()
          .unwrap()
          .to_owned(),
      );
    }

    if config.use_websocket_server_insecure() {
      if config.websocket_server_all_interfaces() {
        args.push("--wsallinterfaces".to_owned());
      }
      args.push("--wsinsecureport".to_owned());
      args.push(config.websocket_server_insecure_port().to_string());
    }

    args.push("--log".to_owned());
    args.push(config.server_log_level().to_string());

    if config.server_max_ping_time() > 0 {
      args.push("--pingtime".to_owned());
      args.push(config.server_max_ping_time().to_string());
    }

    // Opt-out services

    if !config.with_bluetooth_le() {
      args.push("--without-bluetooth-le".to_owned());
    }
    if !config.with_hid() {
      args.push("--without-hid".to_owned());
    }
    if !config.with_lovense_hid_dongle() {
      args.push("--without-lovense-dongle-hid".to_owned());
    }
    if !config.with_lovense_serial_dongle() {
      args.push("--without-lovense-dongle-serial".to_owned());
    }
    if !config.with_serial_port() {
      args.push("--without-serial".to_owned());
    }
    if !config.with_xinput() {
      args.push("--without-xinput".to_owned());
    }

    // Opt-in services
    if config.with_lovense_connect_service() {
      args.push("--with-lovense-connect".to_owned());
    }

    args
  }

  pub fn run(&mut self, config: &IntifaceConfiguration) -> Result<(), ProcessError> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();
    #[cfg(target_os="windows")]
    let pipe_name = format!("\\\\.\\pipe\\{}", rand_string);
    #[cfg(not(target_os="windows"))]
    unimplemented!("Implement domain socket name generation!");

    let command_path = util::engine_file_path();
    let args = self.build_arguments(&pipe_name, config);
    info!("{:?}", command_path);
    info!("{:?}", args);
    let process_ended_token = CancellationToken::new();
    let process_ended_token_child = process_ended_token.child_token();
    let (tx, rx) = mpsc::channel(1);
    self.process_stop_sender = Some(tx);
    let sender = self.process_events.clone();
    let client_name = self.client_name.clone();
    let client_devices = self.client_devices.clone();
    tokio::spawn(async move {
      run_windows_named_pipe(&pipe_name, sender, rx, process_ended_token_child, client_name, client_devices).await;
    });

    match Command::new(command_path)
      .args(args)
      .kill_on_drop(true)
      .spawn()
    {
      Ok(mut child) => {
        let process_running = self.process_running.clone();
        process_running.store(true, Ordering::SeqCst);

        tokio::spawn(async move {
          match child.wait().await {
            Ok(status) => {
              info!("Child process ended successfully.");
            }
            Err(e) => {
              error!("Child process ended with error.");
            }
          }
          process_running.store(false, Ordering::SeqCst);
          process_ended_token.cancel();
        });
        Ok(())
      }
      Err(err) => Err(ProcessError::ProcessStartupError(err.to_string())),
    }
  }

  pub fn is_running(&self) -> bool {
    self.process_running.load(Ordering::SeqCst)
  }

  pub fn stop(&mut self) {
    if let Some(sender) = &self.process_stop_sender {
      if sender.try_send(()).is_err() {
        error!("Tried stopping process multiple times, or process is no longer up.");
      }
    }
  }

  pub fn events(&self) -> broadcast::Receiver<EngineMessage> {
    self.process_events.subscribe()
  }

  pub fn client_name(&self) -> Option<String> {
    if self.client_name.is_empty() {
      None
    } else {
      if let Some(name) = self.client_name.iter().next() {
        Some(name.clone())
      } else {
        None
      }
    }
  }

  pub fn client_devices(&self) -> Vec<String> {
    self.client_devices.iter().map(|val| val.value().clone()).collect()
  }
}
