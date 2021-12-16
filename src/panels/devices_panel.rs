use crate::core::AppCore;
use buttplug::{client::{ButtplugClient, RotateCommand, VibrateCommand, ButtplugClientEvent}, connector::{ButtplugRemoteClientConnector, ButtplugWebsocketClientTransport}, core::messages::{ButtplugCurrentSpecDeviceMessageType, serializer::ButtplugClientJSONSerializer}};
use eframe::egui;
use std::sync::Arc;
use futures::StreamExt;
use tracing::info;
use sentry::SentryFutureExt;

#[derive(Default)]
pub struct DevicesPanel {}

impl DevicesPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
      ui.collapsing("Allow/Deny Devices", |ui| {
        ui.vertical(|ui| {
          for (address, config) in core.user_device_config_manager.get_user_config().clone() {
            ui.label(format!("{}", address));
            if config.allow().is_some() && ui.button("Remove Allow").clicked() {
              core.user_device_config_manager.remove_allowed_device(&address);
              core.user_device_config_manager.save_user_config();
            }
            if config.deny().is_some() && ui.button("Remove Deny").clicked() {
              core.user_device_config_manager.remove_denied_device(&address);
              core.user_device_config_manager.save_user_config();
            }
          }
          if core.process_manager.is_running() {
            //let config = core.user_device_config_manager.get_user_config();
            for device in core.process_manager.client_devices() {
              if !core.user_device_config_manager.get_user_config().contains_key(&device.address) {
                ui.label(format!("{} ({})", device.name, device.address));
                if ui.button("Allow").clicked() {
                  core.user_device_config_manager.add_allowed_device(&device.address);
                  core.user_device_config_manager.save_user_config();
                }
                if ui.button("Deny").clicked() {
                  core.user_device_config_manager.add_denied_device(&device.address);
                  core.user_device_config_manager.save_user_config();
                }
              }
            }
          }
        });
      });      
      ui.collapsing("Add/Remove Device Configurations", |ui| {});
      if core.process_manager.is_running() {
        ui.collapsing("Device Testing", |ui| {
          let id = ui.make_persistent_id("DevicesPanel::ButtplugClient");
          let maybe_client = ui.memory().data.get_temp::<Arc<ButtplugClient>>(id).clone();
          if let Some(client) = maybe_client {
            ui.vertical(|ui| {
              ui.label("Client connected.");
              if ui.button("Scan for Devices").clicked() {
                let client_clone = client.clone();
                tokio::spawn(async move {
                  client_clone.start_scanning().await;
                }.bind_hub(sentry::Hub::current().clone()));
              }
              for device in client.devices() {
                ui.collapsing(format!("{}", device.name), |ui| {
                  if device.allowed_messages.contains_key(&ButtplugCurrentSpecDeviceMessageType::VibrateCmd) {
                    let vibrate_id = ui.make_persistent_id(format!("DevicesPanel::Vibrate::{}", device.index()));
                    let mut vibrate_value = ui.memory().data.get_temp_mut_or_default::<f64>(vibrate_id).clone();
                    if ui.add(egui::Slider::new::<f64>(&mut vibrate_value, 0.0..=1.0).text("Vibration Level")).changed() {
                      let device_clone = device.clone();
                      tokio::spawn(async move {
                        device_clone.vibrate(VibrateCommand::Speed(vibrate_value)).await;
                      }.bind_hub(sentry::Hub::current().clone()));
                    }
                    ui.memory().data.insert_temp(vibrate_id, vibrate_value);
                  }
                  if device.allowed_messages.contains_key(&ButtplugCurrentSpecDeviceMessageType::RotateCmd) {
                    let rotate_id = ui.make_persistent_id(format!("DevicesPanel::Rotate::{}", device.index()));
                    let mut rotate_value = ui.memory().data.get_temp_mut_or_default::<f64>(rotate_id).clone();
                    if ui.add(egui::Slider::new::<f64>(&mut rotate_value, -1.0..=1.0).text("Rotation Level")).changed() {
                      let device_clone = device.clone();
                      tokio::spawn(async move {                        
                        device_clone.rotate(RotateCommand::Rotate(rotate_value.abs(), rotate_value < 0f64)).await;
                      }.bind_hub(sentry::Hub::current().clone()));
                    }
                    ui.memory().data.insert_temp(rotate_id, rotate_value);
                  }
                });
              }
            });
          } else {
            if ui.button("Connect To Server").clicked() {
              let client = Arc::new(ButtplugClient::new("Intiface Desktop Device Tab Client"));
              let client_clone = client.clone();
              ui.memory().data.insert_temp(id, client);
              
              tokio::spawn(async move {
                let connector = ButtplugRemoteClientConnector::<ButtplugWebsocketClientTransport, ButtplugClientJSONSerializer>::new(ButtplugWebsocketClientTransport::new_insecure_connector("ws://localhost:12345"));
                let mut stream = client_clone.event_stream();
                client_clone.connect(connector).await;
                while let Some(event) = stream.next().await {
                  match event {
                    ButtplugClientEvent::ServerDisconnect => break,
                    msg => info!("Client got event: {:?}", msg)
                  }
                }
              }.bind_hub(sentry::Hub::current().clone()));
            }
          }
        });
      } else {
        let id = ui.make_persistent_id("DevicesPanel::ButtplugClient");
        let maybe_client = ui.memory().data.get_temp::<Arc<ButtplugClient>>(id).clone();        
        if maybe_client.is_some() {
          ui.memory().data.remove::<Arc<ButtplugClient>>(id);
        }      
      }
    });
  }
}
