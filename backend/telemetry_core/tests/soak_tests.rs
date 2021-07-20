/*!
Soak tests. These are ignored by default, and are intended to be long runs
of the core + shards(s) under different loads to get a feel for CPU/memory
usage and general performance over time.

Note that on MacOS inparticular, you may need to increase some limits to be
able to open a large number of connections. Try commands like:

```sh
sudo sysctl -w kern.maxfiles=50000
sudo sysctl -w kern.maxfilesperproc=50000
ulimit -n 50000
sudo sysctl -w kern.ipc.somaxconn=50000
sudo sysctl -w kern.ipc.maxsockbuf=16777216
```

In general, if you run into issues, it may be better to run this on a linux
box; MacOS seems to hit limits quicker in general.
*/

use futures::{ StreamExt };
use structopt::StructOpt;
use test_utils::workspace::start_server_release;
use common::ws_client::{ SentMessage };
use serde_json::json;
use std::time::Duration;
use std::sync::atomic::{ Ordering, AtomicUsize };
use std::sync::Arc;
use common::node_types::BlockHash;

/// A configurable soak_test runner. Configure by providing the expected args as
/// an environment variable. One example to run this test is:
///
/// ```sh
/// SOAK_TEST_ARGS='--feeds 10 --nodes 100 --shards 4' cargo test -- soak_test --ignored --nocapture
/// ```
///
/// You can also run this test against the pre-sharding actix binary with something like this:
/// ```sh
/// TELEMETRY_BIN=~/old_telemetry_binary SOAK_TEST_ARGS='--feeds 100 --nodes 100 --shards 4' cargo test -- soak_test --ignored --nocapture
/// ```
///
/// Or, you can run it against existing processes with something like this:
/// ```sh
/// TELEMETRY_SUBMIT_HOSTS='127.0.0.1:8001' TELEMETRY_FEED_HOST='127.0.0.1:8000' SOAK_TEST_ARGS='--feeds 100 --nodes 100 --shards 4' cargo test -- soak_test --ignored --nocapture
/// ```
///
/// Each will establish the same total number of connections and send the same messages.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn soak_test() {
    let opts = get_soak_test_opts();
    run_soak_test(opts).await;
}

/// The general soak test runner. This is called by tests.
async fn run_soak_test(opts: SoakTestOpts) {
    let mut server = start_server_release().await;

    // Start up the shards we requested:
    let mut shard_ids = vec![];
    for _ in 0..opts.shards {
        let shard_id = server.add_shard().await.expect("shard can't be added");
        shard_ids.push(shard_id);
    }

    // Connect nodes to each shard:
    let mut nodes = vec![];
    for &shard_id in &shard_ids {
        let mut conns = server
            .get_shard(shard_id)
            .unwrap()
            .connect_multiple_nodes(opts.nodes)
            .await
            .expect("node connections failed");
        nodes.append(&mut conns);
    }

    // Each node tells the shard about itself:
    for (idx, (node_tx, _)) in nodes.iter_mut().enumerate() {
        node_tx.send_json_binary(json!({
            "id":1, // Only needs to be unique per node
            "ts":"2021-07-12T10:37:47.714666+01:00",
            "payload": {
                "authority":true,
                "chain": "Polkadot", // <- so that we don't go over quota with lots of nodes.
                "config":"",
                "genesis_hash": BlockHash::from_low_u64_ne(1),
                "implementation":"Substrate Node",
                "msg":"system.connected",
                "name": format!("Node #{}", idx),
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1625565542717",
                "version":"2.0.0-07a1af348-aarch64-macos"
            },
        })).unwrap();
    }

    // Connect feeds to the core:
    let mut feeds = server
        .get_core()
        .connect_multiple_feeds(opts.feeds)
        .await
        .expect("feed connections failed");

    // Every feed subscribes to the chain above to recv messages about it:
    for (feed_tx, _) in &mut feeds {
        feed_tx.send_command("subscribe", "Polkadot").unwrap();
    }

    // Start sending "update" messages from nodes at time intervals.
    let bytes_in = Arc::new(AtomicUsize::new(0));
    let bytes_in2 = Arc::clone(&bytes_in);
    let send_handle = tokio::task::spawn(async move {
        let msg = json!({
            "id":1,
            "payload":{
                "bandwidth_download":576,
                "bandwidth_upload":576,
                "msg":"system.interval",
                "peers":1
            },
            "ts":"2021-07-12T10:37:48.330433+01:00"
        });
        let msg_bytes: &'static [u8] = Box::new(serde_json::to_vec(&msg).unwrap()).leak();

        loop {
            // every ~1second we aim to have sent messages from all of the nodes. So we cycle through
            // the node IDs and send a message from each at roughly 1s / number_of_nodes.
            let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / nodes.len() as f64));

            for node_id in (0..nodes.len()).cycle() {
                interval.tick().await;
                let node_tx = &mut nodes[node_id].0;
                node_tx.unbounded_send(SentMessage::StaticBinary(msg_bytes)).unwrap();
                bytes_in2.fetch_add(msg_bytes.len(), Ordering::Relaxed);
            }
        }
    });

    // Also start receiving messages, counting the bytes received so far.
    let bytes_out = Arc::new(AtomicUsize::new(0));
    for (_, mut feed_rx) in feeds {
        let bytes_out = Arc::clone(&bytes_out);
        tokio::task::spawn(async move {
            while let Some(msg) = feed_rx.next().await {
                let msg = msg.expect("message could be received");
                let num_bytes = msg.len();
                bytes_out.fetch_add(num_bytes, Ordering::Relaxed);
            }
        });
    }

    // Periodically report on bytes out
    tokio::task::spawn(async move {
        let one_mb = 1024.0 * 1024.0;
        let mut last_bytes_in = 0;
        let mut last_bytes_out = 0;
        let mut n = 1;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let bytes_in_val = bytes_in.load(Ordering::Relaxed);
            let bytes_out_val = bytes_out.load(Ordering::Relaxed);

            println!("#{}: MB in/out per measurement: {:.4} / {:.4}, total bytes in/out: {} / {})",
                n,
                (bytes_in_val - last_bytes_in) as f64 / one_mb,
                (bytes_out_val - last_bytes_out) as f64 / one_mb,
                bytes_in_val,
                bytes_out_val
            );

            n += 1;
            last_bytes_in = bytes_in_val;
            last_bytes_out = bytes_out_val;
        }
    });

    // Wait for sending to finish before ending.
    send_handle.await.unwrap();
}

/// General arguments that are used to start a soak test. Run `soak_test` as
/// instructed by its documentation for full control over what is ran, or run
/// preconfigured variants.
#[derive(StructOpt, Debug)]
struct SoakTestOpts {
    /// The number of shards to run this test with
    #[structopt(long)]
    shards: usize,
    /// The number of feeds to connect
    #[structopt(long)]
    feeds: usize,
    /// The number of nodes to connect to each feed
    #[structopt(long)]
    nodes: usize
}

/// Get soak test args from an envvar and parse them via structopt.
fn get_soak_test_opts() -> SoakTestOpts {
    let arg_string = std::env::var("SOAK_TEST_ARGS")
        .expect("Expecting args to be provided in the env var SOAK_TEST_ARGS");
    let args = shellwords::split(&arg_string)
        .expect("Could not parse SOAK_TEST_ARGS as shell arguments");

    // The binary name is expected to be the first arg, so fake it:
    let all_args = std::iter::once("soak_test".to_owned())
        .chain(args.into_iter());

    SoakTestOpts::from_iter(all_args)
}