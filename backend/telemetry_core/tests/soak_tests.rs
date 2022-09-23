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
Soak tests. These are ignored by default, and are intended to be long runs
of the core + shards(s) under different loads to get a feel for CPU/memory
usage and general performance over time.

Note that on MacOS inparticular, you may need to increase some limits to be
able to open a large number of connections. Try commands like:

```sh
sudo sysctl -w kern.maxfiles=100000
sudo sysctl -w kern.maxfilesperproc=100000
ulimit -n 100000
sudo sysctl -w kern.ipc.somaxconn=100000
sudo sysctl -w kern.ipc.maxsockbuf=16777216
```

In general, if you run into issues, it may be better to run this on a linux
box; MacOS seems to hit limits quicker in general.
*/

use common::node_types::BlockHash;
use common::ws_client::SentMessage;
use futures::{future, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use test_utils::workspace::{start_server, CoreOpts, ServerOpts, ShardOpts};

/// A test runner which sends realistic(ish) messages from fake nodes to a telemetry server.
///
/// To start up 4 telemetry_shards and 1 telemetry_core with 10 feeds and 100 nodes:
/// ```sh
/// SOAK_TEST_ARGS='--feeds 10 --nodes 100 --shards 4' cargo test --release -- soak_test --ignored --nocapture
/// ```
///
/// You can also run this test against the pre-sharding actix binary with something like this:
/// ```sh
/// TELEMETRY_BIN=~/old_telemetry_binary SOAK_TEST_ARGS='--feeds 100 --nodes 100 --shards 4' cargo test --release -- soak_test --ignored --nocapture
/// ```
///
/// Or, you can run it against existing processes on the network with something like this:
/// ```sh
/// TELEMETRY_SUBMIT_HOSTS='127.0.0.1:8001' TELEMETRY_FEED_HOST='127.0.0.1:8000' SOAK_TEST_ARGS='--feeds 100 --nodes 100 --shards 4' cargo test --release -- soak_test --ignored --nocapture
/// ```
///
#[ignore]
#[test]
pub fn soak_test() {
    let opts = get_soak_test_opts();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(opts.test_worker_threads)
        .enable_all()
        .thread_name("telemetry_test_runner")
        .build()
        .unwrap()
        .block_on(run_soak_test(opts));
}

/// A general soak test runner.
/// This test sends realistic messages from connected nodes
/// so that we can see how things react under more normal
/// circumstances
async fn run_soak_test(opts: SoakTestOpts) {
    let mut server = start_server(
        ServerOpts {
            release_mode: true,
            log_output: opts.log_output,
        },
        CoreOpts {
            worker_threads: opts.core_worker_threads,
            num_aggregators: opts.core_num_aggregators,
            ..Default::default()
        },
        ShardOpts {
            worker_threads: opts.shard_worker_threads,
            ..Default::default()
        },
    )
    .await;
    println!("Telemetry core running at {}", server.get_core().host());

    // Start up the shards we requested:
    let mut shard_ids = vec![];
    for _ in 0..opts.shards {
        let shard_id = server.add_shard().await.expect("shard can't be added");
        shard_ids.push(shard_id);
    }

    // Connect nodes to each shard for each chain:
    let mut nodes = vec![];
    for chain_name in chain_names(opts.chains) {
        for &shard_id in &shard_ids {
            let conns = server
                .get_shard(shard_id)
                .unwrap()
                .connect_multiple_nodes(opts.nodes)
                .await
                .expect("node connections failed");
            nodes.push((chain_name.clone(), conns));
        }
    }

    let first_genesis_hash = BlockHash::from_low_u64_be(1);
    let first_genesis_hash_string = format!("{:0x}", first_genesis_hash);

    // Start nodes talking to the shards:
    let bytes_in = Arc::new(AtomicUsize::new(0));
    let ids_per_node = opts.ids_per_node;

    // For each chain...
    for (i, (chain_name, conns)) in nodes.into_iter().enumerate() {
        // ...Broadcast an init message from each node with that chain name
        for (j, (tx, _)) in conns.into_iter().enumerate() {
            let idx = i * opts.nodes + j;
            for id in 0..ids_per_node {
                let bytes_in = Arc::clone(&bytes_in);
                let tx = tx.clone();
                let chain_name = chain_name.clone();

                tokio::spawn(async move {
                    let telemetry = test_utils::fake_telemetry::FakeTelemetry {
                        block_time: Duration::from_secs(3),
                        node_name: format!("{} Node {}", chain_name, (ids_per_node * idx) + id + 1),
                        chain: chain_name,
                        genesis_hash: BlockHash::from_low_u64_be((i + 1) as u64),
                        message_id: id + 1,
                    };

                    let res = telemetry
                        .start(|msg| async {
                            bytes_in.fetch_add(msg.len(), Ordering::Relaxed);
                            tx.unbounded_send(SentMessage::Binary(msg))?;
                            Ok::<_, anyhow::Error>(())
                        })
                        .await;

                    if let Err(e) = res {
                        log::error!("Telemetry Node #{} has died with error: {}", idx, e);
                    }
                });
            }
        }
    }

    // Connect feeds to the core:
    let mut feeds = server
        .get_core()
        .connect_multiple_feeds(opts.feeds)
        .await
        .expect("feed connections failed");

    // Every feed subscribes to the first chain we have started up. We ignore the rest.
    for (feed_tx, _) in &mut feeds {
        feed_tx
            .send_command("subscribe", &first_genesis_hash_string)
            .unwrap();
    }

    // Also start receiving messages, counting the bytes received so far.
    let bytes_out = Arc::new(AtomicUsize::new(0));
    let msgs_out = Arc::new(AtomicUsize::new(0));
    for (_, mut feed_rx) in feeds {
        let bytes_out = Arc::clone(&bytes_out);
        let msgs_out = Arc::clone(&msgs_out);
        tokio::task::spawn(async move {
            while let Some(msg) = feed_rx.next().await {
                let msg = msg.expect("message could be received");
                let num_bytes = msg.len();
                bytes_out.fetch_add(num_bytes, Ordering::Relaxed);
                msgs_out.fetch_add(1, Ordering::Relaxed);
            }
            eprintln!("Error: feed has been closed unexpectedly");
        });
    }

    // Periodically report on bytes out
    tokio::task::spawn(async move {
        let one_mb = 1024.0 * 1024.0;
        let mut last_bytes_in = 0;
        let mut last_bytes_out = 0;
        let mut last_msgs_out = 0;
        let mut n = 1;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let bytes_in_val = bytes_in.load(Ordering::Relaxed);
            let bytes_out_val = bytes_out.load(Ordering::Relaxed);
            let msgs_out_val = msgs_out.load(Ordering::Relaxed);

            println!(
                "#{}: MB in/out per measurement: {:.4} / {:.4}, total bytes in/out: {} / {}, msgs out: {}, total msgs out: {})",
                n,
                (bytes_in_val - last_bytes_in) as f64 / one_mb,
                (bytes_out_val - last_bytes_out) as f64 / one_mb,
                bytes_in_val,
                bytes_out_val,
                (msgs_out_val - last_msgs_out),
                msgs_out_val
            );

            n += 1;
            last_bytes_in = bytes_in_val;
            last_bytes_out = bytes_out_val;
            last_msgs_out = msgs_out_val;
        }
    });

    // Wait forever.
    future::pending().await
}

