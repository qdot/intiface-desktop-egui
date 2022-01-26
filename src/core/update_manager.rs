use super::{util, IntifaceConfiguration, news_file_path};
use buttplug::util::device_configuration::load_protocol_config_from_json;
use sentry::SentryFutureExt;
use std::{
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
  },
  fs::File,
  error::Error,
  io::{self, copy},
};
use thiserror::Error;
use tracing::{error, info};
use std::sync::RwLock;

//#[cfg(debug_assertions)]
const INTIFACE_REPO_OWNER: &str = "qdot";
/*
#[cfg(not(debug_assertions))]
const INTIFACE_REPO_OWNER: &str = "intiface";
*/
const INTIFACE_DESKTOP_REPO: &str = "intiface-desktop-egui";
const INTIFACE_ENGINE_REPO: &str = "intiface-cli-rs";
// const PRERELEASE_TAG: &str = "420.69.666";
const DEVICE_CONFIG_VERSION_URL: &str = "https://buttplug-rs-device-config.buttplug.io/version";
const DEVICE_CONFIG_URL: &str = "https://buttplug-rs-device-config.buttplug.io";
const NEWS_URL: &str = "https://intiface-news.intiface.com/intiface.news.md";

#[derive(Debug, Error)]
pub enum UpdateError {
  #[error("Error establishing connection to update URL {0}: {1}")]
  ConnectionError(String, String),
  #[error("Error retreiving information from Github: {0}")]
  GithubError(String),
  #[error("Invalid data received from version check: {0}")]
  InvalidData(String),
}

#[derive(Default)]
pub struct UpdateManager {
  latest_engine_version: Arc<AtomicU32>,
  latest_device_file_version: Arc<AtomicU32>,
  latest_application_version: Arc<RwLock<Option<String>>>,
  current_application_version: String,
  current_engine_version: Arc<RwLock<Option<u32>>>,
  current_device_config_file_version: Arc<RwLock<Option<u32>>>,
  has_new_news: Arc<AtomicBool>,
  is_updating: Arc<AtomicBool>,
  has_errors: Arc<AtomicBool>,
}

impl UpdateManager {
  pub fn new() -> Self {
    let manager = Self {
      current_application_version: format!("{}", env!("VERGEN_BUILD_SEMVER")),
      ..Default::default()
    };
    UpdateManager::update_internal_versions(manager.current_engine_version.clone(), manager.current_device_config_file_version.clone());
    manager
  }

  async fn update_news(has_new_news: Arc<AtomicBool>) {
    let downloaded_news_file = reqwest::get(NEWS_URL)
      .await
      /*
      .map_err(|e| {
        UpdateError::ConnectionError(DEVICE_CONFIG_VERSION_URL.to_string(), e.to_string())
      })?
      */
      .unwrap()
      .text()
      .await
      .unwrap();
    //.map_err(|e| UpdateError::InvalidData(e.to_string()))?;
    let news_file = news_file_path();
    let news_str = if !news_file.exists() {
      "News file not available. Please run update.".to_owned()
    } else {
      tokio::fs::read_to_string(&news_file).await.unwrap()
    };
    if downloaded_news_file != news_str {
      info!("Updating news file...");
      tokio::fs::write(news_file, downloaded_news_file).await.unwrap();
      has_new_news.store(true, Ordering::SeqCst);
    }
  }

  fn update_internal_versions(current_engine_version: Arc<RwLock<Option<u32>>>, current_device_config_file_version: Arc<RwLock<Option<u32>>>) {
    if util::device_config_file_path().exists() {
      // TODO This could fail if the file is invalid.
      let config_file = std::fs::read_to_string(util::device_config_file_path()).unwrap();
      let version = load_protocol_config_from_json(&config_file, true).unwrap().version;
      info!("Device config file version: {}", version);
      *current_device_config_file_version.write().unwrap() = Some(version)
    } else {
      info!("No device config file found");
      *current_device_config_file_version.write().unwrap() = None;
    }

    if util::engine_file_path().exists() {
      match std::process::Command::new(util::engine_file_path()).args(["--serverversion"]).output() {
        Ok(process_result) => {
          if let Ok(vstr) = String::from_utf8(process_result.stdout) {
            let version_strings: Vec<&str> = vstr.split(".").collect();
            if let Ok(version) = u32::from_str_radix(version_strings[0], 10) {
              *current_engine_version.write().unwrap() = Some(version);
              return;
            } else {
              error!("Error checking engine version, cannot parse output string into version, assuming old version.");
            }
          } else {
            error!("Error checking engine version, cannot get output string.");
          }
          
          *current_engine_version.write().unwrap() = None;
        }
        Err(e) => {
          error!("Error checking engine version, cannot run process: {:?}", e);
          *current_engine_version.write().unwrap() = None;
        }
      }
    } else {
      *current_engine_version.write().unwrap() = None;
    }
  }

