use crate::core::AppCore;
use buttplug::{
  client::{ButtplugClient, ButtplugClientEvent, RotateCommand, VibrateCommand},
  connector::{ButtplugRemoteClientConnector, ButtplugWebsocketClientTransport},
  core::messages::{
    serializer::ButtplugClientJSONSerializer, ButtplugCurrentSpecDeviceMessageType,
  },
};
use eframe::egui;
use futures::StreamExt;
use sentry::SentryFutureExt;
use std::sync::Arc;
use tracing::info;

#[derive(Default)]
pub struct DeviceTestPanel {
  client: Option<Arc<ButtplugClient>>
}

impl DeviceTestPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    if core.process_manager.is_running() && (core.process_manager.client_name().is_none() || self.client.is_some()) {
      if self.client.is_some() {
        let client = self.client.as_ref().expect("Already checked existence").clone();
        if !client.connected() {
          ui.label("Connecting...");
          return;
        }
        ui.vertical(|ui| {
          ui.horizontal(|ui| {
            if ui.button("Scan for Devices").clicked() {
              let client_clone = client.clone();
              tokio::spawn(
                async move {
                  client_clone.start_scanning().await;
                }
                .bind_hub(sentry::Hub::current().clone()),
              );
            }
            if ui.button("Disconnect").clicked() {
              let client_clone = client.clone();
              tokio::spawn(
                async move {
                  client_clone.disconnect().await;
                }
                .bind_hub(sentry::Hub::current().clone()),
              );
              let _ = self.client.take();
            }
          });
          for device in client.devices() {
            ui.collapsing(format!("{}", device.name), |ui| {
              if device
                .allowed_messages
                .contains_key(&ButtplugCurrentSpecDeviceMessageType::VibrateCmd)
              {
                let vibrate_id =
                  ui.make_persistent_id(format!("DevicesPanel::Vibrate::{}", device.index()));
                let mut vibrate_value = ui
                  .memory()
                  .data
                  .get_temp_mut_or_default::<f64>(vibrate_id)
                  .clone();
                if ui
                  .add(
                    egui::Slider::new::<f64>(&mut vibrate_value, 0.0..=1.0).text("Vibration Level"),
                  )
                  .changed()
                {
                  let device_clone = device.clone();
                  tokio::spawn(
                    async move {
                      device_clone
                        .vibrate(VibrateCommand::Speed(vibrate_value))
                        .await;
                    }
                    .bind_hub(sentry::Hub::current().clone()),
                  );
                }
                ui.memory().data.insert_temp(vibrate_id, vibrate_value);
              }
              if device
                .allowed_messages
                .contains_key(&ButtplugCurrentSpecDeviceMessageType::RotateCmd)
              {
                let rotate_id =
                  ui.make_persistent_id(format!("DevicesPanel::Rotate::{}", device.index()));
                let mut rotate_value = ui
                  .memory()
                  .data
                  .get_temp_mut_or_default::<f64>(rotate_id)
                  .clone();
                if ui
                  .add(
                    egui::Slider::new::<f64>(&mut rotate_value, -1.0..=1.0).text("Rotation Level"),
                  )
                  .changed()
                {
                  let device_clone = device.clone();
                  tokio::spawn(
                    async move {
                      device_clone
                        .rotate(RotateCommand::Rotate(
                          rotate_value.abs(),
                          rotate_value < 0f64,
                        ))
                        .await;
                    }
                    .bind_hub(sentry::Hub::current().clone()),
                  );
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
          self.client = Some(client);

          tokio::spawn(
            async move {
              let connector = ButtplugRemoteClientConnector::<
                ButtplugWebsocketClientTransport,
                ButtplugClientJSONSerializer,
              >::new(
                ButtplugWebsocketClientTransport::new_insecure_connector("ws://localhost:12345"),
              );
              let mut stream = client_clone.event_stream();
              client_clone.connect(connector).await;
              while let Some(event) = stream.next().await {
                match event {
                  ButtplugClientEvent::ServerDisconnect => break,
                  msg => info!("Client got event: {:?}", msg),
                }
              }
            }
            .bind_hub(sentry::Hub::current().clone()),
          );
        }
      }
    } else {
      if core.process_manager.client_name().is_some() {
        ui.label("Cannot connect Device Test panel while another client application is connected.");
      } else {
        ui.label("Intiface Server must be running in order to use Device Test panel.");
      }
      if self.client.is_some() {
        let _ = self.client.take();
      }
    }
  }
}
