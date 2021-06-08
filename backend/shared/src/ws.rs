use actix_http::ws::Item;
use actix_web_actors::ws::{self, CloseReason, CloseCode};
use bytes::{Bytes, BytesMut};
use serde::{Serialize, Deserialize};
use actix::prelude::Message;

/// Helper that will buffer continuation messages from actix
/// until completion, capping at 10mb.
#[derive(Default)]
pub struct MultipartHandler {
    buf: BytesMut,
}

/// Message to signal that a node should be muted for a reason that's
/// cheap to transfer between Actors or over the wire for shards.
#[derive(Serialize, Deserialize, Message, Clone, Copy, Debug)]
#[rtype("()")]
pub enum MuteReason {
    /// Node was denied connection for any arbitrary reason,
    /// and should not attempt to reconnect.
    Denied,
    /// Node was denied because the chain it belongs to is currently
    /// at the limit of allowed nodes, and it may attempt to reconnect.
    Overquota,
}

impl From<MuteReason> for CloseReason {
    fn from(mute: MuteReason) -> CloseReason {
        match mute {
            MuteReason::Denied => CloseReason {
                code: CloseCode::Abnormal,
                description: Some("Denied".into()),
            },
            MuteReason::Overquota => CloseReason {
                code: CloseCode::Again,
                description: Some("Overquota".into()),
            },
        }
    }
}

/// Continuation buffer limit, 10mb
const CONT_BUF_LIMIT: usize = 10 * 1024 * 1024;

pub enum WsMessage {
    Nop,
    Ping(Bytes),
    Data(Bytes),
    Close(Option<CloseReason>),
}

impl MultipartHandler {
    pub fn handle(&mut self, msg: ws::Message) -> WsMessage {
        match msg {
            ws::Message::Ping(msg) => WsMessage::Ping(msg),
            ws::Message::Pong(_) => WsMessage::Nop,
            ws::Message::Text(text) => WsMessage::Data(text.into_bytes()),
            ws::Message::Binary(data) => WsMessage::Data(data),
            ws::Message::Close(reason) => WsMessage::Close(reason),
            ws::Message::Nop => WsMessage::Nop,
            ws::Message::Continuation(cont) => match cont {
                Item::FirstText(bytes) | Item::FirstBinary(bytes) => {
                    self.start_frame(&bytes);
                    WsMessage::Nop
                }
                Item::Continue(bytes) => {
                    self.continue_frame(&bytes);
                    WsMessage::Nop
                }
                Item::Last(bytes) => {
                    self.continue_frame(&bytes);
                    WsMessage::Data(self.finish_frame())
                }
            },
        }
    }

    fn start_frame(&mut self, bytes: &[u8]) {
        if !self.buf.is_empty() {
            log::error!("Unused continuation buffer");
            self.buf.clear();
        }
        self.continue_frame(bytes);
    }

    fn continue_frame(&mut self, bytes: &[u8]) {
        if self.buf.len() + bytes.len() <= CONT_BUF_LIMIT {
            self.buf.extend_from_slice(&bytes);
        } else {
            log::error!("Continuation buffer overflow");
            self.buf = BytesMut::new();
        }
    }

    fn finish_frame(&mut self) -> Bytes {
        std::mem::replace(&mut self.buf, BytesMut::new()).freeze()
    }
}
