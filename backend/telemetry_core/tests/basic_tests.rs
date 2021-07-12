//! These only run when the "e2e" feature is set (eg `cargo test --features e2e`).
//! The rust IDE plugins may behave better if you comment out this line during development:
/// #![cfg(feature = "e2e")]

use test_utils::{feed_message_de::FeedMessage, server::Server};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn feed_sent_version_on_connect() {
    let server = Server::start_default()
        .await
        .expect("server could start");

    // Connect a feed:
    let (_feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    // Expect a version response of 31:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(feed_messages, vec![FeedMessage::Version(31)], "expecting version");

    // Tidy up:
    server.shutdown().await;
}

#[tokio::test]
async fn ping_responded_to_with_pong() {
    let server = Server::start_default()
        .await
        .expect("server could start");

    // Connect a feed:
    let (mut feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    // Ping it:
    feed_tx.send_command("ping", "hello!").await.unwrap();

    // Expect a pong response:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::Pong { msg: "hello!".to_owned() }), "Expecting pong");

    // Tidy up:
    server.shutdown().await;
}

#[tokio::test]
async fn node_can_be_added_and_removed() {
    let mut server = Server::start_default()
        .await
        .expect("server could start");

    // Add a shard:
    let shard_id = server.add_shard()
        .await
        .expect("shard could be added");

    // Connect a node to the shard:
    let (mut node_tx, _node_rx) = server.get_shard(shard_id)
        .unwrap()
        .connect()
        .await
        .expect("can connect to shard");

    // Send a "system connected" message:
    node_tx.send_json_text(json!(
        {
            "id":1,
            "ts":"2021-07-12T10:37:47.714666+01:00",
            "payload": {
                "authority":true,
                "chain":"Local Testnet",
                "config":"",
                "genesis_hash":"0x340358f3029f5211d20d6a1f4bbe3567b39dffd35ce0d4b358fa7c62ba3f5505",
                "implementation":"Substrate Node",
                "msg":"system.connected",
                "name":"Alice",
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1625565542717",
                "version":"2.0.0-07a1af348-aarch64-macos"
            },
        }
    )).await.unwrap();

    // Wait a little for this message to propagate to the core
    // (so that our feed connects after the core knows and not before).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect a feed.
    let (_feed_tx, mut feed_rx) = server.get_core().connect()
        .await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(
        &FeedMessage::AddedChain {
            name: "Local Testnet".to_owned(),
            node_count: 1
        }
    ));

    // Disconnect the node:
    node_tx.close().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(
        &FeedMessage::RemovedChain {
            name: "Local Testnet".to_owned(),
        }
    ));

    // Tidy up:
    server.shutdown().await;
}