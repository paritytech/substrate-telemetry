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

/// Create/connect to a server consisting of shards and a core process that we can interact with.
pub mod server;

/// Test support for deserializing feed messages from the feed processes. This basically
/// is the slightly-lossy inverse of the custom serialization we do to feed messages.
pub mod feed_message_de;

/// A couple of macros to make it easier to test for the presence of things (mainly, feed messages)
/// in an iterable container.
#[macro_use]
pub mod contains_matches;

/// Utilities to help with running tests from within this current workspace.
pub mod workspace;

/// A utility to generate fake telemetry messages at realistic intervals.
pub mod fake_telemetry;
