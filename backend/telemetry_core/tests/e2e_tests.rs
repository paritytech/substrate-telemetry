// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

/*!
General end-to-end tests

Note that on MacOS inparticular, you may need to increase some limits to be
able to open a large number of connections and run some of the tests.
Try running these:

```sh
sudo sysctl -w kern.maxfiles=100000
sudo sysctl -w kern.maxfilesperproc=100000
ulimit -n 100000
sudo sysctl -w kern.ipc.somaxconn=100000
sudo sysctl -w kern.ipc.maxsockbuf=16777216
```

These tests can be run with:

```sh
cargo test e2e -- --ignored
```
*/

use common::node_types::BlockHash;
use common::ws_client::SentMessage;
use serde_json::json;
use std::{str::FromStr, time::Duration};
use test_utils::{
    assert_contains_matches,
    feed_message_de::{FeedMessage, NodeDetails},
    workspace::{start_server, start_server_debug, CoreOpts, ServerOpts, ShardOpts},
};

fn polkadot_genesis_hash() -> BlockHash {
    BlockHash::from_str("0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3")
        .expect("valid polkadot genesis hash")
}

/// Helper for concise testing
fn ghash(id: u64) -> BlockHash {
    BlockHash::from_low_u64_be(id)
}

/// The simplest test we can run; the main benefit of this test (since we check similar)
/// below) is just to give a feel for _how_ we can test basic feed related things.
#[tokio::test]
async fn e2e_feed_sent_version_on_connect() {
    let server = start_server_debug().await;

    // Connect a feed:
    let (_feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    // Expect a version response of 31:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_eq!(
        feed_messages,
        vec![FeedMessage::Version(32)],
        "expecting version"
    );

    // Tidy up:
    server.shutdown().await;
}

/// Another very simple test: pings from feeds should be responded to by pongs
/// with the same message content.
#[tokio::test]
async fn e2e_feed_ping_responded_to_with_pong() {
    let server = start_server_debug().await;

    // Connect a feed:
    let (feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

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
async fn e2e_multiple_feeds_sent_version_on_connect() {
    let server = start_server_debug().await;

    // Connect a bunch of feeds:
    let mut feeds = server
        .get_core()
        .connect_multiple_feeds(1000)
        .await
        .unwrap();

    // Wait for responses all at once:
    let responses =
        futures::future::join_all(feeds.iter_mut().map(|(_, rx)| rx.recv_feed_messages()));

    let responses = tokio::time::timeout(Duration::from_secs(10), responses)
        .await
        .expect("we shouldn't hit a timeout waiting for responses");

    // Expect a version response of 31 to all of them:
    for feed_messages in responses {
        assert_eq!(
            feed_messages.expect("should have messages"),
            vec![FeedMessage::Version(32)],
            "expecting version"
        );
    }

    // Tidy up:
    server.shutdown().await;
}

/// When a lot of nodes are added, the chain becomes overquota.
/// This leads to a load of messages being sent back to the shard. If bounded channels
/// are used to send messages back to the shard, it's possible that we get into a situation
/// where the shard is waiting trying to send the next "add node" message, while the
/// telemetry core is waiting trying to send up to the shard the next "mute node" message,
/// resulting in a deadlock. This test gives confidence that we don't run into such a deadlock.
#[tokio::test]
async fn e2e_lots_of_mute_messages_dont_cause_a_deadlock() {
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
        node_tx
            .send_json_text(json!({
                "id":1, // message ID, not node ID. Can be the same for all.
                "ts":"2021-07-12T10:37:47.714666+01:00",
                "payload": {
                    "authority":true,
                    "chain":"Local Testnet",
                    "config":"",
                    "genesis_hash": ghash(1),
                    "implementation":"Substrate Node",
                    "msg":"system.connected",
                    "name": format!("Alice {}", idx),
                    "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                    "startup_time":"1625565542717",
                    "version":"2.0.0-07a1af348-aarch64-macos"
                }
            }))
            .unwrap();
    }

    // Wait a little time (just to let everything get deadlocked) before
    // trying to have the aggregator send out feed messages.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Start a feed. If deadlock has happened, it won't receive
    // any messages.
    let (_, mut feed_rx) = server
        .get_core()
        .connect_feed()
        .await
        .expect("feed can connect");

    // Give up after a timeout:
    tokio::time::timeout(Duration::from_secs(10), feed_rx.recv_feed_messages())
        .await
        .expect("should not hit timeout waiting for messages (deadlock has happened)")
        .expect("shouldn't run into error receiving messages");
}

/// If a node is added, a connecting feed should be told about the new chain.
/// If the node is removed, the feed should be told that the chain has gone.
#[tokio::test]
async fn e2e_feed_add_and_remove_node() {
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
                    "genesis_hash": ghash(1),
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
        genesis_hash: ghash(1),
        node_count: 1,
    }));

    // Disconnect the node:
    node_tx.close().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::RemovedChain {
        genesis_hash: ghash(1),
    }));

    // Tidy up:
    server.shutdown().await;
}

