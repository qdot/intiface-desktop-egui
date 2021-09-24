use crate::core::IntifaceConfiguration;
use eframe::egui;

#[derive(Default)]
pub struct SettingsPanel {}

impl SettingsPanel {
  pub fn update(&mut self, config: &mut IntifaceConfiguration, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
      ui.collapsing("Versions and Updates", |ui| {
        ui.horizontal(|ui| {
          ui.label("Server Name");
          ui.text_edit_singleline(config.server_name_mut());
        })
      });
      ui.collapsing("Server Process Settings", |ui| {
        ui.horizontal(|ui| {
          ui.label("Server Name");
          ui.text_edit_singleline(config.server_name_mut());
        })
      });
      ui.collapsing("Server Websocket Settings", |ui| {
        ui.horizontal(|ui| {
          ui.label("Server Name");
          ui.text_edit_singleline(config.server_name_mut());
        })
      });
      ui.collapsing("Other Settings", |ui| {
        ui.horizontal(|ui| {
          ui.label("Server Name");
          ui.text_edit_singleline(config.server_name_mut());
        })
      });
    });
  }
}
