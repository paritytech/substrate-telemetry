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

/// Functionality to establish a connection
mod connect;
/// A close helper that we use in sender/receiver.
mod on_close;
/// The channel based receive interface
mod receiver;
/// The channel based send interface
mod sender;

pub use connect::{connect, ConnectError, Connection, RawReceiver, RawSender};
pub use receiver::{Receiver, RecvError, RecvMessage};
pub use sender::{SendError, Sender, SentMessage};