/// If nodes connect and the chain name changes, feeds will be told about this
/// and will keep receiving messages about the renamed chain (despite subscribing
/// to it by name).
#[tokio::test]
async fn e2e_feeds_told_about_chain_rename_and_stay_subscribed() {
    // Connect a node:
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .expect("can connect to shard");

    let node_init_msg = |id, chain_name: &str, node_name: &str| {
        json!({
            "id":id,
            "ts":"2021-07-12T10:37:47.714666+01:00",
            "payload": {
                "authority":true,
                "chain": chain_name,
                "config":"",
                "genesis_hash": ghash(1),
                "implementation":"Substrate Node",
                "msg":"system.connected",
                "name": node_name,
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1625565542717",
                "version":"2.0.0-07a1af348-aarch64-macos"
            },
        })
    };

    // Subscribe a chain:
    node_tx
        .send_json_text(node_init_msg(1, "Initial chain name", "Node 1"))
        .unwrap();

    // Wait a little for this message to propagate to the core so that
    // it knows what we're on about when we subscribe, below.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect a feed and subscribe to the above chain:
    let (feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();
    feed_tx
        .send_command(
            "subscribe",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

    // Feed is told about the chain, and the node on this chain:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedChain { name, genesis_hash, node_count: 1 } if name == "Initial chain name" && genesis_hash == ghash(1),
        FeedMessage::SubscribedTo { genesis_hash } if genesis_hash == ghash(1),
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 1",
    );

    // Subscribe another node. The chain doesn't rename yet but we are told about the new node
    // count and the node that's been added.
    node_tx
        .send_json_text(node_init_msg(2, "New chain name", "Node 2"))
        .unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 2",
        FeedMessage::AddedChain { name, genesis_hash, node_count: 2 } if name == "Initial chain name" && genesis_hash == ghash(1),
    );

    // Subscribe a third node. The chain renames, so we're told about the new node but also
    // about the chain rename.
    node_tx
        .send_json_text(node_init_msg(3, "New chain name", "Node 3"))
        .unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 3",
        FeedMessage::RemovedChain { genesis_hash } if genesis_hash == ghash(1),
        FeedMessage::AddedChain { name, genesis_hash, node_count: 3 } if name == "New chain name" && genesis_hash == ghash(1),
    );

    // Just to be sure, subscribing a fourth node on this chain will still lead to updates
    // to this feed.
    node_tx
        .send_json_text(node_init_msg(4, "New chain name", "Node 4"))
        .unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        FeedMessage::AddedNode { node: NodeDetails { name: node_name, .. }, ..} if node_name == "Node 4",
        FeedMessage::AddedChain { name, genesis_hash, node_count: 4 } if name == "New chain name" && genesis_hash == ghash(1),
    );
}

/// If we add a couple of shards and a node for each, all feeds should be
/// told about both node chains. If one shard goes away, we should get a
/// "removed chain" message only for the node connected to that shard.
#[tokio::test]
async fn e2e_feed_add_and_remove_shard() {
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
                    "genesis_hash": ghash(id),
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
        genesis_hash: ghash(1),
        node_count: 1
    }));
    assert!(feed_messages.contains(&FeedMessage::AddedChain {
        name: "Local Testnet 2".to_owned(),
        genesis_hash: ghash(2),
        node_count: 1
    }));

    // Disconnect the first shard:
    server.kill_shard(shards[0].0).await;

    // We should be told about the node connected to that shard disconnecting:
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert!(feed_messages.contains(&FeedMessage::RemovedChain {
        genesis_hash: ghash(1),
    }));
    assert!(!feed_messages.contains(
        // Spot the "!"; this chain was not removed.
        &FeedMessage::RemovedChain {
            genesis_hash: ghash(2),
        }
    ));

    // Tidy up:
    server.shutdown().await;
}

