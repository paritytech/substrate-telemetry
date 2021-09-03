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
use futures::channel;
use std::sync::Arc;

/// A message that can be sent into the channel interface
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

/// Send messages into the connection
#[derive(Clone)]
pub struct Sender {
    pub(super) inner: channel::mpsc::UnboundedSender<SentMessage>,
    pub(super) closer: Arc<OnClose>,
}

impl Sender {
    /// Ask the underlying Websocket connection to close.
    pub async fn close(&mut self) -> Result<(), SendError<SentMessage>> {
        self.closer.0.send(()).map_err(|_| SendError::CloseError)?;
        Ok(())
    }
    /// Returns whether this channel is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
    /// Unbounded send will always queue the message and doesn't
    /// need to be awaited.
    pub fn unbounded_send(&self, msg: SentMessage) -> Result<(), channel::mpsc::SendError> {
        self.inner
            .unbounded_send(msg)
            .map_err(|e| e.into_send_error())?;
        Ok(())
    }
    /// Convert this sender into a Sink
    pub fn into_sink(
        self,
    ) -> impl futures::Sink<SentMessage> + std::marker::Unpin + Clone + 'static {
        self.inner
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum SendError<T: std::fmt::Debug + 'static> {
    #[error("Failed to send message: {0}")]
    ChannelError(#[from] flume::SendError<T>),
    #[error("Failed to send close message")]
    CloseError,
}
