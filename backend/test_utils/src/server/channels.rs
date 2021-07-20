use std::{ops::{Deref, DerefMut}, time::Duration};

use crate::feed_message_de::FeedMessage;
use common::ws_client;
use futures::{Sink, SinkExt, Stream, StreamExt};

/// Wrap a `ws_client::Sender` with convenient utility methods for shard connections
pub struct ShardSender(ws_client::Sender);

impl From<ws_client::Sender> for ShardSender {
    fn from(c: ws_client::Sender) -> Self {
        ShardSender(c)
    }
}

impl Sink<ws_client::SentMessage> for ShardSender {
    type Error = ws_client::SendError;
    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready_unpin(cx)
    }
    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: ws_client::SentMessage,
    ) -> Result<(), Self::Error> {
        self.0.start_send_unpin(item)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_flush_unpin(cx)
    }
    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_close_unpin(cx)
    }
}

impl ShardSender {
    /// Send JSON as a binary websocket message
    pub fn send_json_binary(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), ws_client::SendError> {
        let bytes = serde_json::to_vec(&json).expect("valid bytes");
        self.unbounded_send(ws_client::SentMessage::Binary(bytes))
    }
    /// Send JSON as a textual websocket message
    pub fn send_json_text(
        &mut self,
        json: serde_json::Value,
    ) -> Result<(), ws_client::SendError> {
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

impl Sink<ws_client::SentMessage> for FeedSender {
    type Error = ws_client::SendError;
    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready_unpin(cx)
    }
    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: ws_client::SentMessage,
    ) -> Result<(), Self::Error> {
        self.0.start_send_unpin(item)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_flush_unpin(cx)
    }
    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_close_unpin(cx)
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
        &mut self,
        command: S,
        param: S,
    ) -> Result<(), ws_client::SendError> {
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
        self.0.poll_next_unpin(cx).map_err(|e| e.into())
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
    pub async fn recv_feed_messages_once(&mut self) -> Result<Vec<FeedMessage>, anyhow::Error> {
        let msg = self
            .0
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("Stream closed: no more messages"))??;

        match msg {
            ws_client::RecvMessage::Binary(data) => {
                let messages = FeedMessage::from_bytes(&data)?;
                Ok(messages)
            },
            ws_client::RecvMessage::Text(text) => {
                let messages = FeedMessage::from_bytes(text.as_bytes())?;
                Ok(messages)
            },
        }
    }

    /// Wait for feed messages to be sent back, building up a list of output messages until
    /// the channel goes quiet for a short while.
    pub async fn recv_feed_messages(&mut self) -> Result<Vec<FeedMessage>, anyhow::Error> {
        // Block as long as needed for messages to start coming in:
        let mut feed_messages = self.recv_feed_messages_once().await?;
        // Then, loop a little to make sure we catch any additional messages that are sent soon after:
        loop {
            match tokio::time::timeout(Duration::from_millis(250), self.recv_feed_messages_once())
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
}
