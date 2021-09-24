
use eframe::{egui, epi};
use crate::core::IntifaceConfiguration;

use super::panels::SettingsPanel;

#[derive(Debug, PartialEq)]
enum AppScreens {
  ServerStatus,
  Devices,
  Settings,
  Help,
  About,
  IntifaceConfiguration,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
  // Example stuff:
  label: String,
  current_screen: AppScreens,
  config: IntifaceConfiguration,
}

impl Default for TemplateApp {
  fn default() -> Self {
    Self {
      // Example stuff:
      label: "Hello World!".to_owned(),
      current_screen: AppScreens::ServerStatus,
      config: IntifaceConfiguration::default()
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
      label,
      current_screen,
      config
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
      })

      /*
      ui.horizontal(|ui| {
          ui.label("Write something: ");
          ui.text_edit_singleline(label);
      });

      ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
      if ui.button("Increment").clicked() {
          *value += 1.0;
      }

      ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
          ui.add(
              egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
          );
      });
      */
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      // The central panel the region left after adding TopPanel's and SidePanel's

      match current_screen {
        AppScreens::Settings => {
          let mut status = SettingsPanel::default();
          status.update(config, ui);
        }
        _ => {}
      }

      /*
      ui.heading("egui template TESTING");
      ui.hyperlink("https://github.com/emilk/egui_template");
      ui.add(egui::github_link_file!(
        "https://github.com/emilk/egui_template/blob/master/",
        "Source code."
      ));
      ui.label("testing 1 2 3");
      egui::warn_if_debug_build(ui);
      */
    });
  }
}
