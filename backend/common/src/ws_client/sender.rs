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
use futures::{Sink, SinkExt};

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

/// Messages sent into the channel interface can be anything publically visible, or a close message.
#[derive(Debug, Clone)]
pub(super) enum SentMessageInternal {
    Message(SentMessage),
    Close,
}

/// Send messages into the connection
#[derive(Clone)]
pub struct Sender {
    pub(super) inner: mpsc::UnboundedSender<SentMessageInternal>,
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
    ChannelError(#[from] mpsc::SendError),
}

impl Sink<SentMessage> for Sender {
    type Error = SendError;
    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(|e| e.into())
    }
    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SentMessage,
    ) -> Result<(), Self::Error> {
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
