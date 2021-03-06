use std::str::FromStr;

use getset::{CopyGetters, Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

fn default_tracing_level() -> String {
  tracing::Level::INFO.to_string()
}

#[derive(Setters, MutGetters, Getters, CopyGetters, Serialize, Deserialize, PartialEq, Clone)]
#[getset(get_mut = "pub", set = "pub")]
pub struct IntifaceConfiguration {
  #[getset(get = "pub")]
  #[serde(default)]
  server_name: String,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  server_max_ping_time: u32,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  use_websocket_server_insecure: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  websocket_server_all_interfaces: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  websocket_server_insecure_port: u16,
  #[serde(default = "default_tracing_level")]
  server_log_level: String,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  use_prerelease_engine: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  check_for_updates_on_start: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  has_run_setup: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  has_usable_engine_executable: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  start_server_on_startup: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_bluetooth_le: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_serial_port: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_hid: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_lovense_hid_dongle: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_lovense_serial_dongle: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_lovense_connect_service: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_xinput: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  with_device_websocket_server: bool,  
  #[getset(get_copy = "pub")]
  #[serde(default)]
  crash_reporting: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  show_notifications: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  has_run_first_use: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  show_extended_ui: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  allow_raw_messages: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  has_error_message: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  force_open_log: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  force_open_updates: bool,
  #[getset(get_copy = "pub")]
  #[serde(skip)]
  force_open_news: bool,
  #[getset(get_copy = "pub")]
  #[serde(default)]
  unread_news: bool,
}

impl Default for IntifaceConfiguration {
  fn default() -> Self {
    Self {
      server_name: "Intiface Desktop Server".to_owned(),
      server_max_ping_time: 0,
      use_websocket_server_insecure: true,
      websocket_server_all_interfaces: false,
      websocket_server_insecure_port: 12345,
      server_log_level: tracing::Level::INFO.to_string(),
      use_prerelease_engine: false,
      check_for_updates_on_start: true,
      has_run_setup: false,
      has_usable_engine_executable: false,
      start_server_on_startup: false,
      with_bluetooth_le: true,
      with_serial_port: true,
      with_hid: true,
      with_lovense_hid_dongle: true,
      with_lovense_serial_dongle: true,
      with_lovense_connect_service: false,
      with_xinput: true,
      with_device_websocket_server: false,
      crash_reporting: true,
      show_notifications: true,
      has_run_first_use: false,
      show_extended_ui: false,
      allow_raw_messages: false,
      has_error_message: false,
      force_open_log: false,
      force_open_updates: false,
      force_open_news: false,
      unread_news: false,
    }
  }
}

impl IntifaceConfiguration {
  pub fn server_log_level(&self) -> tracing::Level {
    tracing::Level::from_str(&self.server_log_level).unwrap()
  }

  pub fn load_from_string(json_config: &str) -> Result<IntifaceConfiguration, serde_json::Error> {
    serde_json::from_str::<IntifaceConfiguration>(json_config)
  }
}
