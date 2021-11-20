mod intiface_configuration;
mod process_manager;
mod update_manager;
mod util;
mod file_storage;
mod process_messages;
mod user_device_config_manager;

pub use intiface_configuration::IntifaceConfiguration;
pub use process_manager::ProcessManager;
pub use update_manager::*;
pub use util::*;
pub use file_storage::*;

use self::user_device_config_manager::UserDeviceConfigManager;

#[derive(Default)]
pub struct AppCore {
  pub config: IntifaceConfiguration,
  pub process_manager: ProcessManager,
  pub update_manager: UpdateManager,
  pub user_device_config_manager: UserDeviceConfigManager
}