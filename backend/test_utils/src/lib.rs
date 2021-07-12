/// Create/connect to a server consisting of shards and a core process that we can interact with.
pub mod server;

/// Test support for deserializing feed messages from the feed processes. This basically
/// is the slightly-lossy inverse of the custom serialization we do to feed messages.
pub mod feed_message_de;

/// A wrapper around soketto to simplify the process of establishing connections
/// and sending messages. Provides cancel-safe message channels.
pub mod ws_client;

/// A couple of macros to make it easier to test for the presense of things (mainly, feed messages)
/// in an iterable container.
#[macro_use]
pub mod contains_matches;
