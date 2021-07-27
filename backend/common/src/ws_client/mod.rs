/// Functionality to establish a connection
mod connect;
/// The channel based receive interface
mod receiver;
/// The channel based send interface
mod sender;

pub use connect::{connect, ConnectError, Connection, RawReceiver, RawSender};
pub use receiver::{Receiver, RecvError, RecvMessage};
pub use sender::{SendError, Sender, SentMessage};
