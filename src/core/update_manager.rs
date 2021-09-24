use thiserror::Error;

use super::IntifaceConfiguration;

const BUTTPLUG_REPO_OWNER: &str = "buttplugio";
const INTIFACE_REPO_OWNER: &str = "intiface";
const INTIFACE_DESKTOP_REPO: &str = "intiface-desktop";
const INTIFACE_ENGINE_REPO: &str = "intiface-cli-rs";
const PRERELEASE_TAG: &str = "420.69.666";
const DEVICE_CONFIG_VERSION_URL: &str = "https://buttplug-rs-device-config.buttplug.io/version";
const DEVICE_CONFIG_URL: &str = "https://buttplug-rs-device-config.buttplug.io";

#[derive(Debug, Error)]
pub enum UpdateError {
  #[error("Error establishing connection to update URL {0}: {1}")]
  ConnectionError(String, String),
  #[error("Error retreiving information from Github: {0}")]
  GithubError(String),
  #[error("Invalid data received from version check: {0}")]
  InvalidData(String),
}

pub async fn check_for_device_file_update(
  config: &IntifaceConfiguration,
) -> Result<bool, UpdateError> {
  let version = reqwest::get(DEVICE_CONFIG_VERSION_URL)
    .await
    .map_err(|e| {
      UpdateError::ConnectionError(DEVICE_CONFIG_VERSION_URL.to_string(), e.to_string())
    })?
    .text()
    .await
    .map_err(|e| UpdateError::InvalidData(e.to_string()))?
    .parse::<u32>()
    .map_err(|e| UpdateError::InvalidData(e.to_string()))?;
  Ok(version > *config.current_device_file_version())
}

pub async fn check_for_application_update(
  config: &IntifaceConfiguration,
) -> Result<bool, UpdateError> {
  let release = octocrab::instance()
    .repos(INTIFACE_REPO_OWNER, INTIFACE_DESKTOP_REPO)
    .releases()
    .get_latest()
    .await
    .map_err(|e| UpdateError::GithubError(e.to_string()))?;
  // TODO: Implement version update check
  //Ok(release.tag_name != (*self.config.read().await).current_application_tag)
  unimplemented!("Need to implement version update check.")
}

pub async fn check_for_engine_update(config: &IntifaceConfiguration) -> Result<bool, UpdateError> {
  let release = octocrab::instance()
    .repos(INTIFACE_REPO_OWNER, INTIFACE_ENGINE_REPO)
    .releases()
    .get_latest()
    .await
    .map_err(|e| UpdateError::GithubError(e.to_string()))?;
  Ok(release.tag_name != *config.current_engine_version())
}

pub async fn download_device_file_update(config: &IntifaceConfiguration) {
}

pub async fn download_application_update(config: &IntifaceConfiguration) {
}

pub async fn download_engine_update(config: &IntifaceConfiguration) {
}

pub async fn install_engine(config: &IntifaceConfiguration) {
}

pub async fn install_application(config: &IntifaceConfiguration) {
}

#[cfg(test)]
mod test {
  use super::super::IntifaceConfiguration;
  use super::*;

  #[test]
  fn test_device_file_update() {
    // Create the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async move {
      let config = IntifaceConfiguration::default();
      // Should always return true.
      assert!(check_for_device_file_update(&config).await.unwrap());
    })
  }

  #[test]
  fn test_engine_update() {
    // Create the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Execute the future, blocking the current thread until completion
    rt.block_on(async move {
      let config = IntifaceConfiguration::default();
      // Should always return true.
      assert!(check_for_engine_update(&config).await.unwrap());
    })
  }
}
