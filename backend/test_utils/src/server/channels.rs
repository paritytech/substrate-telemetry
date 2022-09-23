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

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::feed_message_de::FeedMessage;
use common::ws_client;
use futures::{channel, Stream, StreamExt};

/// Wrap a `ws_client::Sender` with convenient utility methods for shard connections
pub struct ShardSender(ws_client::Sender);

impl From<ws_client::Sender> for ShardSender {
    fn from(c: ws_client::Sender) -> Self {
        ShardSender(c)
    }
}

impl ShardSender {
    /// Send JSON as a binary websocket message
    pub fn send_json_binary(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), channel::mpsc::SendError> {
        let bytes = serde_json::to_vec(&json).expect("valid bytes");
        self.unbounded_send(ws_client::SentMessage::Binary(bytes))
    }
    /// Send JSON as a textual websocket message
    pub fn send_json_text(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), channel::mpsc::SendError> {
        let s = serde_json::to_string(&json).expect("valid string");
        self.unbounded_send(ws_client::SentMessage::Text(s))
    }
}

impl Deref for ShardSender {
    type Target = ws_client::Sender;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ShardSender {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wrap a `ws_client::Receiver` with convenient utility methods for shard connections
pub struct ShardReceiver(ws_client::Receiver);

impl From<ws_client::Receiver> for ShardReceiver {
    fn from(c: ws_client::Receiver) -> Self {
        ShardReceiver(c)
    }
}

impl Stream for ShardReceiver {
    type Item = Result<ws_client::RecvMessage, ws_client::RecvError>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

impl Deref for ShardReceiver {
    type Target = ws_client::Receiver;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ShardReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wrap a `ws_client::Sender` with convenient utility methods for feed connections
pub struct FeedSender(ws_client::Sender);

impl From<ws_client::Sender> for FeedSender {
    fn from(c: ws_client::Sender) -> Self {
        FeedSender(c)
    }
}

impl Deref for FeedSender {
    type Target = ws_client::Sender;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FeedSender {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FeedSender {
    /// Send a command into the feed. A command consists of a string
    /// "command" part, and another string "parameter" part.
    pub fn send_command<S: AsRef<str>>(
        &self,
        command: S,
        param: S,
    ) -> Result<(), channel::mpsc::SendError> {
        self.unbounded_send(ws_client::SentMessage::Text(format!(
            "{}:{}",
            command.as_ref(),
            param.as_ref()
        )))
    }
}

/// Wrap a `ws_client::Receiver` with convenient utility methods for feed connections
pub struct FeedReceiver(ws_client::Receiver);

impl From<ws_client::Receiver> for FeedReceiver {
    fn from(c: ws_client::Receiver) -> Self {
        FeedReceiver(c)
    }
}

impl Stream for FeedReceiver {
    type Item = Result<ws_client::RecvMessage, ws_client::RecvError>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx).map_err(|e| e)
    }
}

impl Deref for FeedReceiver {
    type Target = ws_client::Receiver;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FeedReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FeedReceiver {
    /// Wait for the next set of feed messages to arrive. Returns an error if the connection
    /// is closed, or the messages that come back cannot be properly decoded.
    ///
    /// Prefer [`FeedReceiver::recv_feed_messages`]; tests should generally be
    /// robust in assuming that messages may not all be delivered at once (unless we are
    /// specifically testing which messages are buffered together).
    pub async fn recv_feed_messages_once_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Vec<FeedMessage>, anyhow::Error> {
        let msg = match tokio::time::timeout(timeout, self.0.next()).await {
            // Timeout elapsed; no messages back:
            Err(_) => return Ok(Vec::new()),
            // Something back; Complain if error no stream closed:
            Ok(res) => res.ok_or_else(|| anyhow::anyhow!("Stream closed: no more messages"))??,
        };

        match msg {
            ws_client::RecvMessage::Binary(data) => {
                let messages = FeedMessage::from_bytes(&data)?;
                Ok(messages)
            }
            ws_client::RecvMessage::Text(text) => {
                let messages = FeedMessage::from_bytes(text.as_bytes())?;
                Ok(messages)
            }
        }
    }

    /// Wait for the next set of feed messages to arrive.
    /// See `recv_feed_messages_once_timeout`.
    pub async fn recv_feed_messages_once(&mut self) -> Result<Vec<FeedMessage>, anyhow::Error> {
        // This will never practically end; use the `timeout` version explciitly if you want that.
        self.recv_feed_messages_once_timeout(Duration::from_secs(u64::MAX))
            .await
    }

    /// Wait for feed messages to be sent back, building up a list of output messages until
    /// the channel goes quiet for a short while.
    ///
    /// If no new messages are received within the timeout given, bail with whatever we have so far.
    /// This differs from `recv_feed_messages` and `recv_feed_messages_once`, which will block indefinitely
    /// waiting for something to arrive
    pub async fn recv_feed_messages_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Vec<FeedMessage>, anyhow::Error> {
        // Block as long as needed for messages to start coming in:
        let mut feed_messages =
            match tokio::time::timeout(timeout, self.recv_feed_messages_once()).await {
                Ok(msgs) => msgs?,
                Err(_) => return Ok(Vec::new()),
            };

        // Then, loop a little to make sure we catch any additional messages that are sent soon after:
        loop {
            match tokio::time::timeout(Duration::from_millis(1000), self.recv_feed_messages_once())
                .await
            {
                // Timeout elapsed; return the messages we have so far
                Err(_) => {
                    break Ok(feed_messages);
                }
                // Append messages that come back to our vec
                Ok(Ok(mut msgs)) => {
                    feed_messages.append(&mut msgs);
                }
                // Error came back receiving messages; return it
                Ok(Err(e)) => break Err(e),
            }
        }
    }

    /// Wait for feed messages until nothing else arrives in a timely fashion.
    /// See `recv_feed_messages_timeout`.
    pub async fn recv_feed_messages(&mut self) -> Result<Vec<FeedMessage>, anyhow::Error> {
        // This will never practically end; use the `timeout` version explciitly if you want that.
        self.recv_feed_messages_timeout(Duration::from_secs(u64::MAX))
            .await
    }
}
