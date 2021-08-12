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

use super::commands;
use crate::server::{self, Command, Server};

/// Options for the server
pub struct ServerOpts {
    pub release_mode: bool,
    pub log_output: bool,
}

impl Default for ServerOpts {
    fn default() -> Self {
        Self {
            release_mode: false,
            log_output: false,
        }
    }
}

/// Additional options to pass to the core command.
pub struct CoreOpts {
    pub feed_timeout: Option<u64>,
    pub worker_threads: Option<usize>,
    pub num_aggregators: Option<usize>,
}

impl Default for CoreOpts {
    fn default() -> Self {
        Self {
            feed_timeout: None,
            worker_threads: None,
            num_aggregators: None,
        }
    }
}

/// Additional options to pass to the shard command.
pub struct ShardOpts {
    pub max_nodes_per_connection: Option<usize>,
    pub max_node_data_per_second: Option<usize>,
    pub node_block_seconds: Option<u64>,
    pub worker_threads: Option<usize>,
}

impl Default for ShardOpts {
    fn default() -> Self {
        Self {
            max_nodes_per_connection: None,
            max_node_data_per_second: None,
            node_block_seconds: None,
            worker_threads: None,
        }
    }
}

/// Start a telemetry server. We'll use `cargo run` by default, but you can also provide
/// env vars to configure the binary that runs for the shard and core process. Either:
///
/// - `TELEMETRY_BIN` - path to the telemetry binary (which can function as shard _and_ core)
///
/// Or alternately neither/one/both of:
///
/// - `TELEMETRY_SHARD_BIN` - path to telemetry_shard binary
/// - `TELEMETRY_CORE_BIN` - path to telemetry_core binary
///
/// (Whatever is not provided will be substituted with a `cargo run` variant instead)
///
/// Or alternately alternately, we can connect to a running instance by providing:
///
/// - `TELEMETRY_SUBMIT_HOSTS` - hosts (comma separated) to connect to for telemetry `/submit`s.
/// - `TELEMETRY_FEED_HOST` - host to connect to for feeds (eg 127.0.0.1:3000)
///
pub async fn start_server(
    server_opts: ServerOpts,
    core_opts: CoreOpts,
    shard_opts: ShardOpts,
) -> Server {
    // Start to a single process:
    if let Ok(bin) = std::env::var("TELEMETRY_BIN") {
        return Server::start(server::StartOpts::SingleProcess {
            command: Command::new(bin),
            log_output: server_opts.log_output,
        })
        .await
        .unwrap();
    }

    // Connect to a running instance:
    if let Ok(feed_host) = std::env::var("TELEMETRY_FEED_HOST") {
        let feed_host = feed_host.trim().into();
        let submit_hosts: Vec<_> = std::env::var("TELEMETRY_SUBMIT_HOSTS")
            .map(|var| var.split(",").map(|var| var.trim().into()).collect())
            .unwrap_or(Vec::new());
        return Server::start(server::StartOpts::ConnectToExisting {
            feed_host,
            submit_hosts,
            log_output: server_opts.log_output,
        })
        .await
        .unwrap();
    }

    // Build the shard command
    let mut shard_command = std::env::var("TELEMETRY_SHARD_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| {
            commands::cargo_run_telemetry_shard(server_opts.release_mode)
                .expect("must be in rust workspace to run shard command")
        });

    // Append additional opts to the shard command
    if let Some(val) = shard_opts.max_nodes_per_connection {
        shard_command = shard_command
            .arg("--max-nodes-per-connection")
            .arg(val.to_string());
    }
    if let Some(val) = shard_opts.max_node_data_per_second {
        shard_command = shard_command
            .arg("--max-node-data-per-second")
            .arg(val.to_string());
    }
    if let Some(val) = shard_opts.node_block_seconds {
        shard_command = shard_command
            .arg("--node-block-seconds")
            .arg(val.to_string());
    }
    if let Some(val) = shard_opts.worker_threads {
        shard_command = shard_command.arg("--worker-threads").arg(val.to_string());
    }

    // Build the core command
    let mut core_command = std::env::var("TELEMETRY_CORE_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| {
            commands::cargo_run_telemetry_core(server_opts.release_mode)
                .expect("must be in rust workspace to run core command")
        });

    // Append additional opts to the core command
    if let Some(val) = core_opts.feed_timeout {
        core_command = core_command.arg("--feed-timeout").arg(val.to_string());
    }
    if let Some(val) = core_opts.worker_threads {
        core_command = core_command.arg("--worker-threads").arg(val.to_string());
    }
    if let Some(val) = core_opts.num_aggregators {
        core_command = core_command.arg("--num-aggregators").arg(val.to_string());
    }

    // Start the server
    Server::start(server::StartOpts::ShardAndCore {
        shard_command,
        core_command,
        log_output: server_opts.log_output,
    })
    .await
    .unwrap()
}

/// Start a telemetry core server in debug mode. see [`start_server`] for details.
pub async fn start_server_debug() -> Server {
    start_server(
        ServerOpts::default(),
        CoreOpts::default(),
        ShardOpts::default(),
    )
    .await
}

/// Start a telemetry core server in release mode. see [`start_server`] for details.
pub async fn start_server_release() -> Server {
    start_server(
        ServerOpts::default(),
        CoreOpts::default(),
        ShardOpts::default(),
    )
    .await
}