  pub fn current_application_version(&self) -> &'_ str {
    &self.current_application_version
  }

  pub fn current_engine_version(&self) -> Option<u32> {
    self.current_engine_version.read().unwrap().clone()
  }

  pub fn current_device_config_file_version(&self) -> Option<u32> {
    self.current_device_config_file_version.read().unwrap().clone()
  }

  pub fn check_for_updates(&self) {
    let is_updating = self.is_updating.clone();

    let device_check_fut = UpdateManager::get_latest_device_file_version(
      self.latest_device_file_version.clone(),
    );
    let engine_check_fut = UpdateManager::get_latest_engine_version(
      self.latest_engine_version.clone(),
    );
    let application_check_fut = UpdateManager::get_latest_application_version(
      self.latest_application_version.clone(),
    );

    let news_check_fut = UpdateManager::update_news(self.has_new_news.clone());
    tokio::spawn(async move {
      is_updating.store(true, Ordering::SeqCst);
      tokio::join!(
        async move { device_check_fut.await }.bind_hub(sentry::Hub::current().clone()),
        async move { engine_check_fut.await }.bind_hub(sentry::Hub::current().clone()),
        async move { application_check_fut.await }.bind_hub(sentry::Hub::current().clone()),
        async move { news_check_fut.await }.bind_hub(sentry::Hub::current().clone()),
      );
      is_updating.store(false, Ordering::SeqCst);
    });
  }

  pub fn get_updates(&self) {
    let is_updating = self.is_updating.clone();
    let should_update_device_config_file = self.needs_device_config_file_update();
    let should_update_engine = self.needs_engine_update();
    let current_engine_version = self.current_engine_version.clone();
    let current_device_config_file_version = self.current_device_config_file_version.clone();
    tokio::spawn(async move {
      is_updating.store(true, Ordering::SeqCst);
      if should_update_device_config_file {
        UpdateManager::download_device_file_update().await;
      }
      if should_update_engine {
        UpdateManager::download_engine_update().await;
      }
      UpdateManager::update_internal_versions(current_engine_version, current_device_config_file_version);
      is_updating.store(false, Ordering::SeqCst);
    });
  }

  pub fn needs_device_config_file_update(&self) -> bool {
    let current_device_config_file_version = self.current_device_config_file_version.read().unwrap();
    current_device_config_file_version.is_none() || self.latest_device_file_version.load(Ordering::SeqCst) != current_device_config_file_version.unwrap()
  }

  pub fn needs_engine_update(&self) -> bool {
    let current_engine_version = self.current_engine_version.read().unwrap();
    current_engine_version.is_none() || self.latest_engine_version.load(Ordering::SeqCst) != current_engine_version.unwrap()
  }

  pub fn needs_application_update(&self) -> bool {
    let latest_application_version = self.latest_application_version.read().unwrap();
    latest_application_version.is_some() && self.current_application_version != *latest_application_version.as_ref().unwrap()
  }

  pub fn needs_internal_updates(&self) -> bool {
    self.needs_device_config_file_update() || self.needs_engine_update()
  }

  pub fn needs_updates(&self) -> bool {
    self.needs_internal_updates() || self.needs_application_update()
  }

  pub fn has_new_news(&self) -> bool {
    self.has_new_news.load(Ordering::SeqCst)
  }

  pub fn reset_news_status(&self) {
    self.has_new_news.store(false, Ordering::SeqCst)
  }

  pub fn is_updating(&self) -> bool {
    self.is_updating.load(Ordering::SeqCst)
  }

  async fn get_latest_device_file_version(
    latest_device_file_version: Arc<AtomicU32>,
  ) {
    let version = reqwest::get(DEVICE_CONFIG_VERSION_URL)
      .await
      /*
      .map_err(|e| {
        UpdateError::ConnectionError(DEVICE_CONFIG_VERSION_URL.to_string(), e.to_string())
      })?
      */
      .unwrap()
      .text()
      .await
      .unwrap()
      //.map_err(|e| UpdateError::InvalidData(e.to_string()))?
      .parse::<u32>()
      .unwrap();
    //.map_err(|e| UpdateError::InvalidData(e.to_string()))?;
    info!("Remote device file Version: {}", version);
    latest_device_file_version.store(version, Ordering::SeqCst);
  }

  async fn get_latest_application_version(
    latest_application_version: Arc<RwLock<Option<String>>>
  ) {
    match octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_DESKTOP_REPO)
      .releases()
      .get_latest()
      .await {
      Ok(release) => {
        info!("Latest application version: {}", release.tag_name);
        *latest_application_version.write().unwrap() = Some(release.tag_name);
      },
      Err(e) => {
        error!("Can't get latest version: {:?}", e.source());
        *latest_application_version.write().unwrap() = None;
      }
    }
  }

  async fn get_latest_engine_version(
    latest_engine_version: Arc<AtomicU32>,
  ) -> Result<(), UpdateError> {
    let release = octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_ENGINE_REPO)
      .releases()
      .get_latest()
      .await
      .map_err(|e| UpdateError::GithubError(e.to_string()))?;
    let current_version_number = release.tag_name[1..].parse::<u32>().unwrap();
    info!("Remote Engine Version: {}", current_version_number);
    latest_engine_version.store(current_version_number, Ordering::SeqCst);
    Ok(())
  }

  pub async fn download_device_file_update() {
    let response = reqwest::get(DEVICE_CONFIG_URL).await.unwrap();
    let mut dest = { File::create(super::util::device_config_file_path()).unwrap() };
    let content = response.text().await.unwrap();
    copy(&mut content.as_bytes(), &mut dest).unwrap();
  }

  pub async fn download_application_update(&self, config: &IntifaceConfiguration) {
  }

  async fn download_engine_update() {
    #[cfg(target_os = "windows")]
    let platform = "win-x64";
    #[cfg(target_os = "linux")]
    let platform = "linux-x64";
    #[cfg(target_os = "darwin")]
    let platform = "macos-x64";

    let release_name = format!("intiface-cli-rs-{}-Release.zip", platform);
    let release = octocrab::instance()
      .repos(INTIFACE_REPO_OWNER, INTIFACE_ENGINE_REPO)
      .releases()
      .get_latest()
      .await
      .map_err(|e| UpdateError::GithubError(e.to_string()))
      .unwrap();

    for asset in release.assets {
      if asset.name.starts_with(&release_name) {
        info!(
          "Found release asset {} for version {}",
          asset.name, release.tag_name
        );
        info!("Getting {}", asset.browser_download_url);
        let file_bytes = reqwest::get(asset.browser_download_url)
          .await
          .unwrap()
          .bytes()
          .await
          .unwrap();
        let reader = std::io::Cursor::new(&file_bytes);
        let mut files = zip::ZipArchive::new(reader).unwrap();
        for file_idx in 0..files.len() {
          let mut file = files.by_index(file_idx).unwrap();
          if file.enclosed_name().is_none() {
            continue;
          };
          let extension = file.enclosed_name().unwrap().extension();
          if extension.is_some() && extension.unwrap() == "md" {
            info!(
              "Skipping extracting file {} from zip...",
              file.enclosed_name().unwrap().display()
            );
            continue;
          }
          let final_out_path = util::engine_file_path();
          info!(
            "Extracting file {} from zip to {:?}...",
            file.enclosed_name().unwrap().display(),
            final_out_path
          );
          if !super::engine_path().exists() {
            std::fs::create_dir_all(super::engine_path());
          }
          let mut outfile = File::create(&final_out_path).unwrap();
          io::copy(&mut file, &mut outfile).unwrap();
        }

        break;
      }
    }
  }

  pub async fn install_application(&self, config: &IntifaceConfiguration) {
  }
}