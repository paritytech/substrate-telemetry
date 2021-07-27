use futures::channel::mpsc;
use futures::{Stream, StreamExt};

/// Receive messages out of a connection
pub struct Receiver {
    pub(super) inner: mpsc::UnboundedReceiver<Result<RecvMessage, RecvError>>,
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

/// A message that can be received from the channel interface
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
