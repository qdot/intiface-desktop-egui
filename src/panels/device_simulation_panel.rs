use crate::core::{AppCore, SimulatedDevice};
use eframe::egui;
use core::ops::RangeInclusive;

#[derive(Default)]
pub struct DeviceSimulationPanel {}

impl DeviceSimulationPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {

    ui.collapsing("Simulated Device Settings", |ui| {
      ui.vertical(|ui|{
        let mut identifier = String::new();
        ui.horizontal(|ui| {
          ui.label("Identifier");
          ui.text_edit_singleline(&mut identifier);
        });
        let mut num_vibrators = 0u32;
        ui.horizontal(|ui| {
          ui.label("# of Vibrating Features");
          ui.add(egui::DragValue::new(&mut num_vibrators).speed(1).clamp_range(RangeInclusive::new(1u32, 10)));
        });
        let mut num_rotators = 0u32;
        ui.horizontal(|ui| {
          ui.label("# of Rotating Features");
          ui.add(egui::DragValue::new(&mut num_rotators).speed(1).clamp_range(RangeInclusive::new(1u32, 10)));
        });
        let mut num_linear = 0u32;
        ui.horizontal(|ui| {
          ui.label("# of Linear Features");
          ui.add(egui::DragValue::new(&mut num_linear).speed(1).clamp_range(RangeInclusive::new(1u32, 10)));
        });
        ui.horizontal(|ui| {
          ui.button("Add Simulated Device")
        });  
      });
    });

    if core.process_manager.is_running() {
      if ui.button("Connect").clicked() {
        let device = SimulatedDevice::new("testing");
        device.connect("ws://127.0.0.1:54817");
      }
    }
  }
}
