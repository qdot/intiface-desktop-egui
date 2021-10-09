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
      if core.process_manager.is_running() {
        if let Some(name) = core.process_manager.client_name() {
          ui.label(format!("Client Connected: {}", name));
        } else {
          ui.label("Disconnected");
        }

        let devices = core.process_manager.client_devices();
        if !devices.is_empty() {
          ui.label("Devices Connected:");
          for device in devices {
            ui.label(format!("- {} {}", device.name, device.address));
          }
        } else {
          ui.label("No devices connected.");
        }
      }
    });
  }
}
