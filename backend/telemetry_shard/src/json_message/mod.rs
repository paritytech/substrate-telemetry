//! This module contains the types we need to deserialize JSON messages from nodes

mod hash;
mod node_message;

pub use hash::Hash;
pub use node_message::*;
