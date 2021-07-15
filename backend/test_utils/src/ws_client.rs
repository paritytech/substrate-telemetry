use futures::channel::mpsc;
use futures::{Sink, SinkExt, Stream, StreamExt};
use soketto::handshake::{Client, ServerResponse};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Send messages into the connection
#[derive(Clone)]
pub struct Sender {
    inner: mpsc::UnboundedSender<SentMessageInternal>,
}

impl Sender {
    /// Ask the underlying Websocket connection to close.
    pub async fn close(&mut self) -> Result<(), SendError> {
        self.inner.send(SentMessageInternal::Close).await?;
        Ok(())
    }
    /// Returns whether this channel is closed.
    pub fn is_closed(&mut self) -> bool {
        self.inner.is_closed()
    }
    /// Unbounded send will always queue the message and doesn't
    /// need to be awaited.
    pub fn unbounded_send(&self, msg: SentMessage) -> Result<(), SendError> {
        self.inner
            .unbounded_send(SentMessageInternal::Message(msg))
            .map_err(|e| e.into_send_error())?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum SendError {
    #[error("Failed to send message: {0}")]
    ChannelError(#[from] mpsc::SendError)
}

impl Sink<SentMessage> for Sender {
    type Error = SendError;
    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(|e| e.into())
    }
    fn start_send(mut self: std::pin::Pin<&mut Self>, item: SentMessage) -> Result<(), Self::Error> {
        self.inner
            .start_send_unpin(SentMessageInternal::Message(item))
            .map_err(|e| e.into())
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(|e| e.into())
    }
    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx).map_err(|e| e.into())
    }
}

/// Receive messages out of a connection
pub struct Receiver {
    inner: mpsc::UnboundedReceiver<Result<RecvMessage, RecvError>>,
}

#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    #[error("Text message contains invalid UTF8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Stream finished")]
    StreamFinished,
}

impl Stream for Receiver {
    type Item = Result<RecvMessage, RecvError>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx).map_err(|e| e.into())
    }
}

/// A message that can be received from the connection
#[derive(Debug, Clone)]
pub enum RecvMessage {
    /// Send an owned string into the socket.
    Text(String),
    /// Send owned bytes into the socket.
    Binary(Vec<u8>),
}

impl RecvMessage {
    pub fn len(&self) -> usize {
        match self {
            RecvMessage::Binary(b) => b.len(),
            RecvMessage::Text(s) => s.len(),
        }
    }
}

/// A message that can be sent into the connection
#[derive(Debug, Clone)]
pub enum SentMessage {
    /// Being able to send static text is primarily useful for benchmarking,
    /// so that we can avoid cloning an owned string and pass a static reference
    /// (one such option here is using [`Box::leak`] to generate strings with
    /// static lifetimes).
    StaticText(&'static str),
    /// Being able to send static bytes is primarily useful for benchmarking,
    /// so that we can avoid cloning an owned string and pass a static reference
    /// (one such option here is using [`Box::leak`] to generate bytes with
    /// static lifetimes).
    StaticBinary(&'static [u8]),
    /// Send an owned string into the socket.
    Text(String),
    /// Send owned bytes into the socket.
    Binary(Vec<u8>),
}

/// Sent messages can be anything publically visible, or a close message.
#[derive(Debug, Clone)]
enum SentMessageInternal {
    Message(SentMessage),
    Close,
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
/// A thin wrapper around Soketto that provides cancel-safe send/receive handles.
///
/// This must be called within the context of a tokio runtime.
pub async fn connect(uri: &http::Uri) -> Result<(Sender, Receiver), ConnectError> {
    let host = uri.host().unwrap_or("127.0.0.1");
    let port = uri.port_u16().unwrap_or(80);
    let path = uri.path();

    let socket = TcpStream::connect((host, port)).await?;
    socket.set_nodelay(true).expect("socket set_nodelay failed");

    // Establish a WS connection:
    let mut client = Client::new(socket.compat(), host, &path);
    let (mut ws_to_connection, mut ws_from_connection) = match client.handshake().await? {
        ServerResponse::Accepted { .. } => client.into_builder().finish(),
        ServerResponse::Redirect { status_code, .. } => {
            return Err(ConnectError::ConnectionFailedRedirect { status_code })
        }
        ServerResponse::Rejected { status_code } => {
            return Err(ConnectError::ConnectionFailedRejected { status_code })
        }
    };

    // Soketto sending/receiving isn't cancel safe, so we wrap the message stuff into spawned
    // tasks and use channels (which are cancel safe) to send/recv messages atomically..

    // Receive messages from the socket and post them out:
    let (mut tx_to_external, rx_from_ws) = mpsc::unbounded();
    tokio::spawn(async move {
        let mut data = Vec::with_capacity(128);
        loop {
            // Clear the buffer and wait for the next message to arrive:
            data.clear();

            let message_data = match ws_from_connection.receive_data(&mut data).await {
                Err(e) => {
                    // Couldn't receive data may mean all senders are gone, so log
                    // the error and shut this down:
                    log::error!(
                        "Shutting down websocket connection: Failed to receive data: {}",
                        e
                    );
                    break;
                }
                Ok(data) => data,
            };

            let msg = match message_data {
                soketto::Data::Text(_) => Ok(RecvMessage::Binary(data)),
                soketto::Data::Binary(_) => String::from_utf8(data)
                    .map(|s| RecvMessage::Text(s))
                    .map_err(|e| e.into()),
            };

            data = Vec::with_capacity(128);

            if let Err(e) = tx_to_external.send(msg).await {
                // Failure to send likely means that the recv has been dropped,
                // so let's drop this loop too.
                log::error!(
                    "Shutting down websocket connection: Failed to send data out: {}",
                    e
                );
                break;
            }
        }
    });

    // Receive messages externally to send to the socket.
    let (tx_to_ws, mut rx_from_external) = mpsc::unbounded();
    tokio::spawn(async move {
        while let Some(msg) = rx_from_external.next().await {
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
                },
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
                },
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

    Ok((Sender { inner: tx_to_ws }, Receiver { inner: rx_from_ws }))
}
