use crate::core::{load_config_file, AppCore, IntifaceConfiguration};
use eframe::{egui, epi};

use super::panels::{ServerStatusPanel, SettingsPanel};

#[derive(Debug, PartialEq)]
enum AppScreens {
  ServerStatus,
  Devices,
  Settings,
  Help,
  About,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
  current_screen: AppScreens,
  core: AppCore,
}

impl Default for TemplateApp {
  fn default() -> Self {
    let mut core = AppCore::default();
    let json_str = load_config_file().unwrap();
    core.config = IntifaceConfiguration::load_from_string(&json_str).unwrap();
    Self {
      current_screen: AppScreens::ServerStatus,
      core,
    }
  }
}

impl epi::App for TemplateApp {
  fn name(&self) -> &str {
    "egui template"
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
    } = self;

    // Examples of how to create different panels and windows.
    // Pick whichever suits you.
    // Tip: a good default choice is to just keep the `CentralPanel`.
    // For inspiration and more examples, go to https://emilk.github.io/egui

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
      // The top panel is often a good place for a menu bar:
      egui::menu::bar(ui, |ui| {
        egui::menu::menu(ui, "File", |ui| {
          if ui.button("Quit").clicked() {
            frame.quit();
          }
        });
      });
    });

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
      ui.heading("Intiface Desktop v41");

      ui.vertical(|ui| {
        ui.selectable_value(current_screen, AppScreens::ServerStatus, "Server Status");
        ui.selectable_value(current_screen, AppScreens::Devices, "Devices");
        ui.selectable_value(current_screen, AppScreens::Settings, "Settings");
        ui.selectable_value(current_screen, AppScreens::Help, "Help");
        ui.selectable_value(current_screen, AppScreens::About, "About");
      });

      egui::warn_if_debug_build(ui);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      // The central panel the region left after adding TopPanel's and SidePanel's

      egui::ScrollArea::auto_sized().show_viewport(ui, |ui, r| match current_screen {
        AppScreens::ServerStatus => {
          let mut status = ServerStatusPanel::default();
          status.update(core, ui);
        }
        AppScreens::Settings => {
          let mut settings = SettingsPanel::default();
          settings.update(core, ui);
        }
        _ => {}
      });
    });
  }
}
