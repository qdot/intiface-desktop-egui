use std::path::PathBuf;

pub const USER_DEVICE_CONFIG_FILENAME: &str = "buttplug-user-device-config.json";
pub const DEVICE_CONFIG_FILENAME: &str = "buttplug-device-config.json";
pub const INTIFACE_CONFIG_FILENAME: &str = "intiface.config.json";
pub const INTIFACE_CONFIG_DIRECTORY_NAME: &str = "IntifaceDesktop";

#[cfg(target_os = "windows")]
const EXECUTABLE_NAME: &str = "IntifaceCLI.exe";
#[cfg(not(target_os = "windows"))]
const EXECUTABLE_NAME: &str = "IntifaceCLI";

pub fn user_config_directory() -> PathBuf {
  let mut home = dirs::data_local_dir().unwrap();
  home.push(INTIFACE_CONFIG_DIRECTORY_NAME);
  home
}

pub fn config_file_path() -> PathBuf {
  let mut dir = user_config_directory();
  dir.push(INTIFACE_CONFIG_FILENAME);
  dir
}

pub fn device_config_file_path() -> PathBuf {
  let mut dir = user_config_directory();
  dir.push(DEVICE_CONFIG_FILENAME);
  dir
}

pub fn user_device_config_file_path() -> PathBuf {
  let mut dir = user_config_directory();
  dir.push(USER_DEVICE_CONFIG_FILENAME);
  dir
}

#[cfg(not(debug_assertions))]
pub fn engine_file_path() -> PathBuf {
  let mut dir = user_config_directory();
  dir.push("engine");
  dir.push(EXECUTABLE_NAME);
  dir
}

#[cfg(debug_assertions)]
pub fn engine_file_path() -> PathBuf {
  let mut dir = PathBuf::from("c:\\Users\\qdot\\code\\intiface-cli-rs\\target\\debug");
  dir.push("intiface-cli.exe");
  dir
}