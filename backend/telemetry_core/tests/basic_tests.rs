use test_utils::{
    feed_message_de::{ FeedMessage, NodeDetails },
    server::{ self, Server },
    assert_contains_matches
};
use serde_json::json;
use std::time::Duration;
use common::node_types::{ BlockHash };

async fn cargo_run_server() -> Server {
    Server::start(server::StartOpts {
        shard_command: server::cargo_run_commands::telemetry_shard().expect("valid shard command"),
        core_command: server::cargo_run_commands::telemetry_core().expect("valid core command")
    }).await.unwrap()
}

#[tokio::test]
async fn feed_sent_version_on_connect() {
    let server = cargo_run_server().await;

    // Connect a feed:
    let (_feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    // Expect a version response of 31:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(feed_messages, vec![FeedMessage::Version(31)], "expecting version");

    // Tidy up:
    server.shutdown().await;
}

#[tokio::test]
async fn feed_ping_responded_to_with_pong() {
    let server = cargo_run_server().await;

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
async fn feed_add_and_remove_node() {
    // Connect server and add shard
    let mut server = cargo_run_server().await;
    let shard_id = server.add_shard().await.unwrap();

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
                "genesis_hash": BlockHash::from_low_u64_ne(1),
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

#[tokio::test]
async fn feed_add_and_remove_shard() {
    let mut server = cargo_run_server().await;

    let mut shards = vec![];
    for id in 1 ..= 2 {
        // Add a shard:
        let shard_id = server.add_shard().await.unwrap();

        // Connect a node to it:
        let (mut node_tx, _node_rx) = server.get_shard(shard_id)
            .unwrap()
            .connect()
            .await
            .expect("can connect to shard");

        // Send a "system connected" message:
        node_tx.send_json_text(json!(
            {
                "id":id,
                "ts":"2021-07-12T10:37:47.714666+01:00",
                "payload": {
                    "authority":true,
                    "chain": format!("Local Testnet {}", id),
                    "config":"",
                    "genesis_hash": BlockHash::from_low_u64_ne(id),
                    "implementation":"Substrate Node",
                    "msg":"system.connected",
                    "name":"Alice",
                    "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                    "startup_time":"1625565542717",
                    "version":"2.0.0-07a1af348-aarch64-macos"
                },
            }
        )).await.unwrap();

        // Keep what we need to to keep connection alive and let us kill a shard:
        shards.push((shard_id, node_tx));
    }

    // Connect a feed.
    let (_feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    // The feed should be told about both of the chains that we've sent info about:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(
        &FeedMessage::AddedChain {
            name: "Local Testnet 1".to_owned(),
            node_count: 1
        }
    ));
    assert!(feed_messages.contains(
        &FeedMessage::AddedChain {
            name: "Local Testnet 2".to_owned(),
            node_count: 1
        }
    ));

    // Disconnect the first shard:
    server.kill_shard(shards[0].0).await;

    // We should be told about the node connected to that shard disconnecting:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(
        &FeedMessage::RemovedChain {
            name: "Local Testnet 1".to_owned(),
        }
    ));
    assert!(!feed_messages.contains( // Spot the "!"; this chain was not removed.
        &FeedMessage::RemovedChain {
            name: "Local Testnet 2".to_owned(),
        }
    ));

    // Tidy up:
    server.shutdown().await;
}

#[tokio::test]
async fn feed_can_subscribe_and_unsubscribe_from_chain() {
    use FeedMessage::*;

    // Start server, add shard, connect node:
    let mut server = cargo_run_server().await;
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server.get_shard(shard_id).unwrap().connect().await.unwrap();

    // Send a "system connected" message for a few nodes/chains:
    for id in 1..=3 {
        node_tx.send_json_text(json!(
            {
                "id":id,
                "ts":"2021-07-12T10:37:47.714666+01:00",
                "payload": {
                    "authority":true,
                    "chain":format!("Local Testnet {}", id),
                    "config":"",
                    "genesis_hash": BlockHash::from_low_u64_ne(id),
                    "implementation":"Substrate Node",
                    "msg":"system.connected",
                    "name":format!("Alice {}", id),
                    "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                    "startup_time":"1625565542717",
                    "version":"2.0.0-07a1af348-aarch64-macos"
                },
            }
        )).await.unwrap();
    }

    // Connect a feed
    let (mut feed_tx, mut feed_rx) = server.get_core().connect().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(feed_messages, AddedChain { name, node_count: 1 } if name == "Local Testnet 1");

    // Subscribe it to a chain
    feed_tx.send_command("subscribe", "Local Testnet 1").await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        SubscribedTo { name } if name == "Local Testnet 1",
        TimeSync {..},
        BestBlock { block_number: 0, timestamp: 0, avg_block_time: None },
        BestFinalized { block_number: 0, .. },
        AddedNode { node_id: 0, node: NodeDetails { name, .. }, .. } if name == "Alice 1",
        FinalizedBlock { node_id: 0, block_number: 0, .. }
    );

    // We receive updates relating to nodes on that chain:
    node_tx.send_json_text(json!(
        {"id":1, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:37:48.330433+01:00" }
    )).await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_ne!(feed_messages.len(), 0);

    // We don't receive anything for updates to nodes on other chains (wait a sec to ensure no messages are sent):
    node_tx.send_json_text(json!(
        {"id":2, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:37:48.330433+01:00" }
    )).await.unwrap();

    tokio::time::timeout(Duration::from_secs(1), feed_rx.recv_feed_messages())
        .await
        .expect_err("Timeout should elapse since no messages sent");

    // We can change our subscription:
    feed_tx.send_command("subscribe", "Local Testnet 2").await.unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();

    // We are told about the subscription change and given similar on-subscribe messages to above.
    assert_contains_matches!(
        &feed_messages,
        UnsubscribedFrom { name } if name == "Local Testnet 1",
        SubscribedTo { name } if name == "Local Testnet 2",
        TimeSync {..},
        BestBlock {..},
        BestFinalized {..},
        AddedNode { node: NodeDetails { name, .. }, ..} if name == "Alice 2",
        FinalizedBlock {..},
    );

    // We didn't get messages from this earlier, but we will now we've subscribed:
    node_tx.send_json_text(json!(
        {"id":2, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:38:48.330433+01:00" }
    )).await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_ne!(feed_messages.len(), 0);

    // Tidy up:
    server.shutdown().await;
}