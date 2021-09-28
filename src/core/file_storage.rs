use std::{io, fs};
use super::util::config_file_path;

pub fn load_config_file() -> Result<String, String> {
  if !config_file_path().exists() {
    Err("Cannot find config file".to_owned())
  } else {
    Ok(String::from_utf8(fs::read(config_file_path()).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?)
  }
}

pub fn save_config_file(config_str: &str) -> Result<(), io::Error> {
  Ok(fs::write(config_file_path(), config_str)?)
}