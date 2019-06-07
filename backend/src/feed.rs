use serde::Serialize;

pub mod connector;

pub trait FeedMessage: Serialize {
    const ACTION: u8;
}

#[derive(Serialize)]
pub struct Version(pub usize);

impl FeedMessage for Version { const ACTION: u8 = 0; }
