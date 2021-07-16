//! General end-to-end tests

use common::node_types::BlockHash;
use serde_json::json;
use std::time::Duration;
use test_utils::{
    assert_contains_matches,
    feed_message_de::{FeedMessage, NodeDetails},
    workspace::start_server_debug
};

/// The simplest test we can run; the main benefit of this test (since we check similar)
/// below) is just to give a feel for _how_ we can test basic feed related things.
#[tokio::test]
async fn feed_sent_version_on_connect() {
    let server = start_server_debug().await;

    // Connect a feed:
    let (_feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    // Expect a version response of 31:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(
        feed_messages,
        vec![FeedMessage::Version(31)],
        "expecting version"
    );

    // Tidy up:
    server.shutdown().await;
}

/// Another very simple test: pings from feeds should be responded to by pongs
/// with the same message content.
#[tokio::test]
async fn feed_ping_responded_to_with_pong() {
    let server = start_server_debug().await;

    // Connect a feed:
    let (mut feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    // Ping it:
    feed_tx.send_command("ping", "hello!").unwrap();

    // Expect a pong response:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(
        feed_messages.contains(&FeedMessage::Pong {
            msg: "hello!".to_owned()
        }),
        "Expecting pong"
    );

    // Tidy up:
    server.shutdown().await;
}


/// As a prelude to `lots_of_mute_messages_dont_cause_a_deadlock`, we can check that
/// a lot of nodes can simultaneously subscribe and are all sent the expected response.
#[tokio::test]
async fn multiple_feeds_sent_version_on_connect() {
    let server = start_server_debug().await;

    // Connect a bunch of feeds:
    let mut feeds = server
        .get_core()
        .connect_multiple_feeds(1000)
        .await
        .unwrap();

    // Wait for responses all at once:
    let responses = futures::future::join_all(
        feeds.iter_mut()
        .map(|(_, rx)| rx.recv_feed_messages())
    );

    let responses = tokio::time::timeout(Duration::from_secs(10), responses)
        .await
        .expect("we shouldn't hit a timeout waiting for responses");

    // Expect a version response of 31 to all of them:
    for feed_messages in responses {
        assert_eq!(
            feed_messages.expect("should have messages"),
            vec![FeedMessage::Version(31)],
            "expecting version"
        );
    }

    // Tidy up:
    server.shutdown().await;
}

/// When a lot (> ~700 in this case) of nodes are added, the chain becomes overquota.
/// this leads to a load of messages being sent back to the shard. If bounded channels
/// are used to send messages back to the shard, it's possible that we get into a situation
/// where the shard is waiting trying to send the next "add node" message, while the
/// telemetry core is waiting trying to send up to the shard the next "mute node" message,
/// resulting in a deadlock. This test gives confidence that we don't run into such a deadlock.
#[tokio::test]
async fn lots_of_mute_messages_dont_cause_a_deadlock() {
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();

    // Connect 1000 nodes to the shard:
    let mut nodes = server
        .get_shard(shard_id)
        .unwrap()
        .connect_multiple_nodes(2000) // 1500 of these will be overquota.
        .await
        .expect("nodes can connect");

    // Every node announces itself on the same chain:
    for (idx, (node_tx, _)) in nodes.iter_mut().enumerate() {
        node_tx.send_json_text(json!({
            "id":1, // message ID, not node ID. Can be the same for all.
            "ts":"2021-07-12T10:37:47.714666+01:00",
            "payload": {
                "authority":true,
                "chain":"Local Testnet",
                "config":"",
                "genesis_hash": BlockHash::from_low_u64_ne(1),
                "implementation":"Substrate Node",
                "msg":"system.connected",
                "name": format!("Alice {}", idx),
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1625565542717",
                "version":"2.0.0-07a1af348-aarch64-macos"
            }
        })).unwrap();
    }

    // Wait a little time (just to let everything get deadlocked) before
    // trying to have the aggregator send out feed messages.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Start a bunch of feeds. If deadlock has happened, none of them will
    // receive any messages back.
    let mut feeds = server
        .get_core()
        .connect_multiple_feeds(1)
        .await
        .expect("feeds can connect");

    // Wait to see whether we get anything back:
    let msgs_fut = futures::future::join_all(
        feeds
            .iter_mut()
            .map(|(_,rx)| rx.recv_feed_messages())
    );

    // Give up after a timeout:
    tokio::time::timeout(Duration::from_secs(10), msgs_fut)
        .await
        .expect("should not hit timeout waiting for messages (deadlock has happened)");
}

/// If a node is added, a connecting feed should be told about the new chain.
/// If the node is removed, the feed should be told that the chain has gone.
#[tokio::test]
async fn feed_add_and_remove_node() {
    // Connect server and add shard
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();

    // Connect a node to the shard:
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .expect("can connect to shard");

    // Send a "system connected" message:
    node_tx
        .send_json_text(json!(
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
        ))
        .unwrap();

    // Wait a little for this message to propagate to the core
    // (so that our feed connects after the core knows and not before).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect a feed.
    let (_feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::AddedChain {
        name: "Local Testnet".to_owned(),
        node_count: 1
    }));

    // Disconnect the node:
    node_tx.close().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::RemovedChain {
        name: "Local Testnet".to_owned(),
    }));

    // Tidy up:
    server.shutdown().await;
}

