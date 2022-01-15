use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use buttplug::{
  server::comm_managers::websocket_server::websocket_server_comm_manager::WebsocketServerDeviceCommManagerInitInfo,
  core::messages::{VibrateCmd, RotateCmd, LinearCmd},
};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub enum SimulatedEvent {
  VibrateCmd(VibrateCmd),
  RotateCmd(RotateCmd),
  LinearCmd(LinearCmd),
  Disconnect,
}

async fn device_simulator_loop(server_address: &str, identifier: &str) {
  let (mut writer, mut reader) = match connect_async(server_address).await {
    Ok((stream, _)) => {
      stream.split()
    }
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
    version: 1 
  }).expect("This will always convert correctly");

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
          return;
        }
        match response.expect("Already checked for none.") {
          Ok(msg) => match msg {
            Message::Text(t) => {
            }
            Message::Binary(binary_msg) => {
              let msg_str = std::str::from_utf8(&binary_msg).unwrap();
              
              info!(msg_str);
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
      /*
      _ = disconnect_notifier.notified().fuse() => {
        // If we can't close, just print the error to the logs but
        // still break out of the loop.
        //
        // TODO Emit a full error here that should bubble up to the client.
        info!("Websocket requested to disconnect.");
        writer.close().await.unwrap_or_else(|err| error!("{}", err));
        return;
      }
      */
    }
  }
}

pub struct SimulatedDevice {
  identifier: String,
  connected: Arc<AtomicBool>
}

impl SimulatedDevice {
  pub fn new(identifier: &str) -> Self {
    Self {
      identifier: identifier.to_owned(),
      connected: Arc::new(AtomicBool::new(false))
    }
  }

  pub fn connect(&self, server_url: &str) {
    let server_url = server_url.to_owned();
    let identifier = self.identifier.clone();
    tokio::spawn(async move {
      device_simulator_loop(&server_url, &identifier).await;
    });
  }

  pub fn disconnect(&self) {}

  pub fn connected(&self) {}
}
