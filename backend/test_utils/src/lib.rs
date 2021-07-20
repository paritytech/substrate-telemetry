/// Create/connect to a server consisting of shards and a core process that we can interact with.
pub mod server;

/// Test support for deserializing feed messages from the feed processes. This basically
/// is the slightly-lossy inverse of the custom serialization we do to feed messages.
pub mod feed_message_de;

/// A couple of macros to make it easier to test for the presense of things (mainly, feed messages)
/// in an iterable container.
#[macro_use]
pub mod contains_matches;

/// Utilities to help with running tests from within this current workspace.
pub mod workspace;