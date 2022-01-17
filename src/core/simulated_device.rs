use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use buttplug::{
  core::messages::{ButtplugCurrentSpecClientMessage, ButtplugDeviceMessageType},
  device::configuration_manager::ProtocolAttributes,
  server::comm_managers::websocket_server::websocket_server_comm_manager::WebsocketServerDeviceCommManagerInitInfo,
};
use futures::{SinkExt, StreamExt};
use std::{sync::{
  atomic::{AtomicI8, AtomicU8, AtomicU32, AtomicU64, Ordering},
  Arc,
}, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::Notify;

fn epoch_time() -> u64 {
  let start = SystemTime::now();
  start
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_millis() as u64
}

// We hardcode all of our actuators to have 100 steps, so just store this as a U8.
#[derive(Debug)]
pub struct SimulatedActuators {
  pub vibrators: Vec<AtomicU8>,
  pub rotators: Vec<AtomicI8>,
  pub linear: Vec<(AtomicU64, AtomicU8, AtomicU32)>
}

impl SimulatedActuators {
  pub fn new(num_vibrators: u32, num_rotators: u32, num_linear: u32) -> Self {
    let mut vibrators = Vec::new();
    for _ in 0..num_vibrators {
      vibrators.push(AtomicU8::new(0));
    }

    let mut rotators = Vec::new();
    for _ in 0..num_rotators {
      rotators.push(AtomicI8::new(0));
    }

    let mut linear = Vec::new();
    for _ in 0..num_linear {
      linear.push((AtomicU64::new(0), AtomicU8::new(0), AtomicU32::new(0)));
    }

    Self {
      vibrators,
      rotators,
      linear,
    }
  }

  pub fn stop(&self) {
    self
      .vibrators
      .iter()
      .for_each(|v| v.store(0, Ordering::SeqCst));
    self
      .rotators
      .iter()
      .for_each(|r| r.store(0, Ordering::SeqCst));
  }
}

async fn device_simulator_loop(
  server_address: &str,
  identifier: &str,
  actuators: Arc<SimulatedActuators>,
  disconnect_notifier: Arc<Notify>,
) {
  let (mut writer, mut reader) = match connect_async(server_address).await {
    Ok((stream, _)) => stream.split(),
    Err(websocket_error) => {
      error!("Cannot connect simulated device: {}", websocket_error);
      return;
    }
  };

  let address = super::util::random_string();

  // If we've gotten this far, we need to construct a init info packet to identify ourselves to the server.
  let info = serde_json::to_string(&WebsocketServerDeviceCommManagerInitInfo {
    identifier: identifier.to_owned(),
    address,
    version: 1,
  })
  .expect("This will always convert correctly");

  if let Err(e) = writer.send(Message::Text(info)).await {
    error!("Cannot send simulated device info packet: {}", e)
  }

  loop {
    tokio::select! {
      /*
      msg = outgoing_receiver.recv().fuse() => {
        if let Some(msg) = msg {
          let out_msg = match msg {
            ButtplugSerializedMessage::Text(text) => Message::Text(text),
            ButtplugSerializedMessage::Binary(bin) => Message::Binary(bin),
          };
          // TODO see what happens when we try to send to a remote that's closed connection.
          writer.send(out_msg).await.expect("This should never fail?");
        } else {
          info!("Connector holding websocket dropped, returning");
          writer.close().await.unwrap_or_else(|err| error!("{}", err));
          return;
        }
      },
      */
      response = reader.next() => {
        trace!("Websocket receiving: {:?}", response);
        if response.is_none() {
          info!("Connector holding websocket dropped, returning");
          writer.close().await.unwrap_or_else(|err| error!("{}", err));
          break;
        }
        match response.expect("Already checked for none.") {
          Ok(msg) => match msg {
            Message::Text(t) => {
            }
            Message::Binary(binary_msg) => {
              let msg_str = std::str::from_utf8(&binary_msg).unwrap();
              // By the time a message gets here, the server will have already upcast it into the
              // current spec version.
              let device_msg = serde_json::from_str::<ButtplugCurrentSpecClientMessage>(msg_str).expect("This should always work with our server");
              match device_msg {
                ButtplugCurrentSpecClientMessage::VibrateCmd(msg) => {
                  for command in msg.speeds() {
                    actuators.vibrators[command.index() as usize].store((command.speed() * 100f64) as u8, Ordering::SeqCst);
                  }
                },
                ButtplugCurrentSpecClientMessage::RotateCmd(msg) => {
                  for command in msg.rotations {
                    let clockwise = if command.clockwise() { 1f64 } else { -1f64 };
                    actuators.rotators[command.index() as usize].store((command.speed() * 100f64 * clockwise) as i8, Ordering::SeqCst);
                  }
                },
                ButtplugCurrentSpecClientMessage::LinearCmd(msg) => {
                  let time = epoch_time();
                  for command in msg.vectors() {
                    actuators.linear[command.index() as usize].0.store(time, Ordering::SeqCst);
                    actuators.linear[command.index() as usize].1.store((command.position() * 100f64) as u8, Ordering::SeqCst);
                    actuators.linear[command.index() as usize].2.store(command.duration(), Ordering::SeqCst);
                  }
                },
                ButtplugCurrentSpecClientMessage::StopDeviceCmd(msg) => actuators.stop(),
                msg => { warn!("Unhandled message: {:?}", msg) }
              }
              trace!(msg_str);
            }
            Message::Ping(data) => {
              writer.send(Message::Pong(data)).await.expect("This should never fail?");
            }
            Message::Pong(_) => {}
            Message::Close(_) => {
              info!("Websocket has requested close.");
              return;
            }
          },
          Err(err) => {
            error!(
              "Error in websocket client loop (assuming disconnect): {}",
              err
            );
            break;
          }
        }
      }
      _ = disconnect_notifier.notified() => {
        // If we can't close, just print the error to the logs but
        // still break out of the loop.
        //
        // TODO Emit a full error here that should bubble up to the client.
        info!("Websocket requested to disconnect.");
        writer.close().await.unwrap_or_else(|err| error!("{}", err));
        return;
      }
    }
  }
}

#[derive(Default, Debug, Clone)]
struct LinearStatus {
  pub last_change_time: u64,
  pub last_change_duration: u32,
  pub last_change_position: u8,
  pub last_goal_position: u8,
  pub current_position: u8,
}

pub struct SimulatedDevice {
  identifier: String,
  actuators: Arc<SimulatedActuators>,
  linear_status: Vec<LinearStatus>,
  disconnect_notifier: Arc<Notify>,
}

impl SimulatedDevice {
  pub fn new(device: &ProtocolAttributes) -> Self {
    let identifier = device
      .identifier()
      .as_ref()
      .expect("We had to have an identifier to get here")[0]
      .clone();
    let messages = device
      .messages()
      .as_ref()
      .expect("Should have message defined");
    let num_vibrators =
      if let Some(attributes) = messages.get(&ButtplugDeviceMessageType::VibrateCmd) {
        attributes.feature_count.as_ref().unwrap_or(&0).clone()
      } else {
        0
      };
    let num_rotators = if let Some(attributes) = messages.get(&ButtplugDeviceMessageType::RotateCmd)
    {
      attributes.feature_count.as_ref().unwrap_or(&0).clone()
    } else {
      0
    };
    let num_linear = if let Some(attributes) = messages.get(&ButtplugDeviceMessageType::LinearCmd) {
      attributes.feature_count.as_ref().unwrap_or(&0).clone()
    } else {
      0
    };

    Self {
      identifier,
      disconnect_notifier: Arc::new(Notify::new()),
      linear_status: vec![LinearStatus::default(); num_linear as usize],
      actuators: Arc::new(SimulatedActuators::new(
        num_vibrators,
        num_rotators,
        num_linear,
      )),
    }
  }

  pub fn connect(&self, server_url: &str) {
    let server_url = server_url.to_owned();
    let identifier = self.identifier.clone();
    let actuators = self.actuators.clone();
    let disconnect_notifier = self.disconnect_notifier.clone();
    tokio::spawn(async move {
      device_simulator_loop(&server_url, &identifier, actuators, disconnect_notifier).await;
    });
  }

  pub fn disconnect(&self) {
    self.disconnect_notifier.notify_one();
  }

  pub fn update_linear(&mut self) -> Vec<u8> {
    for (index, linear_status) in self.linear_status.iter_mut().enumerate() {
      let last_change_time = self.actuators.linear[index].0.load(Ordering::SeqCst);
      if last_change_time != linear_status.last_change_time {
        linear_status.last_change_time = last_change_time;
        linear_status.last_goal_position = linear_status.last_change_position;
        linear_status.last_change_position = self.actuators.linear[index].1.load(Ordering::SeqCst);
        linear_status.last_change_duration = self.actuators.linear[index].2.load(Ordering::SeqCst);
      }
      if linear_status.current_position != linear_status.last_change_position {
        linear_status.current_position = (linear_status.last_goal_position as i8 + ((linear_status.last_change_position as i32 - linear_status.last_goal_position as i32) as f64 * ((epoch_time() - last_change_time) as f64 / linear_status.last_change_duration as f64).clamp(0f64, 1f64)) as i8).clamp(0, 100) as u8;
      }
    }
    self.linear_status.iter().map(|x| x.current_position).collect()
  }

  pub fn actuators(&self) -> Arc<SimulatedActuators> {
    self.actuators.clone()
  }
}
