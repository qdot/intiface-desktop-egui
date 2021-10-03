use crate::core::AppCore;
use eframe::egui;

#[derive(Default)]
pub struct ServerStatusPanel {}

impl ServerStatusPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    ui.vertical(|ui| {

      let server_button = if core.process_manager.is_running() {
        ui.button("Stop Server")
      } else {
        ui.button("Start Server")
      };
      if server_button.clicked() {
        if core.process_manager.is_running() {
          core.process_manager.stop();
        } else {
          core.process_manager.run(&core.config);
        }
      }
    });
  }
}
