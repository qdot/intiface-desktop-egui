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
use tokio::{select, process::Command, sync::broadcast, io::Interest};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error};

const INTIFACE_PIPE_NAME: &str = "intiface";

pub struct ProcessManager {
  process_running: Arc<AtomicBool>,
  process_events: Arc<broadcast::Sender<EngineMessage>>,
  process_stop_token: Option<CancellationToken>
}

impl Default for ProcessManager {
  fn default() -> Self {
    let (process_events, _) = broadcast::channel(256);
    Self {
      process_running: Arc::new(AtomicBool::new(false)),
      process_events: Arc::new(process_events),
      process_stop_token: None
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
        error!("{:?}", e);
        break;
      }
    }
  }
  *data = data[stream.byte_offset()..].to_vec();
  messages
}

async fn run_windows_named_pipe(sender: Arc<broadcast::Sender<EngineMessage>>, stop_token: CancellationToken) {
  info!("Starting named pipe server");
  let server = named_pipe::ServerOptions::new()
    .first_pipe_instance(true)
    .create(format!("\\\\.\\pipe\\{}", INTIFACE_PIPE_NAME))
    .unwrap();
  server.connect().await.unwrap();
  loop {
    select! {
      ready = server.ready(Interest::READABLE) => {
        match ready {
          Ok(status) => {
            if status.is_readable() {
              let mut data = vec![0; 1024];
              match server.try_read(&mut data) {
                Ok(n) => {
                  translate_buffer(&mut data);
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
      _ = stop_token.cancelled() => {
        match server.ready(Interest::WRITABLE).await {
          Ok(status) => {
            if status.is_writable() {
              server.try_write(&serde_json::to_vec(&IntifaceMessage::Stop).unwrap());
            }
          },
          Err(e) => {
            warn!("Error receiving info from named pipe, closing and returning.");
          }
        };
        break;
      }
    }
  }
}

impl ProcessManager {
  fn build_arguments<'a>(&self, config: &IntifaceConfiguration) -> Vec<String> {
    let mut args = vec![];
    args.push("--servername".to_owned());
    args.push(config.server_name().clone());
    args.push("--stayopen".to_owned());
    args.push("--frontendpipe".to_owned());
    args.push(INTIFACE_PIPE_NAME.to_owned());

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
    let command_path = util::engine_file_path();
    let args = self.build_arguments(config);
    info!("{:?}", command_path);
    info!("{:?}", args);
    let token = CancellationToken::new();
    let child_token = token.child_token();
    self.process_stop_token = Some(token);
    let sender = self.process_events.clone();
    tokio::spawn(async move {
      run_windows_named_pipe(sender, child_token).await;
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
              process_running.store(false, Ordering::SeqCst);
            }
            Err(e) => {}
          }
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
    if self.process_stop_token.is_some() {
      let token = self.process_stop_token.take().unwrap();
      token.cancel();
    }
  }

  pub fn events(&self) -> broadcast::Receiver<EngineMessage> {
    self.process_events.subscribe()
  }
}
