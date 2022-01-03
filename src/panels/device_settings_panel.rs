use crate::core::AppCore;
use eframe::egui;

#[derive(Default)]
pub struct DeviceSettingsPanel {}

impl DeviceSettingsPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
      ui.collapsing("Allow/Deny Devices", |ui| {
        ui.vertical(|ui| {
          for (address, config) in core.user_device_config_manager.get_user_config().clone() {
            ui.label(format!("{}", address));
            if config.allow().is_some() && ui.button("Remove Allow").clicked() {
              core
                .user_device_config_manager
                .remove_allowed_device(&address);
              core.user_device_config_manager.save_user_config();
            }
            if config.deny().is_some() && ui.button("Remove Deny").clicked() {
              core
                .user_device_config_manager
                .remove_denied_device(&address);
              core.user_device_config_manager.save_user_config();
            }
          }
          if core.process_manager.is_running() {
            //let config = core.user_device_config_manager.get_user_config();
            for device in core.process_manager.client_devices() {
              if !core
                .user_device_config_manager
                .get_user_config()
                .contains_key(&device.address)
              {
                ui.label(format!("{} ({})", device.name, device.address));
                if ui.button("Allow").clicked() {
                  core
                    .user_device_config_manager
                    .add_allowed_device(&device.address);
                  core.user_device_config_manager.save_user_config();
                }
                if ui.button("Deny").clicked() {
                  core
                    .user_device_config_manager
                    .add_denied_device(&device.address);
                  core.user_device_config_manager.save_user_config();
                }
              }
            }
          }
        });
      });
      ui.collapsing("Add/Remove Device Configurations", |_ui| {});
    });
  }
}
