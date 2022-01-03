mod file_storage;
mod intiface_configuration;
mod modal_manager;
mod process_manager;
mod process_messages;
mod update_manager;
mod user_device_config_manager;
mod util;

pub use file_storage::*;
pub use intiface_configuration::IntifaceConfiguration;
pub use modal_manager::*;
pub use process_manager::ProcessManager;
pub use update_manager::*;
pub use user_device_config_manager::UserDeviceConfigManager;
pub use util::*;

#[derive(Default)]
pub struct AppCore {
  pub config: IntifaceConfiguration,
  pub process_manager: ProcessManager,
  pub update_manager: UpdateManager,
  pub user_device_config_manager: UserDeviceConfigManager,
  pub modal_manager: ModalManager,
}

impl AppCore {
  pub fn new() -> Self {
    Self {
      update_manager: UpdateManager::new(),
      .. Default::default()
    }
  }
}