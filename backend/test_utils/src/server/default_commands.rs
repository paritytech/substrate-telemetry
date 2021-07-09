use super::Command;
use std::path::PathBuf;

pub fn default_telemetry_shard_command() -> Result<Command, std::io::Error> {
    default_telemetry_command("telemetry_shard")
}

pub fn default_telemetry_core_command() -> Result<Command, std::io::Error> {
    default_telemetry_command("telemetry_core")
}

fn default_telemetry_command(bin: &'static str) -> Result<Command, std::io::Error> {
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