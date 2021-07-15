use std::iter::FromIterator;

use futures::StreamExt;
use test_utils::workspace::start_server_release;
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use serde_json::json;
use common::node_types::BlockHash;

pub fn benchmark_throughput_single_shard(c: &mut Criterion) {
    /*
    let rt = Runtime::new().expect("tokio runtime should start");

    // Setup our server and node/feed connections first:
    let (nodes, feeds) = rt.block_on(async {
        let mut server = start_server_release().await;
        let shard_id = server.add_shard().await.unwrap();

        // Connect 1000 nodes to the shard:
        let mut nodes = server
            .get_shard(shard_id)
            .unwrap()
            .connect_multiple(1000)
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
            })).await.unwrap();
        }
tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        // Start 1000 feeds:
        let mut feeds = server
            .get_core()
            .connect_multiple(1)
            .await
            .expect("feeds can connect");

        // // Subscribe all feeds to the chain:
        // for (feed_tx, _) in feeds.iter_mut() {
        //     feed_tx.send_command("subscribe", "Local Testnet").await.unwrap();
        // }

println!("consuming feed");
{

    let mut msgs = futures::stream::FuturesUnordered::from_iter(
        feeds
        .iter_mut()
        .map(|(_,rx)| rx.recv_feed_messages())
    );

    let mut n = 0;
    while let Some(Ok(msg)) = msgs.next().await {
        n += 1;
        println!("Message {}: {:?}", n, msg);
    }
}

        // // Consume any messages feeds have received so far (every feed should havea few at least):
        // let feed_consumers = feeds
        //     .iter_mut()
        //     .map(|(_,rx)| rx.next());
        // futures::future::join_all(feed_consumers).await;
println!("feed consumed");
        (nodes, feeds)
    });

    // Next, run criterion using the same tokio runtime to benchmark time taken to send
    // messages to nodes and receive them from feeds.
    c.bench_function(
        "throughput time",
        |b| b.to_async(&rt).iter(|| async {

            // TODO: Actually implement the benchmark.

        })
    );
    */
}

criterion_group!(benches, benchmark_throughput_single_shard);
criterion_main!(benches);