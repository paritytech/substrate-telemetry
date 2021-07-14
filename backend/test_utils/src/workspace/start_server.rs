use super::commands;
use crate::server::{self, Server, Command};

/// Start a telemetry core server. We'll use `cargo run` by default, to ensure that
/// the code we run is uptodate, but you can also provide env vars to configure the binary
/// that runs for the shard and core process:
///
/// TELEMETRY_SHARD_BIN - path to telemetry_shard binary
/// TELEMETRY_CORE_BIN - path to telemetry_core binary
async fn start_server(release_mode: bool) -> Server {
    let shard_command = std::env::var("TELEMETRY_SHARD_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| commands::cargo_run_telemetry_shard(release_mode).expect("valid shard command"));

    let core_command = std::env::var("TELEMETRY_CORE_BIN")
        .map(|val| Command::new(val))
        .unwrap_or_else(|_| commands::cargo_run_telemetry_core(release_mode).expect("valid core command"));

    Server::start(server::StartOpts { shard_command, core_command }).await.unwrap()
}

/// Start a telemetry server using debug builds for compile speed
pub async fn start_server_debug() -> Server {
    start_server(false).await
}

/// Start a telemetry server using release builds for performance accuracy
pub async fn start_server_release() -> Server {
    start_server(true).await
}