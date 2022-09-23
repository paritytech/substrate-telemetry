use common::node_types::BlockHash;
use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::time::{Duration, Instant};
use test_utils::feed_message_de::FeedMessage;
use test_utils::workspace::{start_server, CoreOpts, ServerOpts, ShardOpts};
use tokio::runtime::Runtime;

/// This benchmark roughly times the subscribe function. Note that there's a lot of
/// overhead in other areas, so even with the entire subscribe function commented out
/// By benchmark timings are ~50ms (whereas they are ~320ms with the version of the
/// subscribe handler at the time of writing).
///
/// If you want to use this benchmark, it's therefore worth commenting out the subscribe
/// logic entirely and running this to give yourself a "baseline".
pub fn benchmark_subscribe_speed(c: &mut Criterion) {
    const NUMBER_OF_FEEDS: usize = 100;
    const NUMBER_OF_NODES: usize = 10_000;

    let rt = Runtime::new().expect("tokio runtime should start");

    c.bench_function("subscribe speed: time till pong", move |b| {
        b.to_async(&rt).iter_custom(|iters| async move {
            // Now, see how quickly a feed is subscribed. Criterion controls the number of
            // iters performed here, but a lot of the time that number is "1".
            let mut total_time = Duration::ZERO;
            for _n in 0..iters {
                // Start a server:
                let mut server = start_server(
                    ServerOpts {
                        release_mode: true,
                        log_output: false,
                    },
                    CoreOpts {
                        worker_threads: Some(16),
                        num_aggregators: Some(1),
                        ..Default::default()
                    },
                    ShardOpts {
                        max_nodes_per_connection: Some(usize::MAX),
                        max_node_data_per_second: Some(usize::MAX),
                        worker_threads: Some(2),
                        ..Default::default()
                    },
                )
                .await;
                let shard_id = server.add_shard().await.unwrap();

                // Connect a shard:
                let (mut node_tx, _) = server
                    .get_shard(shard_id)
                    .unwrap()
                    .connect_node()
                    .await
                    .expect("node can connect");

                // Add a bunch of actual nodes on the same chain:
                for n in 0..NUMBER_OF_NODES {
                    node_tx
                        .send_json_text(json!({
                            "id":n,
                            "ts":"2021-07-12T10:37:47.714666+01:00",
                            "payload": {
                                "authority":true,
                                "chain":"Polkadot", // No limit to #nodes on this network.
                                "config":"",
                                "genesis_hash": BlockHash::from_low_u64_ne(1),
                                "implementation":"Substrate Node",
                                "msg":"system.connected",
                                "name": format!("Node {}", n),
                                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                                "startup_time":"1625565542717",
                                "version":"2.0.0-07a1af348-aarch64-macos"
                            }
                        }))
                        .unwrap();
                }

                // Give those messages a chance to be handled. This, of course,
                // assumes that those messages _can_ be handled this quickly. If not,
                // we'll start to skew benchmark results with the "time taken to add node".
                tokio::time::sleep(Duration::from_millis(250)).await;

                // Start a bunch of feeds:
                let mut feeds = server
                    .get_core()
                    .connect_multiple_feeds(NUMBER_OF_FEEDS)
                    .await
                    .expect("feeds can connect");

                // Subscribe every feed to the chain:
                for (feed_tx, _) in feeds.iter() {
                    feed_tx.send_command("subscribe", "Polkadot").unwrap();
                }

                // Then, Ping a feed:
                feeds[0].0.send_command("ping", "Finished!").unwrap();
                let finished = FeedMessage::Pong {
                    msg: "Finished!".to_owned(),
                };

                // Wait and see how long it takes to get a pong back:
                let start = Instant::now();
                loop {
                    let msgs = feeds[0].1.recv_feed_messages_once().await.unwrap();
                    if msgs.iter().any(|m| m == &finished) {
                        break;
                    }
                }
                total_time += start.elapsed();
            }

            // The total time spent waiting for subscribes:
            total_time
        })
    });
}

criterion_group!(benches, benchmark_subscribe_speed);
criterion_main!(benches);
