use crate::{
  core::{engine_file_path, AppCore},
};
use super::grid::{GridBuilder, Padding, Size};
use eframe::egui::{self, Button, Color32, RichText};
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

const STATUS_PANEL_HEIGHT: f32 = 65f32;
const LARGE_ICON_SIZE: f32 = 48f32;

#[derive(Default)]
pub struct ServerStatusPanel {}

impl ServerStatusPanel {
  pub fn update(
    &mut self,
    core: &mut AppCore,
    has_error_message: Arc<AtomicBool>,
    ui: &mut egui::Ui,
  ) {
    ui.set_min_height(STATUS_PANEL_HEIGHT + 4.0);
    ui.set_max_height(STATUS_PANEL_HEIGHT + 4.0);
    GridBuilder::new(ui, Padding::new(2.0, 1.0))
      .size(Size::Absolute(STATUS_PANEL_HEIGHT))
      .size(Size::Remainder)
      .size(Size::Absolute(50.0))
      .size(Size::Absolute(20.0))
      .horizontal(|mut grid| {
        grid.cell(|ui| {
          if engine_file_path().exists() {
            let server_button = if core.process_manager.is_running() {
              ui.add_sized(
                [STATUS_PANEL_HEIGHT, STATUS_PANEL_HEIGHT],
                Button::new(
                  RichText::new("⬛")
                    .color(Color32::LIGHT_RED)
                    .size(48.0),
                ),
              )
              .on_hover_text("Stop Server")
            } else {
              ui.add_sized(
                [STATUS_PANEL_HEIGHT, STATUS_PANEL_HEIGHT],
                Button::new(
                  RichText::new("▶")
                    .color(Color32::GREEN)
                    .size(LARGE_ICON_SIZE),
                ),
              )
              .on_hover_text("Start Server")
            };
            if server_button.clicked() {
              if core.process_manager.is_running() {
                core.process_manager.stop();
              } else {
                core.process_manager.run(&core.config);
              }
            }
          } else {
            ui.button(
              RichText::new("🗙")
                .color(Color32::WHITE)
                .size(LARGE_ICON_SIZE),
            )
            .on_hover_text("Server not available, please run upgrade process.");
          }
        });
        grid.cell(|ui| {
          ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown).with_cross_align(egui::Align::Center),
            |ui| {
              ui.vertical_centered(|ui| {
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
            },
          );
        });
        grid.cell(|ui| {
          if !core.process_manager.is_running() {
            ui.label(
              RichText::new("🙉")
                .color(Color32::LIGHT_RED)
                .size(LARGE_ICON_SIZE),
            )
            .on_hover_text("Server not running");
          } else if !core.process_manager.client_name().is_some() {
            ui.label(
              RichText::new("👂")
                .color(Color32::LIGHT_BLUE)
                .size(LARGE_ICON_SIZE),
            )
            .on_hover_text("Server running, waiting for client connection");
          } else {
            ui.label(
              RichText::new("📞")
                .color(Color32::GREEN)
                .size(LARGE_ICON_SIZE),
            )
            .on_hover_text("Server connected to client");
          }
        });
        grid.cell(|ui| {
          ui.vertical(|ui| {
            if has_error_message.load(Ordering::SeqCst) {
              if ui
                .button(RichText::new("！").color(Color32::LIGHT_RED))
                .on_hover_text("New error messages in log")
                .clicked()
              {
                has_error_message.store(false, Ordering::SeqCst);
                *core.config.force_open_log_mut() = true;
              }
            } else {
              ui.add(egui::Button::new("！").frame(false).sense(egui::Sense {
                click: false,
                drag: false,
                focusable: false,
              }))
              .on_hover_text("No new error messages in log");
            }
            if core.update_manager.needs_updates() {
              if ui
                .button(RichText::new("⮉").color(Color32::WHITE))
                .on_hover_text("Updates available")
                .clicked()
              {
                *core.config.force_open_updates_mut() = true;
              }
            } else {
              ui.add(egui::Button::new("⮉").frame(false).sense(egui::Sense {
                click: false,
                drag: false,
                focusable: false,
              }))
              .on_hover_text("No updates available");
            }
            if core.config.unread_news() {
              if ui
                .button(RichText::new("📰").color(Color32::WHITE))
                .on_hover_text("Unread news available")
                .clicked()
              {
                *core.config.force_open_news_mut() = true;
              }
            } else {
              ui.add(egui::Button::new("📰").frame(false).sense(egui::Sense {
                click: false,
                drag: false,
                focusable: false,
              }))
              .on_hover_text("No new news");
            }
          });
        });
      });
  }
}
