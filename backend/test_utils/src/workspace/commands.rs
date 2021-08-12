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

//! Commands that we can use when running `cargo test` style tests in this workspace
//! that want to test the current code.
use crate::server::Command;
use std::path::PathBuf;

/// Runs `cargo run` in the current workspace to start up a telemetry shard process.
///
/// Note: The CWD must be somewhere within this backend workspace for the command to work.
pub fn cargo_run_telemetry_shard(release_mode: bool) -> Result<Command, std::io::Error> {
    telemetry_command("telemetry_shard", release_mode)
}

/// Runs `cargo run` in the current workspace to start up a telemetry core process.
///
/// Note: The CWD must be somewhere within this backend workspace for the command to work.
pub fn cargo_run_telemetry_core(release_mode: bool) -> Result<Command, std::io::Error> {
    telemetry_command("telemetry_core", release_mode)
}

fn telemetry_command(bin: &'static str, release_mode: bool) -> Result<Command, std::io::Error> {
    let mut workspace_dir = try_find_workspace_dir()?;
    workspace_dir.push("Cargo.toml");

    let mut cmd = Command::new("cargo").arg("run");

    // Release mode?
    if release_mode {
        cmd = cmd.arg("--release");
    }

    cmd = cmd
        .arg("--bin")
        .arg(bin)
        .arg("--manifest-path")
        .arg(workspace_dir)
        .arg("--");

    Ok(cmd)
}

/// A _very_ naive way to find the workspace ("backend") directory
/// from the current path (which is assumed to be inside it).
fn try_find_workspace_dir() -> Result<PathBuf, std::io::Error> {
    let mut dir = std::env::current_dir()?;
    while !dir.ends_with("backend") && dir.pop() {}
    Ok(dir)
}
