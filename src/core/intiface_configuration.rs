use getset::{Getters, Setters, MutGetters};
use serde::{Deserialize, Serialize};

#[derive(Getters, Setters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct IntifaceConfiguration {
  server_name: String,
  server_max_ping_time: u32,
  use_websocket_server_insecure: bool,
  websocket_server_all_interfaces: bool,
  websocket_server_insecure_port: u16,
  server_log_level: String,
  use_prerelease_engine: bool,
  current_engine_version: String,
  current_device_file_version: u32,
  check_for_updates_on_start: bool,
  has_run_setup: bool,
  device_file_update_available: bool,
  engine_update_available: bool,
  application_update_available: bool,
  has_usable_engine_executable: bool,
  start_server_on_startup: bool,
  with_bluetooth_le: bool,
  with_serial_port: bool,
  with_hid: bool,
  with_lovense_hid_dongle: bool,
  with_lovense_serial_dongle: bool,
  with_lovense_connect_service: bool,
  with_xinput: bool,
}

impl Default for IntifaceConfiguration {
  fn default() -> Self {
    Self {
      server_name: "Intiface Desktop Server".to_owned(),
      server_max_ping_time: 0,
      use_websocket_server_insecure: true,
      websocket_server_all_interfaces: false,
      websocket_server_insecure_port: 12345,
      server_log_level: "info".to_owned(),
      use_prerelease_engine: false,
      current_engine_version: "0".to_owned(),
      current_device_file_version: 0,
      check_for_updates_on_start: true,
      has_run_setup: false,
      device_file_update_available: false,
      engine_update_available: false,
      application_update_available: false,
      has_usable_engine_executable: false,
      start_server_on_startup: false,
      with_bluetooth_le: true,
      with_serial_port: true,
      with_hid: true,
      with_lovense_hid_dongle: true,
      with_lovense_serial_dongle: true,
      with_lovense_connect_service: true,
      with_xinput: true,
    }
  }
}

impl IntifaceConfiguration {
  pub fn load_from_string(json_config: &str) -> Result<IntifaceConfiguration, serde_json::Error> {
    serde_json::from_str::<IntifaceConfiguration>(json_config)
  }
}
