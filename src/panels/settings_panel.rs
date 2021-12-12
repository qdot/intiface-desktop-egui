use crate::{core::{save_config_file, AppCore, ModalDialog}};
use eframe::egui;

#[derive(Default)]
pub struct ResetIntifaceModalDialog {}

impl ModalDialog for ResetIntifaceModalDialog {
  fn render(&self, core: &mut AppCore, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
      ui.label("You are about to reset your user and device configurations for Intiface Desktop. Are you sure you want to do this?");
      ui.horizontal(|ui| {
        if ui.button("Ok").clicked() {
  
        }
        if ui.button("Cancel").clicked() {
          core.modal_manager.clear_modal_dialog();
        }
      });
    });
  }
}

#[derive(Default)]
pub struct SettingsPanel {}

impl SettingsPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    if core.process_manager.is_running() {
      ui.label("Cannot change settings while server is running.");
      return;
    }

    let original_config = core.config.clone();

    ui.vertical(|ui| {
      ui.collapsing("General", |ui| {
        ui.checkbox(core.config.show_notifications_mut(), "Desktop Notifications");
        ui.checkbox(core.config.crash_reporting_mut(), "Crash Reporting");
      });
      ui.collapsing("Versions and Updates", |ui| {
        ui.horizontal(|ui| {
          if !core.update_manager.is_updating() {
            if core.update_manager.needs_updates() {
              if ui.button("Get Updates").clicked() {
                core.update_manager.get_updates();
              }
            }
            if ui.button("Check For Updates").clicked() {
              core.update_manager.check_for_updates(&core.config);
            }
          } else {
            ui.label("Waiting for update check to finish...");
          }
        });
      });
      ui.collapsing("Device Connection Types", |ui| {
        ui.vertical(|ui| {
          ui.checkbox(core.config.with_bluetooth_le_mut(), "Bluetooth LE");
          ui.checkbox(core.config.with_xinput_mut(), "XInput");
          ui.checkbox(
            core.config.with_lovense_connect_service_mut(),
            "Lovense Connect Service",
          );
          ui.checkbox(
            core.config.with_lovense_hid_dongle_mut(),
            "Lovense HID Dongle",
          );
          ui.checkbox(
            core.config.with_lovense_serial_dongle_mut(),
            "Lovense Serial Dongle",
          );
          ui.checkbox(core.config.with_hid_mut(), "HID");
          ui.checkbox(core.config.with_serial_port_mut(), "Serial Ports");
        });
      });

      ui.collapsing("Server Process Settings", |ui| {
        ui.horizontal(|ui| {
          ui.checkbox(
            core.config.start_server_on_startup_mut(),
            "Start Server When Intiface Desktop Launches",
          );
        });
        ui.horizontal(|ui| {
          ui.label("Server Name");
          ui.text_edit_singleline(core.config.server_name_mut());
        });
        ui.horizontal(|ui| {
          ui.label("Server Log Level");
          ui.selectable_value(core.config.server_log_level_mut(), tracing::Level::ERROR.to_string(), "Error");
          ui.selectable_value(core.config.server_log_level_mut(), tracing::Level::WARN.to_string(), "Warn");
          ui.selectable_value(core.config.server_log_level_mut(), tracing::Level::INFO.to_string(), "Info");
          ui.selectable_value(core.config.server_log_level_mut(), tracing::Level::DEBUG.to_string(), "Debug");
          ui.selectable_value(core.config.server_log_level_mut(), tracing::Level::TRACE.to_string(), "Trace");
        });
      });
      ui.collapsing("Server Websocket Settings", |ui| {
        ui.horizontal(|ui| {
          ui.label("Websocket Port");
          let mut port_str = core.config.websocket_server_insecure_port().to_string();
          let response = ui.text_edit_singleline(&mut port_str);
          if response.changed() {
            match u16::from_str_radix(&port_str, 10) {
              Ok(port) => {
                core.config.set_websocket_server_insecure_port(port);
              }
              Err(_) => {}
            }
          }
        });
        ui.checkbox(
          core.config.websocket_server_all_interfaces_mut(),
          "Listen on all network interfaces.",
        );
      });
      ui.collapsing("Other Settings", |ui| {
        ui.horizontal(|ui| {
          if ui.button("Reset Intiface Configuration").clicked() {
            core.modal_manager.set_modal_dialog(ResetIntifaceModalDialog {});
          }
        })
      });
      #[cfg(debug_assertions)]
      ui.collapsing("Debug", |ui| {
        ui.horizontal(|ui| {
          if ui.button("Crash process").clicked() {
            panic!("Crashing due to request.")
          }
        })
      });
    });

    if core.config != original_config {
      save_config_file(&serde_json::to_string(&core.config).unwrap()).unwrap();
    }
  }
}
