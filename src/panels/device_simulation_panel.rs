use crate::core::{AppCore, SimulatedDevice};
use eframe::egui;
use std::{collections::HashMap, sync::atomic::Ordering};
use core::ops::RangeInclusive;

#[derive(Default)]
pub struct DeviceSimulationPanel {
  name: String,
  num_vibrators: u32,
  num_rotators: u32,
  num_linear: u32,
  devices: HashMap<String, SimulatedDevice>
}

impl DeviceSimulationPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {

    ui.collapsing("Simulated Device Settings", |ui| {
      ui.vertical(|ui|{
        ui.horizontal(|ui| {
          ui.label("Display Name");
          ui.text_edit_singleline(&mut self.name);
        });
        ui.horizontal(|ui| {
          ui.label("# of Vibrating Features");
          ui.add(egui::DragValue::new(&mut self.num_vibrators).speed(0.1).clamp_range(RangeInclusive::new(0u32, 10)));
        });
        ui.horizontal(|ui| {
          ui.label("# of Rotating Features");
          ui.add(egui::DragValue::new(&mut self.num_rotators).speed(0.1).clamp_range(RangeInclusive::new(0u32, 10)));
        });
        ui.horizontal(|ui| {
          ui.label("# of Linear Features");
          ui.add(egui::DragValue::new(&mut self.num_linear).speed(0.1).clamp_range(RangeInclusive::new(0u32, 10)));
        });
        ui.horizontal(|ui| {
          if ui.button("Add Simulated Device").clicked() {
            core.user_device_config_manager.add_simulated_device(&self.name, self.num_vibrators, self.num_rotators, self.num_linear);
          }
        });  
      });
    });
    ui.vertical(|ui| {
      for device in core.user_device_config_manager.get_simulated_devices() {
        let identifier = &device.identifier().as_ref().expect("We should always have a name here")[0];
        ui.label(format!("{}", device.name().as_ref().expect("Should have name").get("en-us").expect("always have en-us")));
        if core.process_manager.is_running() {
          if !self.devices.contains_key(identifier) {
            if ui.button("Connect").clicked() {
              let simulated_device = SimulatedDevice::new(&device);
              simulated_device.connect("ws://127.0.0.1:54817");
              self.devices.insert(identifier.clone(), simulated_device);
            }
          } else {
            let actuators = self.devices.get(identifier).expect("Already checked").actuators();
            for (index, vibrator) in actuators.vibrators.iter().enumerate() {
              let mut speed = vibrator.load(Ordering::SeqCst);
              ui.add(egui::Slider::new(&mut speed, 0..=100).text(format!("Vibrator {index}")));
            }
            for (index, rotator) in actuators.rotators.iter().enumerate() {
              let mut speed = rotator.load(Ordering::SeqCst);
              ui.add(egui::Slider::new(&mut speed, -100..=100).text(format!("Rotator {index}")));
            }
            if !actuators.linear.is_empty() {
              let linear_values = self.devices.get_mut(identifier).expect("Already checked").update_linear();
              for (index, linear_position) in linear_values.iter().enumerate() {
                let mut linear_position = *linear_position;
                ui.add(egui::Slider::new(&mut linear_position, 0..=100).text(format!("Linear {index}")));
              }              
            }
            if ui.button("Disconnect").clicked() {
              {
                let device = self.devices.get(identifier).expect("Already checked for existence");
                device.disconnect();
              }
              let _ = self.devices.remove(identifier);
            }
          }
        } else {
          if ui.button("Remove").clicked() {
            core.user_device_config_manager.remove_simulated_device(&device);
          }
        }
      }
    });
  }
}
