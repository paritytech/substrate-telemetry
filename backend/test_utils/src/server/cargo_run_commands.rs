//! A pair of commands we can use when running `cargo test` style tests in this workspace
//! that want to test the current code. For more external tests, we may want to ask for the
//! commands, or connect to a running instance instead.

use super::Command;
use std::path::PathBuf;

/// Runs `cargo run` in the current workspace to start up a telemetry shard process
pub fn telemetry_shard() -> Result<Command, std::io::Error> {
    telemetry_command("telemetry_shard")
}

/// Runs `cargo run` in the current workspace to start up a telemetry core process
pub fn telemetry_core() -> Result<Command, std::io::Error> {
    telemetry_command("telemetry_core")
}

fn telemetry_command(bin: &'static str) -> Result<Command, std::io::Error> {
    let mut workspace_dir = try_find_workspace_dir()?;
    workspace_dir.push("Cargo.toml");
    Ok(Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg(bin)
        .arg("--manifest-path")
        .arg(workspace_dir)
        .arg("--"))
}

/// A _very_ naive way to find the workspace ("backend") directory
/// from the current path (which is assumed to be inside it).
fn try_find_workspace_dir() -> Result<PathBuf, std::io::Error> {
    let mut dir = std::env::current_dir()?;
    while !dir.ends_with("backend") && dir.pop() {}
    Ok(dir)
}