/// Return an iterator of `total` unique chain names.
fn chain_names(total: usize) -> impl Iterator<Item = String> {
    static CHAIN_STARTS: [&str; 5] = ["Polkadot", "Kusama", "Khala", "Wibble", "Moonbase"];
    static CHAIN_ENDS: [&str; 6] = ["", " Testnet", " Main", "-Dev", "Alpha", "Beta"];

    let mut count = 0;
    let mut s_n = 0;
    let mut e_n = 0;

    std::iter::from_fn(move || {
        if count == total {
            return None;
        }

        let mut res = format!("{}{}", CHAIN_STARTS[s_n], CHAIN_ENDS[e_n]);

        let suffix = count / (CHAIN_STARTS.len() * CHAIN_ENDS.len());
        if suffix > 0 {
            res.push(' ');
            res.push_str(&suffix.to_string());
        }

        s_n += 1;
        count += 1;
        if s_n == CHAIN_STARTS.len() {
            s_n = 0;
            e_n += 1;
            if e_n == CHAIN_ENDS.len() {
                e_n = 0;
            }
        }

        Some(res)
    })
}

/// General arguments that are used to start a soak test. Run `soak_test` as
/// instructed by its documentation for full control over what is ran, or run
/// preconfigured variants.
#[derive(StructOpt)]
struct SoakTestOpts {
    /// The number of shards to run this test with
    #[structopt(long)]
    shards: usize,
    /// The number of feeds to connect to the core
    #[structopt(long)]
    feeds: usize,
    /// The number of chains that nodes will pretend to belong to
    #[structopt(long, default_value = "1")]
    chains: usize,
    /// The number of nodes to connect to each shard * chain combo.
    /// If we have 10 chains and 4 shards, setting this to 1 will connect `10 x 4 x 1 = 40` nodes.
    #[structopt(long)]
    nodes: usize,
    /// The number of different virtual nodes to connect per actual node socket connection
    #[structopt(long, default_value = "1")]
    ids_per_node: usize,
    /// Number of aggregator loops to use in the core
    #[structopt(long)]
    core_num_aggregators: Option<usize>,
    /// Number of worker threads the core will use
    #[structopt(long)]
    core_worker_threads: Option<usize>,
    /// Number of worker threads each shard will use
    #[structopt(long)]
    shard_worker_threads: Option<usize>,
    /// Should we log output from the core/shards to stdout?
    #[structopt(long)]
    log_output: bool,
    /// How many worker threads should the soak test runner use?
    #[structopt(long, default_value = "4")]
    test_worker_threads: usize,
}

/// Get soak test args from an envvar and parse them via structopt.
fn get_soak_test_opts() -> SoakTestOpts {
    let arg_string = std::env::var("SOAK_TEST_ARGS")
        .expect("Expecting args to be provided in the env var SOAK_TEST_ARGS");
    let args =
        shellwords::split(&arg_string).expect("Could not parse SOAK_TEST_ARGS as shell arguments");

    // The binary name is expected to be the first arg, so fake it:
    let all_args = std::iter::once("soak_test".to_owned()).chain(args.into_iter());

    SoakTestOpts::from_iter(all_args)
}
