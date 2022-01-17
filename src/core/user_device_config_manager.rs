use super::{random_string, util};
use buttplug::{
  core::messages::DeviceMessageAttributesMap,
  core::messages::{ButtplugDeviceMessageType, DeviceMessageAttributes},
  device::configuration_manager::{ProtocolAttributes, ProtocolDefinition, WebsocketSpecifier},
  server::device_manager::DeviceUserConfig,
  util::device_configuration::{load_protocol_config_from_json, ProtocolConfiguration},
};
use std::collections::HashMap;

const BUTTPLUG_PASSTHRU: &str = "buttplug-passthru";

pub struct UserDeviceConfigManager {
  config: ProtocolConfiguration,
}

impl Default for UserDeviceConfigManager {
  fn default() -> Self {
    let config = if util::user_device_config_file_path().exists() {
      // TODO This could fail if the file is invalid.
      let user_config_file = std::fs::read_to_string(util::user_device_config_file_path()).unwrap();
      load_protocol_config_from_json(&user_config_file, true).unwrap()
    } else {
      ProtocolConfiguration::default()
    };
    Self { config }
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
    let device_record = self
      .config
      .user_config
      .entry(address.to_owned())
      .or_default();
    device_record.set_allow(Some(true));
  }

  pub fn remove_allowed_device(&mut self, address: &str) {
    let device_record = self
      .config
      .user_config
      .entry(address.to_owned())
      .or_default();
    device_record.set_allow(None);
    self.cleanup();
  }

  pub fn add_denied_device(&mut self, address: &str) {
    let device_record = self
      .config
      .user_config
      .entry(address.to_owned())
      .or_default();
    device_record.set_deny(Some(true));
  }

  pub fn remove_denied_device(&mut self, address: &str) {
    let device_record = self
      .config
      .user_config
      .entry(address.to_owned())
      .or_default();
    device_record.set_deny(None);
    self.cleanup();
  }

  pub fn add_simulated_device(
    &mut self,
    name: &str,
    num_vibrators: u32,
    num_rotators: u32,
    num_linear: u32,
  ) {
    // We'll need to add 2 things to the "buttplug-passthru" protocol:
    // - A name to the websocket identifier list
    // - A protocol attributes block to the configurations list, with the message attributes info

    let identifier = format!("simulator-{}", random_string());

    let protocol_def = if let Some(def) = self.config.protocols.get_mut(BUTTPLUG_PASSTHRU) {
      def
    } else {
      self
        .config
        .protocols
        .insert(BUTTPLUG_PASSTHRU.to_owned(), ProtocolDefinition::default());
      self
        .config
        .protocols
        .get_mut(BUTTPLUG_PASSTHRU)
        .expect("We just added it")
    };

    let websocket_definition = if let Some(def) = protocol_def.websocket_mut() {
      def
    } else {
      protocol_def.set_websocket(Some(WebsocketSpecifier::default()));
      protocol_def
        .websocket_mut()
        .as_mut()
        .expect("We just added it")
    };

    websocket_definition.names.insert(identifier.clone());

    let mut name_map = HashMap::new();
    name_map.insert("en-us".to_owned(), name.to_owned());

    let mut message_map = DeviceMessageAttributesMap::new();
    if num_vibrators > 0 {
      message_map.insert(
        ButtplugDeviceMessageType::VibrateCmd,
        DeviceMessageAttributes {
          feature_count: Some(num_vibrators),
          step_count: Some(vec![
            100u32;
            num_vibrators
              .try_into()
              .expect("Should be a normal size.")
          ]),
          ..Default::default()
        },
      );
    }
    if num_rotators > 0 {
      message_map.insert(
        ButtplugDeviceMessageType::RotateCmd,
        DeviceMessageAttributes {
          feature_count: Some(num_rotators),
          step_count: Some(vec![
            100u32;
            num_rotators
              .try_into()
              .expect("Should be a normal size.")
          ]),
          ..Default::default()
        },
      );
    }
    if num_linear > 0 {
      message_map.insert(
        ButtplugDeviceMessageType::LinearCmd,
        DeviceMessageAttributes {
          feature_count: Some(num_linear),
          step_count: Some(vec![
            100u32;
            num_linear.try_into().expect("Should be a normal size.")
          ]),
          ..Default::default()
        },
      );
    }

    let mut device_config = ProtocolAttributes::default();
    device_config.set_identifier(Some(vec![identifier.clone()]));
    device_config.set_name(Some(name_map));
    device_config.set_messages(Some(message_map));

    protocol_def.configurations_mut().push(device_config);

    self.save_user_config();
  }

  pub fn remove_simulated_device(&mut self, attributes: &ProtocolAttributes) {
    // First off, look up the buttplug-passthru protocol
    let protocol_def = if let Some(def) = self.config.protocols.get_mut(BUTTPLUG_PASSTHRU) {
      def
    } else {
      return;
    };

    for identifier in attributes.identifier().as_ref().unwrap_or(&vec!()).iter() {
      // Remove all instances from names
      if let Some(websocket_def) = protocol_def.websocket_mut() {
        websocket_def.names.remove(identifier);
      }
      // Remove identifier matched instances from configurations. Assume we aren't batching
      // identifiers for simulator configs.
      *protocol_def.configurations_mut() = protocol_def
        .configurations()
        .iter()
        .filter(|config| {
          !config
            .identifier()
            .as_ref()
            .unwrap_or(&vec![])
            .contains(&identifier.to_owned())
        })
        .cloned()
        .collect();
    }

    self.save_user_config();
  }

  pub fn get_simulated_devices(&self) -> Vec<ProtocolAttributes> {
    // First off, look up the buttplug-passthru protocol
    let protocol_def = if let Some(def) = self.config.protocols.get(BUTTPLUG_PASSTHRU) {
      def
    } else {
      return vec![];
    };

    // In the configurations field, find all identifiers that start with "simulator-"
    protocol_def
      .configurations()
      .iter()
      .filter(|x| {
        x.identifier()
          .as_ref()
          .unwrap_or(&vec![])
          .iter()
          .any(|ident| ident.contains("simulator-"))
      })
      .cloned()
      .collect()
  }

  pub fn get_user_config(&self) -> &HashMap<String, DeviceUserConfig> {
    &self.config.user_config
  }

  pub fn save_user_config(&self) {
    let config_json = self.config.to_json();
    std::fs::write(util::user_device_config_file_path(), config_json).unwrap();
  }
}