/// feeds can subscribe to one chain at a time. They should get the relevant
/// messages for that chain and no other.
#[tokio::test]
async fn e2e_feed_can_subscribe_and_unsubscribe_from_chain() {
    use FeedMessage::*;

    // Start server, add shard, connect node:
    let mut server = start_server_debug().await;
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .unwrap();

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
                        "genesis_hash": ghash(id),
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
    let (feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(feed_messages, AddedChain { name, genesis_hash, node_count: 1 } if name == "Local Testnet 1" && genesis_hash == ghash(1));

    // Subscribe it to a chain
    feed_tx
        .send_command(
            "subscribe",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();
    assert_contains_matches!(
        feed_messages,
        SubscribedTo { genesis_hash } if genesis_hash == ghash(1),
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
    feed_tx
        .send_command(
            "subscribe",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();
    let feed_messages = feed_rx.recv_feed_messages().await.unwrap();

    // We are told about the subscription change and given similar on-subscribe messages to above.
    assert_contains_matches!(
        &feed_messages,
        UnsubscribedFrom { genesis_hash } if *genesis_hash == ghash(1),
        SubscribedTo { genesis_hash } if *genesis_hash == ghash(2),
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

/// If a node sends more than some rolling average amount of data, it'll be booted.
#[tokio::test]
async fn e2e_node_banned_if_it_sends_too_much_data() {
    async fn try_send_data(max_bytes: usize, send_msgs: usize, bytes_per_msg: usize) -> bool {
        let mut server = start_server(
            ServerOpts::default(),
            CoreOpts::default(),
            ShardOpts {
                // Remember, this is (currently) averaged over the last 10 seconds,
                // so we need to send 10x this amount of data for an imemdiate ban:
                max_node_data_per_second: Some(max_bytes),
                ..Default::default()
            },
        )
        .await;

        // Give us a shard to talk to:
        let shard_id = server.add_shard().await.unwrap();
        let (node_tx, _node_rx) = server
            .get_shard(shard_id)
            .unwrap()
            .connect_node()
            .await
            .unwrap();

        // Send the data requested to the shard:
        for _ in 0..send_msgs {
            node_tx
                .unbounded_send(SentMessage::Binary(vec![1; bytes_per_msg]))
                .unwrap();
        }

        // Wait a little for the shard to react and cut off the connection (or not):
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Has the connection been closed?
        node_tx.is_closed()
    }

    assert_eq!(
        try_send_data(1000, 10, 1000).await,
        false,
        "shouldn't be closed; we didn't exceed 10x threshold"
    );
    assert_eq!(
        try_send_data(999, 10, 1000).await,
        true,
        "should be closed; we sent just over 10x the block threshold"
    );
}

/// Feeds will be disconnected if they can't receive messages quickly enough.
#[tokio::test]
async fn e2e_slow_feeds_are_disconnected() {
    let mut server = start_server(
        ServerOpts::default(),
        // Timeout faster so the test can be quicker:
        CoreOpts {
            feed_timeout: Some(1),
            ..Default::default()
        },
        // Allow us to send more messages in more easily:
        ShardOpts {
            max_nodes_per_connection: Some(100_000),
            // Prevent the shard being being banned when it sends a load of data at once:
            max_node_data_per_second: Some(100_000_000),
            ..Default::default()
        },
    )
    .await;

    // Give us a shard to talk to:
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .unwrap();

    // Add a load of nodes from this shard so there's plenty of data to give to a feed.
    // We want to exhaust any buffers between core and feed (eg BufWriters). If the number
    // is too low, data will happily be sent into a buffer and the connection won't need to
    // be closed.
    for n in 1..100_000 {
        node_tx
            .send_json_text(json!({
                "id":n,
                "ts":"2021-07-12T10:37:47.714666+01:00",
                "payload": {
                    "authority":true,
                    "chain":"Polkadot",
                    "config":"",
                    "genesis_hash": polkadot_genesis_hash(), // First party node connections aren't limited.
                    "implementation":"Substrate Node",
                    "msg":"system.connected",
                    "name": format!("Alice {}", n),
                    "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                    "startup_time":"1625565542717",
                    "version":"2.0.0-07a1af348-aarch64-macos"
                }
            }))
            .unwrap();
    }

    // Connect a raw feed so that we can control how fast we consume data from the websocket
    let (mut raw_feed_tx, mut raw_feed_rx) = server.get_core().connect_feed_raw().await.unwrap();

    // Subscribe the feed:
    raw_feed_tx
        .send_text("subscribe:0x0000000000000000000000000000000000000000000000000000000000000001")
        .await
        .unwrap();

    // Wait a little.. the feed hasn't been receiving messages so it should
    // be booted after ~a second.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Drain anything out and expect to hit a "closed" error, rather than get stuck
    // waiting to receive more data (or see some other error).
    loop {
        let mut v = Vec::new();
        let data =
            tokio::time::timeout(Duration::from_secs(2), raw_feed_rx.receive_data(&mut v)).await;

        match data {
            Ok(Ok(_)) => {
                continue; // Drain data
            }
            Ok(Err(soketto::connection::Error::Closed)) => {
                break; // End loop; success!
            }
            Ok(Err(_e)) => {
                // Occasionally we might hit an error here before the channel is marked as closed. The error probably
                // means that the socket has been killed, but we haven't managed to set the state to closed in time
                // and so we still hit this. We may be able to tighten this up and avoid this permanently, at which point
                // this can become a test failure.
                break;
            }
            Err(_) => {
                panic!("recv should be closed but seems to be happy waiting for more data");
            }
        }
    }

    // Tidy up:
    server.shutdown().await;
}

/// If something connects to the `/submit` endpoint, there is a limit to the number
/// of different messags IDs it can send telemetry about, to prevent a malicious actor from
/// spamming a load of message IDs and exhausting our memory.
#[tokio::test]
async fn e2e_max_nodes_per_connection_is_enforced() {
    let mut server = start_server(
        ServerOpts::default(),
        CoreOpts::default(),
        // Limit max nodes per connection to 2; any other msgs should be ignored.
        ShardOpts {
            max_nodes_per_connection: Some(2),
            ..Default::default()
        },
    )
    .await;

    // Connect to a shard
    let shard_id = server.add_shard().await.unwrap();
    let (mut node_tx, _node_rx) = server
        .get_shard(shard_id)
        .unwrap()
        .connect_node()
        .await
        .unwrap();

    // Connect a feed.
    let (feed_tx, mut feed_rx) = server.get_core().connect_feed().await.unwrap();

    // We'll send these messages from the node:
    let json_msg = |n| {
        json!({
            "id":n,
            "ts":"2021-07-12T10:37:47.714666+01:00",
            "payload": {
                "authority":true,
                "chain":"Test Chain",
                "config":"",
                "genesis_hash": ghash(1),
                "implementation":"Polkadot",
                "msg":"system.connected",
                "name": format!("Alice {}", n),
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1625565542717",
                "version":"2.0.0-07a1af348-aarch64-macos"
            }
        })
    };

    // First message ID should lead to feed messages:
    node_tx.send_json_text(json_msg(1)).unwrap();
    assert_ne!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // Second message ID should lead to feed messages as well:
    node_tx.send_json_text(json_msg(2)).unwrap();
    assert_ne!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // Third message ID should be ignored:
    node_tx.send_json_text(json_msg(3)).unwrap();
    assert_eq!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // Forth message ID should be ignored as well:
    node_tx.send_json_text(json_msg(4)).unwrap();
    assert_eq!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // (now that the chain "Test Chain" is known about, subscribe to it for update messages.
    // This wasn't needed to receive messages re the above since everybody hears about node
    // count changes)
    feed_tx
        .send_command(
            "subscribe",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
    feed_rx.recv_feed_messages().await.unwrap();

    // Update about non-ignored IDs should still lead to feed output:

    node_tx.send_json_text(json!(
        {"id":1, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:38:48.330433+01:00" }
    )).unwrap();
    assert_ne!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    node_tx.send_json_text(json!(
        {"id":2, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:38:48.330433+01:00" }
    )).unwrap();
    assert_ne!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // Updates about ignored IDs are still ignored:

    node_tx.send_json_text(json!(
        {"id":3, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:38:48.330433+01:00" }
    )).unwrap();
    assert_eq!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    node_tx.send_json_text(json!(
        {"id":4, "payload":{ "bandwidth_download":576,"bandwidth_upload":576,"msg":"system.interval","peers":1},"ts":"2021-07-12T10:38:48.330433+01:00" }
    )).unwrap();
    assert_eq!(
        feed_rx
            .recv_feed_messages_timeout(Duration::from_secs(1))
            .await
            .unwrap()
            .len(),
        0
    );

    // Tidy up:
    server.shutdown().await;
}
