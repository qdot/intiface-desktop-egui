use super::panels::{
  AboutPanel, DeviceSettingsPanel, DeviceTestPanel, FirstUsePanel, LogPanel, ServerStatusPanel, SettingsPanel,
};
use crate::core::{load_config_file, save_config_file, AppCore, IntifaceConfiguration};
use eframe::{egui, epi};
use egui::{FontDefinitions, FontFamily, TextStyle};
use std::{cell::Cell, rc::Rc, time::SystemTime};
use time::OffsetDateTime;
use tracing::{info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, PartialEq)]
enum AppScreens {
  DeviceSettings,
  DeviceTest,
  Settings,
  Log,
  About,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

pub struct IntifaceDesktopApp {
  current_screen: AppScreens,
  core: AppCore,
  expanded: Rc<Cell<bool>>,
  _logging_guard: tracing_appender::non_blocking::WorkerGuard,
  _sentry_guard: Option<sentry::ClientInitGuard>,
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
  let format_str =
    time::format_description::parse("[year]-[month]-[day]-[hour]-[minute]-[second]").unwrap();

  let file_appender = tracing_appender::rolling::never(
    super::core::log_path(),
    format!("intiface-desktop-{}.log", dt.format(&format_str).unwrap()),
  );
  let (non_blocking, logging_guard) = tracing_appender::non_blocking(file_appender);

  let fmt_sub = tracing_subscriber::fmt::layer().with_writer(non_blocking);

  tracing_subscriber::registry()
    .with(fmt_sub)
    .with(filter)
    .with(super::panels::layer())
    .with(sentry_tracing::layer())
    .with(tracing_subscriber::fmt::layer())
    .init();

  let paths = std::fs::read_dir(super::core::log_path()).unwrap();
  let mut logs = vec![];
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
      save_config_file(
        &serde_json::to_string(&super::core::IntifaceConfiguration::default()).unwrap(),
      )
      .unwrap();
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

    //const API_KEY: &str = include_str!(concat!(env!("OUT_DIR"), "/sentry_api_key.txt"));
    const API_KEY: &str =
      "https://ef3893f409824091806a79fcaa3dbf37@o78478.ingest.sentry.io/6078647";
    let _sentry_guard = if core.config.crash_reporting() && !API_KEY.is_empty() {
      info!("Crash reporting activated.");
      Some(sentry::init((
        API_KEY,
        sentry::ClientOptions {
          release: sentry::release_name!(),
          ..Default::default()
        },
      )))
    } else {
      warn!("Crash reporting not activated.");
      if API_KEY.is_empty() {
        warn!("No crash reporting API key available.");
      }
      None
    };
    info!("App created successfully.");
    Self {
      current_screen: AppScreens::DeviceSettings,
      core,
      expanded: Rc::new(Cell::new(false)),
      _logging_guard,
      _sentry_guard,
    }
  }
}

impl epi::App for IntifaceDesktopApp {
  fn name(&self) -> &str {
    #[cfg(debug_assertions)]
    return "Intiface Desktop - DEBUG BUILD";

    #[cfg(not(debug_assertions))]
    return "Intiface Desktop";
  }

  /// Called by the framework to load old app state (if any).
  //#[cfg(feature = "persistence")]
  fn setup(
    &mut self,
    ctx: &egui::CtxRef,
    _frame: &mut epi::Frame<'_>,
    storage: Option<&dyn epi::Storage>,
  ) {
    /*
    if let Some(storage) = storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }
    */
    // Large button text via overriding the HEADING style.
    let mut fonts = FontDefinitions::default();

    fonts
      .family_and_size
      .insert(TextStyle::Heading, (FontFamily::Proportional, 48.0));

    ctx.set_fonts(fonts);
  }

  /// Called by the frame work to save state before shutdown.
  #[cfg(feature = "persistence")]
  fn save(&mut self, storage: &mut dyn epi::Storage) {
    epi::set_value(storage, epi::APP_KEY, self);
  }

  fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
    let Self {
      current_screen,
      core,
      expanded: _,
      _logging_guard: _,
      _sentry_guard: _,
    } = self;

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
        ui.with_layout(
          egui::Layout::centered_and_justified(egui::Direction::TopDown),
          |ui| {
            ui.set_max_height(ui.available_height() * 0.25);
            ui.set_max_width(ui.available_width() * 0.5);
            d.render(core, ui);
          },
        );
      });
    } else if !core.config.has_run_first_use() {
      egui::CentralPanel::default().show(ctx, |ui| {
        FirstUsePanel::default().update(core, ui);
      });
    } else {
      let mut available_minimized_width = 0f32;
      let mut available_minimized_height = 0f32;
      egui::TopBottomPanel::top("top_panel").resizable(false).show(ctx, |ui| {
        ServerStatusPanel::default().update(core, ui);
        //available_minimized_width = ui.available_width
        available_minimized_height += ui.min_size().y;
      });
      let expanded = self.expanded.clone();
      egui::TopBottomPanel::bottom("bottom_panel").frame(egui::Frame::none()).show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
              if !core.config.show_extended_ui() {
                if ui.button("⏷").clicked() {
                  expanded.set(true);
                  *core.config.show_extended_ui_mut() = true;
                }
              } else {
                if ui.button("⏶").clicked() {
                  *core.config.show_extended_ui_mut() = false;
                }
              }
            },
          );
        });
        available_minimized_height += ui.min_size().y;
      });
      if core.config.show_extended_ui() {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
          ui.vertical(|ui| {
            ui.selectable_value(current_screen, AppScreens::DeviceSettings, "Device Settings");
            ui.selectable_value(current_screen, AppScreens::DeviceTest, "Device Test");
            ui.selectable_value(current_screen, AppScreens::Settings, "App Settings");
            ui.selectable_value(current_screen, AppScreens::Log, "App Log");
            ui.selectable_value(current_screen, AppScreens::About, "Help/About");
          });
          available_minimized_height += ui.min_size().y;
        });
        egui::CentralPanel::default().show(ctx, |ui| {
          egui::ScrollArea::vertical()
            .id_source("main_panel")
            .show(ui, |ui| {
              ui.set_min_width(ui.available_width());
              match current_screen {
                AppScreens::DeviceSettings => DeviceSettingsPanel::default().update(core, ui),
                AppScreens::DeviceTest => DeviceTestPanel::default().update(core, ui),
                AppScreens::Settings => SettingsPanel::default().update(core, ui),
                AppScreens::About => AboutPanel::default().update(core, ui),
                AppScreens::Log => LogPanel::default().update(ui),
              };
            });
        });
        if expanded.get() {
          info!("Resetting window height");
          expanded.set(false);

          frame.set_window_size(egui::vec2(500f32, available_minimized_height + 30f32));
        }
      } else {
        frame.set_window_size(egui::vec2(500f32, available_minimized_height + 8f32));
      }
    }

    // Run continuously for now, see what this does to CPU.
    ctx.request_repaint();
  }
}
