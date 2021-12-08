use super::{process_messages::*, util, IntifaceConfiguration};
use dashmap::{DashMap, DashSet};
use notify_rust::Notification;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
  io,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};
use thiserror::Error;
#[cfg(not(target_os = "windows"))]
use tokio::net::unix::{UnixListener, UnixStream};
#[cfg(target_os = "windows")]
use tokio::net::windows::named_pipe;
use tokio::{io::Interest, process::Command, select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct ButtplugServerDevice {
  pub name: String,
  pub display_name: Option<String>,
  pub address: String,
}

impl ButtplugServerDevice {
  pub fn new(name: &str, display_name: Option<String>, address: &str) -> Self {
    Self {
      name: name.to_owned(),
      display_name,
      address: address.to_owned(),
    }
  }
}

pub struct ProcessManager {
  process_running: Arc<AtomicBool>,
  process_stop_sender: Option<mpsc::Sender<()>>,
  client_name: Arc<DashSet<String>>,
  client_devices: Arc<DashMap<u32, ButtplugServerDevice>>,
}

impl Default for ProcessManager {
  fn default() -> Self {
    Self {
      process_running: Arc::new(AtomicBool::new(false)),
      process_stop_sender: None,
      client_name: Arc::new(DashSet::new()),
      client_devices: Arc::new(DashMap::new()),
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
        messages.push(msg);
      }
      Err(e) => {
        //error!("{:?}", e);
        break;
      }
    }
  }
  *data = data[stream.byte_offset()..].to_vec();
  messages
}

async fn run_windows_named_pipe(
  pipe_name: &str,
  mut stop_receiver: mpsc::Receiver<()>,
  process_ended_token: CancellationToken,
  client_name: Arc<DashSet<String>>,
  client_devices: Arc<DashMap<u32, ButtplugServerDevice>>,
) {
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
                      EngineMessage::EngineLog(msg) => {
                        let log_msg: EngineLogMessage = serde_json::from_str(msg).unwrap();
                        log_msg.log_event();
                      }
                      EngineMessage::ClientConnected(name) => {
                        Notification::new()
                            .summary("Intiface Client Connected")
                            .body(&format!("Client {} connected to Intiface Desktop.", name))
                            .show();
                        client_name.clear();
                        client_name.insert(name.clone());
                      }
                      EngineMessage::ClientDisconnected => {
                        Notification::new()
                            .summary("Intiface Client Disconnected")
                            .body(&format!("Client disconnected from Intiface Desktop."))
                            .show();
                        client_name.clear();
                      }
                      EngineMessage::DeviceConnected { name, index, address, display_name } => {
                        let display_name = if display_name.is_empty() {
                          None
                        } else {
                          Some(display_name.clone())
                        };
                        Notification::new()
                            .summary("Intiface Device Connected")
                            .body(&format!("Device {} ({:?}) connected to Intiface Desktop.", name, display_name))
                            .show();
                        client_devices.insert(*index, ButtplugServerDevice::new(&name, display_name, &address));
                      }
                      EngineMessage::DeviceDisconnected(index) => {
                        client_devices.remove(index);
                      }
                      _ => {}
                    }
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

    if config.crash_reporting() {
      args.push("--crash-reporting".to_owned());
    }

    args
  }

  pub fn run(&mut self, config: &IntifaceConfiguration) -> Result<(), ProcessError> {
    let rand_string: String = thread_rng()
      .sample_iter(&Alphanumeric)
      .take(15)
      .map(char::from)
      .collect();
    #[cfg(target_os = "windows")]
    let pipe_name = format!("\\\\.\\pipe\\{}", rand_string);
    #[cfg(not(target_os = "windows"))]
    unimplemented!("Implement domain socket name generation!");

    let command_path = util::engine_file_path();
    let args = self.build_arguments(&pipe_name, config);
    info!("{:?} {}", command_path, args.join(" "));
    let process_ended_token = CancellationToken::new();
    let process_ended_token_child = process_ended_token.child_token();
    let (tx, rx) = mpsc::channel(1);
    self.process_stop_sender = Some(tx);
    let client_name = self.client_name.clone();
    let client_devices = self.client_devices.clone();
    tokio::spawn(async move {
      run_windows_named_pipe(
        &pipe_name,
        rx,
        process_ended_token_child,
        client_name,
        client_devices,
      )
      .await;
    });

    #[cfg(not(target_os = "windows"))]
    let command_result = Command::new(command_path)
      .args(args)
      .kill_on_drop(true)
      .spawn();
    #[cfg(target_os = "windows")]
    let command_result = Command::new(command_path)
      .args(args)
      .creation_flags(0x00000008)
      .kill_on_drop(true)
      .spawn();

    match command_result {
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

  pub fn client_devices(&self) -> Vec<ButtplugServerDevice> {
    self
      .client_devices
      .iter()
      .map(|val| val.value().clone())
      .collect()
  }
}
