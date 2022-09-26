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
use super::on_close::OnClose;
use futures::{channel, StreamExt};
use soketto::handshake::{Client, ServerResponse};
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{OwnedTrustAnchor, ServerName};
use tokio_rustls::{rustls, TlsConnector};
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::{
    receiver::{Receiver, RecvMessage},
    sender::{Sender, SentMessage},
};

pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncReadWrite for T {}

/// The send side of a Soketto WebSocket connection
pub type RawSender =
    soketto::connection::Sender<tokio_util::compat::Compat<Box<dyn AsyncReadWrite>>>;

/// The receive side of a Soketto WebSocket connection
pub type RawReceiver =
    soketto::connection::Receiver<tokio_util::compat::Compat<Box<dyn AsyncReadWrite>>>;

/// A websocket connection. From this, we can either expose the raw connection
/// or expose a cancel-safe interface to it.
pub struct Connection {
    tx: RawSender,
    rx: RawReceiver,
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

        // Shut everything down when we're told to close, which will be either when
        // we hit an error trying to receive data on the socket, or when both the send
        // and recv channels that we hand out are dropped. Notably, we allow either recv or
        // send alone to be dropped and still keep the socket open (we may only care about
        // one way communication).
        let (tx_closed1, mut rx_closed1) = tokio::sync::broadcast::channel::<()>(1);
        let tx_closed2 = tx_closed1.clone();
        let mut rx_closed2 = tx_closed1.subscribe();

        // Receive messages from the socket:
        let (tx_to_external, rx_from_ws) = channel::mpsc::unbounded();
        tokio::spawn(async move {
            let mut send_to_external = true;
            loop {
                let mut data = Vec::new();

                // Wait for messages, or bail entirely if asked to close.
                let message_data = tokio::select! {
                    msg_data = ws_from_connection.receive_data(&mut data) => { msg_data },
                    _ = rx_closed1.recv() => { break }
                };

                let message_data = match message_data {
                    Err(e) => {
                        // The socket had an error, so notify interested parties that we should
                        // shut the connection down and bail out of this receive loop.
                        log::error!(
                            "Shutting down websocket connection: Failed to receive data: {}",
                            e
                        );
                        let _ = tx_closed1.send(());
                        break;
                    }
                    Ok(data) => data,
                };

                // if we hit an error sending, we keep receiving messages and reacting
                // to recv issues, but we stop trying to send them anywhere.
                if !send_to_external {
                    continue;
                }

                let msg = match message_data {
                    soketto::Data::Binary(_) => Ok(RecvMessage::Binary(data)),
                    soketto::Data::Text(_) => String::from_utf8(data)
                        .map(RecvMessage::Text)
                        .map_err(|e| e.into()),
                };

                if let Err(e) = tx_to_external.unbounded_send(msg) {
                    // Our external channel may have closed or errored, but the socket hasn't
                    // been closed, so keep receiving in order to allow the socket to continue to
                    // function properly (we may be happy just sending messages to it), but stop
                    // trying to hand back messages we've received from the socket.
                    log::warn!("Failed to send data out: {}", e);
                    send_to_external = false;
                }
            }
        });

        // Send messages to the socket:
        let (tx_to_ws, mut rx_from_external) = channel::mpsc::unbounded::<SentMessage>();
        tokio::spawn(async move {
            loop {
                // Wait for messages, or bail entirely if asked to close.
                let msg = tokio::select! {
                    msg = rx_from_external.next() => { msg },
                    _ = rx_closed2.recv() => {
                        // attempt to gracefully end the connection.
                        let _ = ws_to_connection.close().await;
                        break
                    }
                };

                // No more messages; channel closed. End this loop. Unlike the recv side which
                // needs to keep receiving data for the WS connection to stay open, there's no
                // reason to keep this side of the loop open if our channel is closed.
                let msg = match msg {
                    Some(msg) => msg,
                    None => break,
                };

                // We don't explicitly shut down the channel if we hit send errors. Why? Because the
                // receive side of the channel will react to socket errors as well, and close things
                // down from there.
                match msg {
                    SentMessage::Text(s) => {
                        if let Err(e) = ws_to_connection.send_text_owned(s).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send text data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessage::Binary(bytes) => {
                        if let Err(e) = ws_to_connection.send_binary_mut(bytes).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send binary data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessage::StaticText(s) => {
                        if let Err(e) = ws_to_connection.send_text(s).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send text data: {}",
                                e
                            );
                            break;
                        }
                    }
                    SentMessage::StaticBinary(bytes) => {
                        if let Err(e) = ws_to_connection.send_binary(bytes).await {
                            log::error!(
                                "Shutting down websocket connection: Failed to send binary data: {}",
                                e
                            );
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

        // Keep track of whether one of sender or received have
        // been dropped. If both have, we close the socket connection.
        let on_close = Arc::new(OnClose(tx_closed2));

        (
            Sender {
                inner: tx_to_ws,
                closer: Arc::clone(&on_close),
            },
            Receiver {
                inner: rx_from_ws,
                closer: on_close,
            },
        )
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
    let scheme = uri.scheme_str().unwrap_or("ws");
    let mut port = 80;
    if scheme == "https" || scheme == "wss" {
        port = 443
    }
    let path = uri.path();
    let port = uri.port_u16().unwrap_or(port);
    let socket = TcpStream::connect((host, port)).await?;
    socket.set_nodelay(true).expect("socket set_nodelay failed");
    // wrap TCP stream with TLS if schema is https or wss
    let socket = may_connect_tls(socket, host, scheme == "https" || scheme == "wss").await?;

    // Establish a WS connection:
    let mut client = Client::new(socket.compat(), host, path);
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

async fn may_connect_tls(
    socket: TcpStream,
    host: &str,
    use_https: bool,
) -> io::Result<Box<dyn AsyncReadWrite>> {
    if !use_https {
        return Ok(Box::new(socket));
    };
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from(host)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dns name"))?;
    let socket = connector.connect(domain, socket).await?;
    Ok(Box::new(socket))
}
