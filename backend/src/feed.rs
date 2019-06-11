use serde::Serialize;
use serde_json::to_writer;
use bytes::Bytes;

pub mod connector;

pub trait FeedMessage: Serialize {
    const ACTION: u8;
}

pub struct FeedMessageSerializer {
    /// Current buffer,
    buffer: Vec<u8>,
}

impl FeedMessageSerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push<Message>(&mut self, msg: Message) -> serde_json::Result<()>
    where
        Message: FeedMessage,
    {
        let glue = match self.buffer.len() {
            0 => b'[',
            _ => b',',
        };

        self.buffer.push(glue);
        to_writer(&mut self.buffer, &Message::ACTION)?;
        self.buffer.push(b',');
        to_writer(&mut self.buffer, &msg)
    }

    pub fn finalize(&mut self) -> Option<Bytes> {
        if self.buffer.len() == 0 {
            return None;
        }

        self.buffer.push(b']');
        let bytes = self.buffer[..].into();
        self.buffer.clear();

        Some(bytes)
    }
}

impl FeedMessage for Version { const ACTION: u8 = 0x00; }
impl FeedMessage for AddedChain<'_> { const ACTION: u8 = 0x0B; }
impl FeedMessage for RemovedChain<'_> { const ACTION: u8 = 0x0C; }

#[derive(Serialize)]
pub struct Version(pub usize);

#[derive(Serialize)]
pub struct AddedChain<'a>(pub &'a str, pub usize);

#[derive(Serialize)]
pub struct RemovedChain<'a>(pub &'a str);
