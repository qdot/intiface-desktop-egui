use std::collections::HashMap;
use buttplug::{server::device_manager::DeviceUserConfig, util::device_configuration::{load_protocol_config_from_json, ProtocolConfiguration}};
use super::util;

pub struct UserDeviceConfigManager {
  config: ProtocolConfiguration
}

impl Default for UserDeviceConfigManager {
  fn default() -> Self {
    let config = if util::user_device_config_file_path().exists() {
      // TODO This could fail if the file is invalid.
      let user_config_file = std::fs::read_to_string(util::user_device_config_file_path()).unwrap();
      load_protocol_config_from_json(&user_config_file).unwrap()
    } else {
      ProtocolConfiguration::default()
    };
    Self {
      config
    }
  }
}

impl UserDeviceConfigManager {
  fn cleanup(&mut self) {
    let default_config = DeviceUserConfig::default();
    for (address, config) in self.config.user_config.clone() {
      if config == default_config {
        let _ = self.config.user_config.remove(&address);
      }
    }
  }

  pub fn add_allowed_device(&mut self, address: &str) {
    let device_record = self.config.user_config.entry(address.to_owned()).or_default();
    device_record.set_allow(Some(true));
  }

  pub fn remove_allowed_device(&mut self, address: &str) {
    let device_record = self.config.user_config.entry(address.to_owned()).or_default();
    device_record.set_allow(None);
    self.cleanup();
  }

  pub fn add_denied_device(&mut self, address: &str) {
    let device_record = self.config.user_config.entry(address.to_owned()).or_default();
    device_record.set_deny(Some(true));
  }

  pub fn remove_denied_device(&mut self, address: &str) {
    let device_record = self.config.user_config.entry(address.to_owned()).or_default();
    device_record.set_deny(None);
    self.cleanup();
  }

  pub fn get_user_config(&self) -> &HashMap::<String, DeviceUserConfig> {
    &self.config.user_config
  }

  pub fn save_user_config(&self) {
    let config_json = self.config.to_json();
    std::fs::write(util::user_device_config_file_path(), config_json).unwrap();
  }
}