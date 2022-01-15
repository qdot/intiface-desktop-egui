use std::path::PathBuf;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub const USER_DEVICE_CONFIG_FILENAME: &str = "buttplug-user-device-config.json";
pub const DEVICE_CONFIG_FILENAME: &str = "buttplug-device-config.json";
pub const INTIFACE_CONFIG_FILENAME: &str = "intiface.config.json";
pub const INTIFACE_APP_DIRECTORY_NAME: &str = "IntifaceDesktopRust";
pub const INTIFACE_CONFIG_DIRECTORY_NAME: &str = "config";
pub const INTIFACE_LOGGING_DIRECTORY_NAME: &str = "logs";
pub const INTIFACE_ENGINE_DIRECTORY_NAME: &str = "engine";

#[cfg(target_os = "windows")]
const EXECUTABLE_NAME: &str = "intiface-cli.exe";
#[cfg(not(target_os = "windows"))]
const EXECUTABLE_NAME: &str = "intiface-cli";

pub fn app_path() -> PathBuf {
  let mut dir = dirs::data_dir().unwrap();
  dir.push(INTIFACE_APP_DIRECTORY_NAME);
  dir
}

pub fn user_config_path() -> PathBuf {
  let mut dir = app_path();
  dir.push(INTIFACE_CONFIG_DIRECTORY_NAME);
  dir
}

pub fn log_path() -> PathBuf {
  let mut dir = app_path();
  dir.push(INTIFACE_LOGGING_DIRECTORY_NAME);
  dir
}

pub fn config_file_path() -> PathBuf {
  let mut dir = user_config_path();
  dir.push(INTIFACE_CONFIG_FILENAME);
  dir
}

pub fn device_config_file_path() -> PathBuf {
  let mut dir = user_config_path();
  dir.push(DEVICE_CONFIG_FILENAME);
  dir
}

pub fn user_device_config_file_path() -> PathBuf {
  let mut dir = user_config_path();
  dir.push(USER_DEVICE_CONFIG_FILENAME);
  dir
}

pub fn engine_path() -> PathBuf {
  let mut dir = app_path();
  dir.push("engine");
  dir
}

#[cfg(not(debug_assertions))]
pub fn engine_file_path() -> PathBuf {
  let mut dir = engine_path();
  dir.push(EXECUTABLE_NAME);
  dir
}

#[cfg(debug_assertions)]
pub fn engine_file_path() -> PathBuf {
  let mut dir = PathBuf::from("c:\\Users\\qdot\\code\\intiface-cli-rs\\target\\debug");
  dir.push(EXECUTABLE_NAME);
  dir
}

pub fn random_string() -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .take(15)
    .map(char::from)
    .collect()
}