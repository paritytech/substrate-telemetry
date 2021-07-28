use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use common::ws_client;
use bincode::Options;

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
) -> (mpsc::Sender<In>, mpsc::Receiver<Message<Out>>)
where
    In: serde::Serialize + Send + 'static,
    Out: serde::de::DeserializeOwned + Send + 'static,
{
    let (tx_in, mut rx_in) = mpsc::channel(10);
    let (mut tx_out, rx_out) = mpsc::channel(10);

    let mut is_connected = false;

    tokio::spawn(async move {
        loop {
            // Throw away any pending messages from the incoming channel so that it
            // doesn't get filled up and begin blocking while we're looping and waiting
            // for a reconnection.
            while let Ok(Some(_)) = rx_in.try_next() {}

            // Try to connect. If connection established, we serialize and forward messages
            // to/from the core. If the external channels break, we end for good. If the internal
            // channels break, we loop around and try connecting again.
            match ws_client::connect(&telemetry_uri).await {
                Ok((tx_to_core, mut rx_from_core)) => {
                    is_connected = true;
                    let mut tx_out = tx_out.clone();

                    if let Err(e) = tx_out.send(Message::Connected).await {
                        // If receiving end is closed, bail now.
                        log::warn!("Aggregator is no longer receiving messages from core; disconnecting (permanently): {}", e);
                        return
                    }

                    // Loop, forwarding messages to and from the core until something goes wrong.
                    loop {
                        tokio::select! {
                            msg = rx_from_core.next() => {
                                let msg = match msg {
                                    Some(msg) => msg,
                                    // No more messages from core? core WS is disconnected.
                                    None => {
                                        log::warn!("No more messages from core: shutting down connection (will reconnect)");
                                        break
                                    }
                                };

                                let bytes = match msg {
                                    Ok(ws_client::RecvMessage::Binary(bytes)) => bytes,
                                    Ok(ws_client::RecvMessage::Text(s)) => s.into_bytes(),
                                    Err(e) => {
                                        log::warn!("Unable to receive message from core: shutting down connection (will reconnect): {}", e);
                                        break;
                                    }
                                };
                                let msg = bincode::options()
                                    .deserialize(&bytes)
                                    .expect("internal messages must be deserializable");

                                if let Err(e) = tx_out.send(Message::Data(msg)).await {
                                    log::error!("Aggregator is no longer receiving messages from core; disconnecting (permanently): {}", e);
                                    return;
                                }
                            },
                            msg = rx_in.next() => {
                                let msg = match msg {
                                    Some(msg) => msg,
                                    None => {
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
                },
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
                if let Err(e) = tx_out.send(Message::Disconnected).await {
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