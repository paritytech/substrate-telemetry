use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::net::IpAddr;
use std::sync::{ Mutex, Arc };

/// Keep track of nodes that have been blocked.
#[derive(Debug, Clone)]
pub struct BlockedAddrs(Arc<BlockAddrsInner>);

#[derive(Debug)]
struct BlockAddrsInner {
    block_duration: Duration,
    inner: Mutex<HashMap<IpAddr, (&'static str, Instant)>>
}

impl BlockedAddrs {
    /// Create a new block list. Nodes are blocked for the duration
    /// provided here.
    pub fn new(block_duration: Duration) -> BlockedAddrs {
        BlockedAddrs(Arc::new(BlockAddrsInner {
            block_duration,
            inner: Mutex::new(HashMap::new())
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
            Some(&(reason,time)) => (reason, time),
            None => return None
        };

        if time + self.0.block_duration < Instant::now() {
            map.remove(addr);
            None
        } else {
            Some(reason)
        }
    }
}