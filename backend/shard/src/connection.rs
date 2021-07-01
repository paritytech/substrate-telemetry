use futures::channel::mpsc;
use futures::{Sink, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Clone, Debug)]
pub enum Message<Out> {
    Connected,
    Disconnected,
    Data(Out),
}

/// Connect to a websocket server, retrying the connection if we're disconnected.
/// - Sends messages when disconnected, reconnected or data received from the connection.
/// - Returns a channel that allows you to send messages to the connection.
/// - Messages all encoded/decoded from bincode.
pub async fn create_ws_connection<In, Out, S, E>(
    mut tx_to_external: S,
    telemetry_uri: http::Uri,
) -> mpsc::Sender<In>
where
    S: Sink<Message<Out>, Error = E> + Unpin + Send + Clone + 'static,
    E: std::fmt::Debug + std::fmt::Display + Send + 'static,
    In: serde::Serialize + Send + 'static,
    Out: serde::de::DeserializeOwned + Send + 'static,
{
    // Set up a proxy channel to relay messages to the telemetry core, and return one end of it.
    // Once a connection to the backend is established, we pass messages along to it. If the connection
    // fails, we
    let (tx_to_connection_proxy, mut rx_from_external_proxy) = mpsc::channel(10);
    tokio::spawn(async move {
        let mut connected = false;

        loop {
            // Throw away any pending messages from the incoming channel so that it
            // doesn't get blocked up while we're looping and waiting for a reconnection.
            while let Ok(Some(_)) = rx_from_external_proxy.try_next() {}

            // The connection will pass messages back to this.
            let tx_from_connection = tx_to_external.clone();

            // Attempt to reconnect.
            match create_ws_connection_no_retry(tx_from_connection, telemetry_uri.clone()).await {
                Ok(mut tx_to_connection) => {
                    connected = true;

                    // Inform the handler loop that we've reconnected.
                    tx_to_external
                        .send(Message::Connected)
                        .await
                        .expect("must be able to send reconnect msg");

                    // Start forwarding messages on to the backend.
                    while let Some(msg) = rx_from_external_proxy.next().await {
                        if let Err(e) = tx_to_connection.send(msg).await {
                            // Issue forwarding a message to the telemetry core?
                            // Give up and try to reconnect on the next loop iteration.
                            log::error!(
                                "Error sending message to websocker server (will reconnect): {}",
                                e
                            );
                            break;
                        }
                    }
                }
                Err(e) => {
                    // Issue connecting? Wait and try again on the next loop iteration.
                    log::error!(
                        "Error connecting to websocker server (will reconnect): {}",
                        e
                    );
                }
            };

            // Tell the aggregator that we're disconnected so that, if we like, we can discard
            // messages without doing any futher processing on them.
            if connected {
                connected = false;
                let _ = tx_to_external.send(Message::Disconnected).await;
            }

            // Wait a little before trying to reconnect.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    tx_to_connection_proxy
}

/// This spawns a connection to a websocket server, serializing/deserialziing
/// from bincode as messages are sent or received.
async fn create_ws_connection_no_retry<In, Out, S, E>(
    mut tx_to_external: S,
    telemetry_uri: http::Uri,
) -> anyhow::Result<mpsc::Sender<In>>
where
    S: Sink<Message<Out>, Error = E> + Unpin + Send + 'static,
    E: std::fmt::Debug + std::fmt::Display,
    In: serde::Serialize + Send + 'static,
    Out: serde::de::DeserializeOwned + Send + 'static,
{
    use bincode::Options;
    use soketto::handshake::{Client, ServerResponse};

    let host = telemetry_uri.host().unwrap_or("127.0.0.1");
    let port = telemetry_uri.port_u16().unwrap_or(8000);
    let path = telemetry_uri.path();

    let socket = TcpStream::connect((host, port)).await?;
    socket.set_nodelay(true).unwrap();

    // Open a websocket connection with the relemetry core:
    let mut client = Client::new(socket.compat(), host, &path);
    let (mut ws_to_connection, mut ws_from_connection) = match client.handshake().await? {
        ServerResponse::Accepted { .. } => client.into_builder().finish(),
        ServerResponse::Redirect { status_code, .. } | ServerResponse::Rejected { status_code } => {
            return Err(anyhow::anyhow!(
                "Failed to connect to {}{}, status code: {}",
                host,
                path,
                status_code
            ));
        }
    };

    // This task reads data sent from the telemetry core and
    // forwards it on to our aggregator loop:
    tokio::spawn(async move {
        let mut data = Vec::with_capacity(128);
        loop {
            // Clear the buffer and wait for the next message to arrive:
            data.clear();
            if let Err(e) = ws_from_connection.receive_data(&mut data).await {
                // Couldn't receive data may mean all senders are gone, so log
                // the error and shut this down:
                log::error!(
                    "Shutting down websocket connection: Failed to receive data: {}",
                    e
                );
                return;
            }

            // Attempt to deserialize, and send to our handler loop:
            match bincode::options().deserialize(&data) {
                Ok(msg) => {
                    if let Err(e) = tx_to_external.send(Message::Data(msg)).await {
                        // Failure to send to our loop likely means it's hit an
                        // issue and shut down, so bail on this loop as well:
                        log::error!(
                            "Shutting down websocket connection: Failed to send data out: {}",
                            e
                        );
                        return;
                    }
                }
                Err(err) => {
                    // Log the error but otherwise ignore it and keep running:
                    log::warn!("Failed to decode message from Backend Core: {:?}", err);
                }
            }
        }
    });

    // This task receives messages from the aggregator,
    // encodes them and sends them to the telemetry core:
    let (tx_to_connection, mut rx_from_aggregator) = mpsc::channel(10);
    tokio::spawn(async move {
        while let Some(msg) = rx_from_aggregator.next().await {
            let bytes = bincode::options()
                .serialize(&msg)
                .expect("must be able to serialize msg");

            // Any errors sending the message leads to this task ending, which should cascade to
            // the entire connection being ended.
            if let Err(e) = ws_to_connection.send_binary_mut(bytes).await {
                log::error!(
                    "Shutting down websocket connection: Failed to send data in: {}",
                    e
                );
                return;
            }
            if let Err(e) = ws_to_connection.flush().await {
                log::error!(
                    "Shutting down websocket connection: Failed to flush data: {}",
                    e
                );
                return;
            }
        }
    });

    // We return a channel that you can send messages down in order to have
    // them sent to the telemetry core:
    Ok(tx_to_connection)
}
