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
use futures::{Stream, StreamExt};

/// Receive messages out of a connection
pub struct Receiver {
    pub(super) inner: mpsc::UnboundedReceiver<Result<RecvMessage, RecvError>>,
    pub(super) closer: tokio::sync::broadcast::Sender<()>,
    pub(super) count: std::sync::Arc<()>,
}

impl Drop for Receiver {
    fn drop(&mut self) {
        // Close the socket connection if this is the last half
        // of the channel (ie the sender has been dropped already).
        if std::sync::Arc::strong_count(&self.count) == 1 {
            let _ = self.closer.send(());
        }
    }
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
