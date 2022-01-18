use crate::core::AppCore;
use buttplug::{
  client::{ButtplugClient, ButtplugClientEvent, RotateCommand, VibrateCommand, LinearCommand},
  connector::{ButtplugRemoteClientConnector, ButtplugWebsocketClientTransport},
  core::messages::{
    serializer::ButtplugClientJSONSerializer, ButtplugCurrentSpecDeviceMessageType,
  },
};
use eframe::egui;
use futures::StreamExt;
use sentry::SentryFutureExt;
use std::{collections::HashMap, sync::Arc};
use tracing::info;

#[derive(Default)]
pub struct DeviceTestPanel {
  client: Option<Arc<ButtplugClient>>,
  // Store the "all" value and each individual actuator value in a tuple.
  vibrate_values: HashMap<u32, (f64, Vec<f64>)>,
  rotate_values: HashMap<u32, (f64, Vec<f64>)>,
  linear_values: HashMap<u32, ((u32, f64), Vec<(u32, f64)>)>,
}

impl DeviceTestPanel {
  pub fn update(&mut self, core: &mut AppCore, ui: &mut egui::Ui) {
    if core.process_manager.is_running()
      && (core.process_manager.client_name().is_none() || self.client.is_some())
    {
      if self.client.is_some() {
        let client = self
          .client
          .as_ref()
          .expect("Already checked existence")
          .clone();
        
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
            ui.collapsing(format!("{}", device.name), |ui| 
            {
              if device
                .allowed_messages
                .contains_key(&ButtplugCurrentSpecDeviceMessageType::VibrateCmd)
              {
                let vibrate_value = self.vibrate_values.entry(device.index()).or_insert((
                  0f64,
                  vec![
                    0f64;
                    device
                      .allowed_messages
                      .get(&ButtplugCurrentSpecDeviceMessageType::VibrateCmd)
                      .expect("Already tested.")
                      .feature_count
                      .unwrap_or(0) as usize
                  ],
                ));
                let mut update = false;
                if vibrate_value.1.len() > 1 {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(&mut vibrate_value.0, 0.0..=1.0)
                        .text("Vibration Level (All Vibrators)"),
                    )
                    .changed()
                  {
                    // .count() used to consume iterator.
                    vibrate_value.1.iter_mut().map(|v| *v = vibrate_value.0).count();
                    update = true;
                  }
                }
                
                for (index, speed) in vibrate_value.1.iter_mut().enumerate() {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(speed, 0.0..=1.0)
                        .text(format!("Vibrator {} Level", index + 1)),
                    )
                    .changed()
                  {
                    update = true;
                  }                  
                }
                if update {
                  let device_clone = device.clone();
                  let level = vibrate_value.1.clone();
                  tokio::spawn(
                    async move {
                      device_clone.vibrate(VibrateCommand::SpeedVec(level)).await;
                    }
                    .bind_hub(sentry::Hub::current().clone()),
                  );
                }
              }

            
              if device
                .allowed_messages
                .contains_key(&ButtplugCurrentSpecDeviceMessageType::RotateCmd)
              {
                let rotate_value = self.rotate_values.entry(device.index()).or_insert((
                  0f64,
                  vec![
                    0f64;
                    device
                      .allowed_messages
                      .get(&ButtplugCurrentSpecDeviceMessageType::RotateCmd)
                      .expect("Already tested.")
                      .feature_count
                      .unwrap_or(0) as usize
                  ],
                ));
                let mut update = false;
                if rotate_value.1.len() > 1 {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(&mut rotate_value.0, -1.0..=1.0)
                        .text("Rotation Level (All Rotators)"),
                    )
                    .changed()
                  {
                    // .count() used to consume iterator.
                    rotate_value.1.iter_mut().map(|v| *v = rotate_value.0).count();
                    update = true;
                  }
                }                
                for (index, speed) in rotate_value.1.iter_mut().enumerate() {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(speed, -1.0..=1.0)
                        .text(format!("Rotator {} Level", index + 1)),
                    )
                    .changed()
                  {
                    update = true;
                  }                  
                }
                if update {
                  let device_clone = device.clone();
                  let level = rotate_value.1.clone();
                  tokio::spawn(
                    async move {
                      device_clone.rotate(RotateCommand::RotateVec(level.iter().map(|v| (v.abs(), *v >= 0f64)).collect())).await;
                    }
                    .bind_hub(sentry::Hub::current().clone()),
                  );
                }
              }

              if device
                .allowed_messages
                .contains_key(&ButtplugCurrentSpecDeviceMessageType::LinearCmd)
              {
                let linear_value = self.linear_values.entry(device.index()).or_insert((
                  (0u32, 0f64),
                  vec![
                    (0u32, 0f64);
                    device
                      .allowed_messages
                      .get(&ButtplugCurrentSpecDeviceMessageType::LinearCmd)
                      .expect("Already tested.")
                      .feature_count
                      .unwrap_or(0) as usize
                  ],
                ));
                let mut update = false;
                if linear_value.1.len() > 1 {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(&mut linear_value.0.1, 0.0..=1.0)
                        .text("Position (All Linear)"),
                    )
                    .changed()
                  {
                    update = true;
                    linear_value.1.iter_mut().map(|v| *v = linear_value.0).count();
                  }
                  if ui
                  .add(
                    egui::Slider::new::<u32>(&mut linear_value.0.0, 0..=3000)
                      .text("Move Time (All Linear)"),
                  )
                  .changed()
                  {
                    update = true;
                    linear_value.1.iter_mut().map(|v| *v = linear_value.0).count();
                  }
                }
                let mut update = false;
                for (index, speed) in linear_value.1.iter_mut().enumerate() {
                  if ui
                    .add(
                      egui::Slider::new::<f64>(&mut speed.1, 0.0..=1.0)
                        .text(format!("Linear {} Position", index + 1)),
                    )
                    .changed()
                  {
                    update = true;
                  }
                  if ui
                  .add(
                    egui::Slider::new::<u32>(&mut speed.0, 0..=3000)
                      .text(format!("Linear {} Move Time", index + 1)),
                  )
                  .changed()
                  {
                    update = true;
                  }             
                }
                if ui.button("Move").clicked() {
                  let device_clone = device.clone();
                  let level = linear_value.1.clone();
                  tokio::spawn(
                    async move {
                      device_clone.linear(LinearCommand::LinearVec(level)).await;
                    }
                    .bind_hub(sentry::Hub::current().clone()),
                  );
                }
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
