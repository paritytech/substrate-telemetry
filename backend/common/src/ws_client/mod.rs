/// Functionality to establish a connection
mod connect;
/// The channel based send interface
mod sender;
/// The channel based receive interface
mod receiver;

pub use connect::{ connect, ConnectError, Connection, RawSender, RawReceiver };
pub use sender::{ Sender, SentMessage, SendError };
pub use receiver::{ Receiver, RecvMessage, RecvError };