use actix_http::ws::Item;
use actix_web_actors::ws::{self, CloseReason};
use bytes::{Bytes, BytesMut};

/// Helper that will buffer continuation messages from actix
/// until completion, capping at 10mb.
#[derive(Default)]
pub struct MultipartHandler {
    buf: BytesMut,
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
