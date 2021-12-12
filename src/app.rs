use crate::core::{load_config_file, save_config_file, AppCore, IntifaceConfiguration};
use eframe::{egui, epi};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing::{info, warn};
use super::panels::{ServerStatusPanel, SettingsPanel, LogPanel, DevicesPanel};
use time::OffsetDateTime;
use std::time::SystemTime;

#[derive(Debug, PartialEq)]
enum AppScreens {
  ServerStatus,
  Devices,
  Settings,
  Log,
  Help,
  About,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct IntifaceDesktopApp {
  current_screen: AppScreens,
  core: AppCore,
  _logging_guard: tracing_appender::non_blocking::WorkerGuard,
  _sentry_guard: Option<sentry::ClientInitGuard>
}

fn setup_logging() -> WorkerGuard {
  if !super::core::log_path().exists() {
    // If we don't, create it and add default files.
    std::fs::create_dir_all(super::core::log_path());
  }

  let filter = EnvFilter::try_from_default_env()
    .or_else(|_| EnvFilter::try_new("info"))
    .unwrap();

  let dt: OffsetDateTime = SystemTime::now().into();
  let format_str = time::format_description::parse("[year]-[month]-[day]-[hour]-[minute]-[second]").unwrap();

  let file_appender = tracing_appender::rolling::never(super::core::log_path(), format!("intiface-desktop-{}.log", dt.format(&format_str).unwrap()));
  let (non_blocking, logging_guard) = tracing_appender::non_blocking(file_appender);

  let fmt_sub = tracing_subscriber::fmt::layer()
    .with_writer(non_blocking);

  tracing_subscriber::registry()
    .with(fmt_sub)
    .with(filter)
    .with(super::panels::layer())
    .with(sentry_tracing::layer())
    .with(tracing_subscriber::fmt::layer())
    .init();

  let paths = std::fs::read_dir(super::core::log_path()).unwrap();
  let mut logs = vec!();
  for path in paths {
    let p = path.unwrap();
    if p.file_name().into_string().unwrap().contains(".log") {
      logs.push(p.path());
    }
  }
  while logs.len() > 10 {
    let log_file = logs.remove(0);
    std::fs::remove_file(log_file).unwrap();
  }
 
  logging_guard
}

impl Default for IntifaceDesktopApp {
  fn default() -> Self {
    // First off, see if we even have a configuration directory.
    if !super::core::user_config_path().exists() {
      // If we don't, create it and add default files.
      std::fs::create_dir_all(super::core::user_config_path());
      save_config_file(&serde_json::to_string(&super::core::IntifaceConfiguration::default()).unwrap()).unwrap();
      super::core::UserDeviceConfigManager::default().save_user_config();
    }

    // Now that we at least have a directory to store logs in, set up logging.
    let _logging_guard = setup_logging();    

    // See if we have an engine.
    if !super::core::engine_file_path().exists() {
      // If not, uh...
    }

    // Everything is where we expect it to be. Actually set up and run the application.
    info!("Setting up application");
    let mut core = AppCore::default();
    let json_str = load_config_file().unwrap();
    core.config = match IntifaceConfiguration::load_from_string(&json_str) {
      Ok(config) => config,
      Err(err) => {
        error!("Error while loading configuration file: {:?}", err);
        info!("Resetting configuration file.");
        core.modal_manager.set_ok_modal_dialog("Your intiface configuration file was corrupt or missing, and has been reset. You may need to update or change settings.");
        let config = super::core::IntifaceConfiguration::default();
        save_config_file(&serde_json::to_string(&config).unwrap()).unwrap();
        config
      }
    };

    const API_KEY: &str = include_str!(concat!(env!("OUT_DIR"), "/sentry_api_key.txt"));
    let _sentry_guard = if core.config.crash_reporting() && !API_KEY.is_empty() {
      info!("Crash reporting activated.");
      Some(sentry::init((API_KEY, sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
      })))
    } else {
      warn!("Crash reporting not activated.");
      if API_KEY.is_empty() {
        warn!("No crash reporting API key available.");
      }
      None
    };
    Self {
      current_screen: AppScreens::ServerStatus,
      core,
      _logging_guard,
      _sentry_guard
    }
  }
}


impl epi::App for IntifaceDesktopApp {
  fn name(&self) -> &str {
    "Intiface Desktop"
  }

  /// Called by the framework to load old app state (if any).
  #[cfg(feature = "persistence")]
  fn setup(
    &mut self,
    _ctx: &egui::CtxRef,
    _frame: &mut epi::Frame<'_>,
    storage: Option<&dyn epi::Storage>,
  ) {
    if let Some(storage) = storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }
  }

  /// Called by the frame work to save state before shutdown.
  #[cfg(feature = "persistence")]
  fn save(&mut self, storage: &mut dyn epi::Storage) {
    epi::set_value(storage, epi::APP_KEY, self);
  }

  /// Called each time the UI needs repainting, which may be many times per second.
  /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
  fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
    let Self {
      current_screen,
      core,
      _logging_guard: _,
      _sentry_guard: _,
    } = self;

    // Examples of how to create different panels and windows.
    // Pick whichever suits you.
    // Tip: a good default choice is to just keep the `CentralPanel`.
    // For inspiration and more examples, go to https://emilk.github.io/egui

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
      // The top panel is often a good place for a menu bar:
      egui::menu::bar(ui, |ui| {
        egui::menu::menu_button(ui, "File", |ui| {
          if ui.button("Quit").clicked() {
            frame.quit();
          }
        });
      });
    });


    /*
    egui::TopBottomPanel::bottom("bottom_panel").resizable(true).default_height(40.0).show(ctx, |ui| {
      // The top panel is often a good place for a menu bar:
      egui::ScrollArea::auto_sized().show_viewport(ui, |ui, r| {
        ui.add(LogPanel);
      });
    });
    */  
    if let Some(d) = core.modal_manager.get_modal_dialog() {
      egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
          ui.set_max_height(ui.available_height() * 0.25);
          ui.set_max_width(ui.available_width() * 0.5);
          d.render(core, ui);
        });
      });
    } else {
      egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.heading("Intiface Desktop v41");

        ui.vertical(|ui| {
          ui.selectable_value(current_screen, AppScreens::ServerStatus, "Server Status");
          ui.selectable_value(current_screen, AppScreens::Devices, "Devices");
          ui.selectable_value(current_screen, AppScreens::Settings, "Settings");
          ui.selectable_value(current_screen, AppScreens::Log, "Log");
          ui.selectable_value(current_screen, AppScreens::Help, "Help");
          ui.selectable_value(current_screen, AppScreens::About, "About");
        });

        egui::warn_if_debug_build(ui);
      });

      egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show_viewport(ui, |ui, r| match current_screen {
          AppScreens::ServerStatus => ServerStatusPanel::default().update(core, ui),
          AppScreens::Devices => DevicesPanel::default().update(core, ui),
          AppScreens::Settings => SettingsPanel::default().update(core, ui),
          AppScreens::Log => {
            ui.add(LogPanel);
          },
          _ => {}
        });
      });
    }
  }
}
