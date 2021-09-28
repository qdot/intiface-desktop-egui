use tokio::process::Command;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::{IntifaceConfiguration, util};
use thiserror::Error;

use tracing::info;


#[derive(Default)]
pub struct ProcessManager {
  process_running: Arc<AtomicBool>
}

#[derive(Error, Debug)]
pub enum ProcessError {
  #[error("Could not start process: {0}")]
  ProcessStartupError(String)
}

impl ProcessManager {

  fn build_arguments<'a>(&self, config: &IntifaceConfiguration) -> Vec<String> {
    let mut args = vec!();
    args.push("--servername".to_owned());
    args.push(config.server_name().clone());

    if util::device_config_file_path().exists() {
      args.push("--deviceconfig".to_owned());
      args.push(util::device_config_file_path().to_str().unwrap().to_owned());
    }
    if util::user_device_config_file_path().exists() {
      args.push("--userdeviceconfig".to_owned());
      args.push(util::user_device_config_file_path().to_str().unwrap().to_owned());
    }
    
    // First, we set up our incoming pipe to receive GUI info from the CLI
    // process
    // TODO Implement frontend pipe.
    // args.push(`--frontendpipe`);
    args.push("--stayopen".to_owned());
    if config.use_websocket_server_insecure() {
      if config.websocket_server_all_interfaces() {
        args.push("--wsallinterfaces".to_owned());
      }
      args.push("--wsinsecureport".to_owned());
      args.push(config.websocket_server_insecure_port().to_string());
    }
    // TODO Reimplement log level output.
    /*
    if (this._config.ServerLogLevel !== "Off") {
      args.push(`--log`, `${this._config.ServerLogLevel}`);
    }
     */
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
    
    match Command::new(command_path).args(args).kill_on_drop(true).spawn() {
      Ok(mut child) => {
        let process_running = self.process_running.clone();
        process_running.store(true, Ordering::SeqCst);
        tokio::spawn(async move {
          match child.wait().await {
            Ok(status) => {
              process_running.store(false, Ordering::SeqCst);
            }
            Err(e) => {

            }
          }
        });
        Ok(())
      }
      Err(err) => {
        Err(ProcessError::ProcessStartupError(err.to_string()))
      }
    }
  }

  pub fn is_running(&self) -> bool {
    self.process_running.load(Ordering::SeqCst)
  }
}