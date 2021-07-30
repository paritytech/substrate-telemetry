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
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use soketto::handshake::{Client, ServerResponse};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::{
    receiver::{Receiver, RecvMessage},
    sender::{Sender, SentMessage, SentMessageInternal},
};

/// The send side of a Soketto WebSocket connection
pub type RawSender = soketto::connection::Sender<tokio_util::compat::Compat<tokio::net::TcpStream>>;

/// The receive side of a Soketto WebSocket connection
pub type RawReceiver =
    soketto::connection::Receiver<tokio_util::compat::Compat<tokio::net::TcpStream>>;

/// A websocket connection. From this, we can either expose the raw connection
/// or expose a cancel-safe interface to it.
pub struct Connection {
    tx: soketto::connection::Sender<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    rx: soketto::connection::Receiver<tokio_util::compat::Compat<tokio::net::TcpStream>>,
}

impl Connection {
    /// Get hold of the raw send/receive interface for this connection.
    /// These are not cancel-safe, but can be more performant than the
    /// cancel-safe channel based interface.
    pub fn into_raw(self) -> (RawSender, RawReceiver) {
        (self.tx, self.rx)
    }

    /// Get hold of send and receive channels for this connection.
    /// These channels are cancel-safe.
    ///
    /// This spawns a couple of tasks for pulling/pushing messages onto the
    /// connection, and so messages will be pushed onto the receiving channel
    /// without any further polling. use [`Connection::into_raw`] if you need
    /// more precise control over when messages are pulled from the socket.
    ///
    /// # Panics
    ///
    /// This will panic if not called within the context of a tokio runtime.
    ///
    pub fn into_channels(self) -> (Sender, Receiver) {
        let (mut ws_to_connection, mut ws_from_connection) = (self.tx, self.rx);

        // Receive messages from the socket and post them out:
        let (mut tx_to_external, rx_from_ws) = mpsc::unbounded();
        let (tx_has_closed, mut rx_has_closed) = futures::channel::oneshot::channel();
        tokio::spawn(async move {
            let mut data = Vec::with_capacity(128);
            loop {
                // Clear the buffer and wait for the next message to arrive:
                data.clear();
                let message_data = match ws_from_connection.receive_data(&mut data).await {
                    Err(e) => {
                        // Couldn't receive data means some issue with the connection. Log
                        // the error, and close the other half of the connection too,
                        // so the associated channels close gracefully.
                        log::error!(
                            "Shutting down websocket connection: Failed to receive data: {}",
                            e
                        );
                        let _ = tx_has_closed.send(());
                        break;
                    }
                    Ok(data) => data,
                };

                let msg = match message_data {
                    soketto::Data::Binary(_) => Ok(RecvMessage::Binary(data)),
                    soketto::Data::Text(_) => String::from_utf8(data)
                        .map(|s| RecvMessage::Text(s))
                        .map_err(|e| e.into()),
                };

                data = Vec::with_capacity(128);

                if let Err(e) = tx_to_external.send(msg).await {
                    // Failure to send likely means that the recv has been dropped,
                    // so let's drop this loop too. An issue with the channel doesn't
                    // mean that our socket connection has failed though, so we make no
                    // attempt to close the other half of our connection here (we may
                    // still be happily sending messages even if we dropped the receiver)
                    log::error!("Failed to send data out: {}", e);
                    break;
                }
            }
        });

        // Receive messages externally to send to the socket.
        let (tx_to_ws, mut rx_from_external) = mpsc::unbounded();
        tokio::spawn(async move {
            loop {
                let msg = tokio::select! {
                    msg = rx_from_external.next() => { msg },
                    // Websocket connection closed? Don't wait for incoming message; break immediately.
                    _ = &mut rx_has_closed => { break },
                };

                let msg = match msg {
                    None => break,
                    Some(msg) => msg,
                };

                match msg {
                    SentMessageInternal::Message(SentMessage::Text(s)) => {
                        if let Err(e) = ws_to_connection.send_text_owned(s).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send text data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessageInternal::Message(SentMessage::Binary(bytes)) => {
                        if let Err(e) = ws_to_connection.send_binary_mut(bytes).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send binary data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessageInternal::Message(SentMessage::StaticText(s)) => {
                        if let Err(e) = ws_to_connection.send_text(s).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send text data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessageInternal::Message(SentMessage::StaticBinary(bytes)) => {
                        if let Err(e) = ws_to_connection.send_binary(bytes).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send binary data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessageInternal::Close => {
                        if let Err(e) = ws_to_connection.close().await {
                            log::error!("Error attempting to close connection: {}", e);
                            break;
                        }
                    }
                }

                if let Err(e) = ws_to_connection.flush().await {
                    log::error!(
                        "Shutting down websocket connection: Failed to flush data: {}",
                        e
                    );
                    break;
                }
            }
        });

        (Sender { inner: tx_to_ws }, Receiver { inner: rx_from_ws })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Handshake error: {0}")]
    Handshake(#[from] soketto::handshake::Error),
    #[error("Redirect not supported (status code: {status_code})")]
    ConnectionFailedRedirect { status_code: u16 },
    #[error("Connection rejected (status code: {status_code})")]
    ConnectionFailedRejected { status_code: u16 },
}

/// Establish a websocket connection that you can send and receive messages from.
pub async fn connect(uri: &http::Uri) -> Result<Connection, ConnectError> {
    let host = uri.host().unwrap_or("127.0.0.1");
    let port = uri.port_u16().unwrap_or(80);
    let path = uri.path();

    let socket = TcpStream::connect((host, port)).await?;
    socket.set_nodelay(true).expect("socket set_nodelay failed");

    // Establish a WS connection:
    let mut client = Client::new(socket.compat(), host, &path);
    let (ws_to_connection, ws_from_connection) = match client.handshake().await? {
        ServerResponse::Accepted { .. } => client.into_builder().finish(),
        ServerResponse::Redirect { status_code, .. } => {
            return Err(ConnectError::ConnectionFailedRedirect { status_code })
        }
        ServerResponse::Rejected { status_code } => {
            return Err(ConnectError::ConnectionFailedRejected { status_code })
        }
    };

    Ok(Connection {
        tx: ws_to_connection,
        rx: ws_from_connection,
    })
}
