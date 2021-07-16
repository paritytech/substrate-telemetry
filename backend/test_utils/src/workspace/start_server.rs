use super::commands;
use crate::server::{self, Server, Command};

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
/// Whatever is not provided will be substituted with a `cargo run` variant instead.
pub async fn start_server(release_mode: bool) -> Server {

    if let Ok(bin) = std::env::var("TELEMETRY_BIN") {
        return Server::start(server::StartOpts::SingleProcess {
            command: Command::new(bin)
        }).await.unwrap();
    }

    let shard_command = std::env::var("TELEMETRY_SHARD_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| commands::cargo_run_telemetry_shard(release_mode).expect("must be in rust workspace to run shard command"));

    let core_command = std::env::var("TELEMETRY_CORE_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| commands::cargo_run_telemetry_core(release_mode).expect("must be in rust workspace to run core command"));

    Server::start(server::StartOpts::ShardAndCore {
        shard_command,
        core_command
    }).await.unwrap()
}

/// Start a telemetry core server in debug mode. see [`start_server`] for details.
pub async fn start_server_debug() -> Server {
    start_server(false).await
}

/// Start a telemetry core server in release mode. see [`start_server`] for details.
pub async fn start_server_release() -> Server {
    start_server(true).await
}