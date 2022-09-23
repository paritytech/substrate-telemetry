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
use futures::{channel, Stream, StreamExt};
use std::sync::Arc;

/// Receive messages out of a connection
pub struct Receiver {
    pub(super) inner: channel::mpsc::UnboundedReceiver<Result<RecvMessage, RecvError>>,
    pub(super) closer: Arc<OnClose>,
}

#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    #[error("Text message contains invalid UTF8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Stream finished")]
    StreamFinished,
    #[error("Failed to send close message")]
    CloseError,
}

impl Receiver {
    /// Ask the underlying Websocket connection to close.
    pub async fn close(&mut self) -> Result<(), RecvError> {
        self.closer.0.send(()).map_err(|_| RecvError::CloseError)?;
        Ok(())
    }
}

impl Stream for Receiver {
    type Item = Result<RecvMessage, RecvError>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx).map_err(|e| e)
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
