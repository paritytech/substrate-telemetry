// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use bincode::Options;
use common::ws_client;
use futures::StreamExt;

#[derive(Clone, Debug)]
pub enum Message<Out> {
    Connected,
    Disconnected,
    Data(Out),
}

/// Connect to the telemetry core, retrying the connection if we're disconnected.
/// - Sends `Message::Connected` and `Message::Disconnected` when the connection goes up/down.
/// - Returns a channel that allows you to send messages to the connection.
/// - Messages are all encoded/decoded to/from bincode, and so need to support being (de)serialized from
///   a non self-describing encoding.
///
/// Note: have a look at [`common::internal_messages`] to see the different message types exchanged
/// between aggregator and core.
pub async fn create_ws_connection_to_core<In, Out>(
    telemetry_uri: http::Uri,
) -> (flume::Sender<In>, flume::Receiver<Message<Out>>)
where
    In: serde::Serialize + Send + 'static,
    Out: serde::de::DeserializeOwned + Send + 'static,
{
    let (tx_in, rx_in) = flume::bounded::<In>(10);
    let (tx_out, rx_out) = flume::bounded(10);

    let mut is_connected = false;

    tokio::spawn(async move {
        loop {
            // Throw away any pending messages from the incoming channel so that it
            // doesn't get filled up and begin blocking while we're looping and waiting
            // for a reconnection.
            while let Ok(_) = rx_in.try_recv() {}

            // Try to connect. If connection established, we serialize and forward messages
            // to/from the core. If the external channels break, we end for good. If the internal
            // channels break, we loop around and try connecting again.
            match ws_client::connect(&telemetry_uri).await {
                Ok(connection) => {
                    let (tx_to_core, mut rx_from_core) = connection.into_channels();
                    is_connected = true;
                    let tx_out = tx_out.clone();

                    if let Err(e) = tx_out.send_async(Message::Connected).await {
                        // If receiving end is closed, bail now.
                        log::warn!("Aggregator is no longer receiving messages from core; disconnecting (permanently): {}", e);
                        return;
                    }

                    // Loop, forwarding messages to and from the core until something goes wrong.
                    loop {
                        tokio::select! {
                            msg = rx_from_core.next() => {
                                let msg = match msg {
                                    Some(Ok(msg)) => msg,
                                    // No more messages from core? core WS is disconnected.
                                    _ => {
                                        log::warn!("No more messages from core: shutting down connection (will reconnect)");
                                        break
                                    }
                                };

                                let bytes = match msg {
                                    ws_client::RecvMessage::Binary(bytes) => bytes,
                                    ws_client::RecvMessage::Text(s) => s.into_bytes()
                                };
                                let msg = bincode::options()
                                    .deserialize(&bytes)
                                    .expect("internal messages must be deserializable");

                                if let Err(e) = tx_out.send_async(Message::Data(msg)).await {
                                    log::error!("Aggregator is no longer receiving messages from core; disconnecting (permanently): {}", e);
                                    return;
                                }
                            },
                            msg = rx_in.recv_async() => {
                                let msg = match msg {
                                    Ok(msg) => msg,
                                    Err(flume::RecvError::Disconnected) => {
                                        log::error!("Aggregator is no longer sending messages to core; disconnecting (permanently)");
                                        return
                                    }
                                };

                                let bytes = bincode::options()
                                    .serialize(&msg)
                                    .expect("internal messages must be serializable");
                                let ws_msg = ws_client::SentMessage::Binary(bytes);

                                if let Err(e) = tx_to_core.unbounded_send(ws_msg) {
                                    log::warn!("Unable to send message to core; shutting down connection (will reconnect): {}", e);
                                    break;
                                }
                            }
                        };
                    }
                }
                Err(connect_err) => {
                    // Issue connecting? Wait and try again on the next loop iteration.
                    log::error!(
                        "Error connecting to websocker server (will reconnect): {}",
                        connect_err
                    );
                }
            }

            if is_connected {
                is_connected = false;
                if let Err(e) = tx_out.send_async(Message::Disconnected).await {
                    log::error!("Aggregator is no longer receiving messages from core; disconnecting (permanently): {}", e);
                    return;
                }
            }

            // Wait a little before we try to connect again.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    (tx_in, rx_out)
}
