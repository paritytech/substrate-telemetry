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

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Keep track of nodes that have been blocked.
#[derive(Debug, Clone)]
pub struct BlockedAddrs(Arc<BlockAddrsInner>);

#[derive(Debug)]
struct BlockAddrsInner {
    block_duration: Duration,
    inner: Mutex<HashMap<IpAddr, (&'static str, Instant)>>,
}

impl BlockedAddrs {
    /// Create a new block list. Nodes are blocked for the duration
    /// provided here.
    pub fn new(block_duration: Duration) -> BlockedAddrs {
        BlockedAddrs(Arc::new(BlockAddrsInner {
            block_duration,
            inner: Mutex::new(HashMap::new()),
        }))
    }

    /// Block a new address
    pub fn block_addr(&self, addr: IpAddr, reason: &'static str) {
        let now = Instant::now();
        self.0.inner.lock().unwrap().insert(addr, (reason, now));
    }

    /// Find out whether an address has been blocked. If it has, a reason
    /// will be returned. Else, we'll get None back. This function may also
    /// perform cleanup if the item was blocked and the block has expired.
    pub fn blocked_reason(&self, addr: &IpAddr) -> Option<&'static str> {
        let mut map = self.0.inner.lock().unwrap();

        let (reason, time) = match map.get(addr) {
            Some(&(reason, time)) => (reason, time),
            None => return None,
        };

        if time + self.0.block_duration < Instant::now() {
            map.remove(addr);
            None
        } else {
            Some(reason)
        }
    }
}
