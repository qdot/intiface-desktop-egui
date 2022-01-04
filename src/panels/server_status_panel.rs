use crate::core::AppCore;
use eframe::egui::{self, Color32, Frame, RichText, TextStyle};

#[derive(Default)]
pub struct ServerStatusPanel {}

impl ServerStatusPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    egui::SidePanel::left("ServerStatusButtonPanel")
      .resizable(false)
      .frame(Frame::none())
      .show_inside(ui, |ui| {
        ui.with_layout(
          egui::Layout::centered_and_justified(egui::Direction::RightToLeft),
          |ui| {
            ui.set_max_width(100f32);

            let server_button = if core.process_manager.is_running() {
              ui.button(
                RichText::new("⬛")
                  .color(Color32::LIGHT_RED)
                  .text_style(TextStyle::Heading),
              ).on_hover_text("Stop Server")
            } else {
              ui.button(
                RichText::new("▶")
                  .color(Color32::GREEN)
                  .text_style(TextStyle::Heading),
              ).on_hover_text("Start Server")
            };
            if server_button.clicked() {
              if core.process_manager.is_running() {
                core.process_manager.stop();
              } else {
                core.process_manager.run(&core.config);
              }
            }
          },
        );
      });

    egui::SidePanel::right("ServerStatusIconPanel")
      .resizable(false)
      .frame(Frame::none())
      .show_inside(ui, |ui| {
        ui.horizontal(|ui| {
          if !core.process_manager.is_running() {
            ui.label(
              RichText::new("🙉")
                .color(Color32::LIGHT_RED)
                .text_style(TextStyle::Heading),
            ).on_hover_text("Server not running");
          } else if !core.process_manager.client_name().is_some() {
            ui.label(
              RichText::new("👂")
                .color(Color32::LIGHT_BLUE)
                .text_style(TextStyle::Heading),
            ).on_hover_text("Server running, waiting for client connection");
          } else {
            ui.label(
              RichText::new("📞")
                .color(Color32::GREEN)
                .text_style(TextStyle::Heading),
            ).on_hover_text("Server connected to client");
          }
          ui.vertical(|ui| {
            if core.config.has_error_message() {
              if ui.button(RichText::new("！").color(Color32::LIGHT_RED)).on_hover_text("New error messages in log").clicked() {
                *core.config.force_open_log_mut() = true;
              }
            } else {
              ui.add(egui::Button::new("！").frame(false).sense(egui::Sense { click: false, drag: false, focusable: false} )).on_hover_text("No new error messages in log");
            }
            if core.update_manager.needs_updates() {
              if ui.button(RichText::new("⮉").color(Color32::WHITE)).on_hover_text("Updates available").clicked() {
                *core.config.force_open_updates_mut() = true;
              }
            } else {
              ui.add(egui::Button::new("⮉").frame(false).sense(egui::Sense { click: false, drag: false, focusable: false} )).on_hover_text("No updates available");
            }
            ui.button(RichText::new("？").color(Color32::GREEN)).on_hover_text("Go to docs/help website");
          });
        });
      });

    let mut available_height = 0f32;
    egui::CentralPanel::default().show_inside(ui, |ui| {
      ui.vertical(|ui| {
        ui.horizontal(|ui| {
          ui.label(RichText::new("Server Status:").strong());
          if core.process_manager.is_running() {
            ui.label("Server Running");
          } else {
            ui.label("Server Disconnected");
          }
        });
        ui.horizontal(|ui| {
          ui.label(RichText::new("Client Status:").strong());
          if core.process_manager.is_running() {
            if let Some(name) = core.process_manager.client_name() {
              ui.label(name);
            } else {
              ui.label("Disconnected");
            }
          } else {
            ui.label("Server Not Available");
          }
        });
        ui.horizontal(|ui| {
          ui.label(RichText::new("Device Status:").strong());
          if core.process_manager.is_running() {
            let devices = core.process_manager.client_devices();
            if !devices.is_empty() {
              for device in devices {
                ui.label(format!("{}, ", device.name));
              }
            } else {
              ui.label("No devices connected.");
            }
          } else {
            ui.label("Server Not Available");
          }
        });
      });
      available_height = ui.min_size().y;
    });
    let id = ui.make_persistent_id("ServerStatusPanel::Height");
    let height = ui
      .memory()
      .data
      .get_temp_mut_or_insert_with(id, || available_height + 20f32)
      .clone();
    ui.set_min_height(height);
  }
}