/// If nodes connect and the chain name changes, feeds will be told about this
/// and will keep receiving messages about the renamed chain (despite subscribing
/// to it by name).
#[tokio::test]
async fn feeds_told_about_chain_rename_and_stay_subscribed() {
    // Connect a node:
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .expect("can connect to shard");

    let node_init_msg = |id, chain_name: &str, node_name: &str| json!({
        "id":id,
        "ts":"2021-07-12T10:37:47.714666+01:00",
        "payload": {
            "authority":true,
            "chain": chain_name,
            "config":"",
            "genesis_hash": BlockHash::from_low_u64_ne(1),
            "implementation":"Substrate Node",
            "msg":"system.connected",
            "name": node_name,
            "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
            "startup_time":"1625565542717",
            "version":"2.0.0-07a1af348-aarch64-macos"
        },
    });

    // Subscribe a chain:
    node_tx.send_json_text(node_init_msg(1, "Initial chain name", "Node 1")).unwrap();

    // Connect a feed and subscribe to the above chain:
    let (mut feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();
    feed_tx.send_command("subscribe", "Initial chain name").unwrap();

    // Feed is told about the chain, and the node on this chain:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedChain { name, node_count: 1 } if name == "Initial chain name",
        FeedMessage::SubscribedTo { name } if name == "Initial chain name",
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 1",
    );

    // Subscribe another node. The chain doesn't rename yet but we are told about the new node
    // count and the node that's been added.
    node_tx.send_json_text(node_init_msg(2, "New chain name", "Node 2")).unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 2",
        FeedMessage::AddedChain { name, node_count: 2 } if name == "Initial chain name",
    );

    // Subscribe a third node. The chain renames, so we're told about the new node but also
    // about the chain rename.
    node_tx.send_json_text(node_init_msg(3, "New chain name", "Node 3")).unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 3",
        FeedMessage::RemovedChain { name } if name == "Initial chain name",
        FeedMessage::AddedChain { name, node_count: 3 } if name == "New chain name",
    );

    // Just to be sure, subscribing a fourth node on this chain will still lead to updates
    // to this feed.
    node_tx.send_json_text(node_init_msg(4, "New chain name", "Node 4")).unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 4",
        FeedMessage::AddedChain { name, node_count: 4 } if name == "New chain name",
    );

}

/// If we add a couple of shards and a node for each, all feeds should be
/// told about both node chains. If one shard goes away, we should get a
/// "removed chain" message only for the node connected to that shard.
#[tokio::test]
async fn feed_add_and_remove_shard() {
    let mut server = start_server_debug().await;

    let mut shards = vec![];
    for id in 1..=2 {
        // Add a shard:
        let shard_id = server.add_shard().await.unwrap();

        // Connect a node to it:
        let (mut node_tx, _node_rx) = server
            .get_shard(shard_id)
            .unwrap()
            .connect_node()
            .await
            .expect("can connect to shard");

        // Send a "system connected" message:
        node_tx
            .send_json_text(json!({
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
            }))
            .unwrap();

        // Keep what we need to to keep connection alive and let us kill a shard:
        shards.push((shard_id, node_tx));
    }

    // Connect a feed.
    let (_feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    // The feed should be told about both of the chains that we've sent info about:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::AddedChain {
        name: "Local Testnet 1".to_owned(),
        node_count: 1
    }));
    assert!(feed_messages.contains(&FeedMessage::AddedChain {
        name: "Local Testnet 2".to_owned(),
        node_count: 1
    }));

    // Disconnect the first shard:
    server.kill_shard(shards[0].0).await;

    // We should be told about the node connected to that shard disconnecting:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::RemovedChain {
        name: "Local Testnet 1".to_owned(),
    }));
    assert!(!feed_messages.contains(
        // Spot the "!"; this chain was not removed.
        &FeedMessage::RemovedChain {
            name: "Local Testnet 2".to_owned(),
        }
    ));

    // Tidy up:
    server.shutdown().await;
}

/// feeds can subscribe to one chain at a time. They should get the relevant
/// messages for that chain and no other.
#[tokio::test]
async fn feed_can_subscribe_and_unsubscribe_from_chain() {
    use FeedMessage::*;

    // Start server, add shard, connect node:
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server.get_shard(shard_id).unwrap().connect_node().await.unwrap();

    // Send a "system connected" message for a few nodes/chains:
    for id in 1..=3 {
        node_tx
            .send_json_text(json!(
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
            ))
            .unwrap();
    }

    // Connect a feed
    let (mut feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(feed_messages, AddedChain { name, node_count: 1 } if name == "Local Testnet 1");

    // Subscribe it to a chain
    feed_tx.send_command("subscribe", "Local Testnet 1").unwrap();

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
    )).unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_ne!(feed_messages.len(), 0);

    // We don't receive anything for updates to nodes on other chains (wait a sec to ensure no messages are sent):
    node_tx.send_json_text(json!(
        {"id":2, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:37:48.330433+01:00" }
    )).unwrap();

    tokio::time::timeout(Duration::from_secs(1), feed_rx.recv_feed_messages())
        .await
        .expect_err("Timeout should elapse since no messages sent");

    // We can change our subscription:
    feed_tx.send_command("subscribe", "Local Testnet 2").unwrap();
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
    )).unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_ne!(feed_messages.len(), 0);

    // Tidy up:
    server.shutdown().await;
}
