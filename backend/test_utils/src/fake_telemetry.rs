use ::time::{format_description::well_known::Rfc3339, OffsetDateTime};
use common::node_types::BlockHash;
use serde_json::json;
use std::future::Future;
use std::time::Duration;
use tokio::time::{self, MissedTickBehavior};

/// This emits fake but realistic looking telemetry messages.
/// Can be connected to a telemetry server to emit realistic messages.
pub struct FakeTelemetry {
    pub block_time: Duration,
    pub node_name: String,
    pub chain: String,
    pub genesis_hash: BlockHash,
    pub message_id: usize,
}

impl FakeTelemetry {
    /// Begin emitting messages from this node, calling the provided callback each
    /// time a new message is emitted.
    // Unused assignments allowed because macro seems to mess with knowledge of what's
    // been read.
    #[allow(unused_assignments)]
    pub async fn start<Func, Fut, E>(self, mut on_message: Func) -> Result<(), E>
    where
        Func: Send + FnMut(Vec<u8>) -> Fut,
        Fut: Future<Output = Result<(), E>>,
        E: Into<anyhow::Error>,
    {
        let id = self.message_id;
        let name = self.node_name;
        let chain = self.chain;
        let genesis_hash = self.genesis_hash;
        let block_time = self.block_time;

        // Our "state". These numbers can be hashed to give a block hash,
        // and also represent the height of the chain so far. Increment each
        // as needed.
        let mut best_block_n: u64 = 0;
        let mut finalized_block_n: u64 = 0;

        // A helper to send JSON messages without consuming on_message:
        macro_rules! send_msg {
            ($($json:tt)+) => {{
                let msg = json!($($json)+);
                let bytes = serde_json::to_vec(&msg).unwrap();
                on_message(bytes).await
            }}
        }

        // Send system connected immediately
        send_msg!({
            "id":id,
            "payload": {
                "authority":true,
                "chain":chain,
                "config":"",
                "genesis_hash":genesis_hash,
                "implementation":"Substrate Node",
                "msg":"system.connected",
                "name":name,
                "network_id":"12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp",
                "startup_time":"1627986634759",
                "version":"2.0.0-07a1af348-aarch64-macos"
            },
            "ts":now_iso()
        })?;
        best_block_n += 1;

        // First block import immediately (height 1)
        send_msg!({
            "id":id,
            "payload":{
                "best":block_hash(best_block_n),
                "height":best_block_n,
                "msg":"block.import",
                "origin":"Own"
            },
            "ts":now_iso()
        })?;
        best_block_n += 1;

        let now = tokio::time::Instant::now();

        // Configure our message intervals:
        let mut new_block_every = time::interval_at(now + block_time, block_time);
        new_block_every.set_missed_tick_behavior(MissedTickBehavior::Burst);

        let mut system_interval_every =
            time::interval_at(now + Duration::from_secs(2), block_time * 2);
        new_block_every.set_missed_tick_behavior(MissedTickBehavior::Burst);

        let mut finalised_every =
            time::interval_at(now + Duration::from_secs(1) + block_time * 3, block_time);
        new_block_every.set_missed_tick_behavior(MissedTickBehavior::Burst);

        // Send messages every interval:
        loop {
            tokio::select! {
                // Add a new block:
                _ = new_block_every.tick() => {

                    send_msg!({
                        "id":id,
                        "payload":{
                            "hash":"0x918bf5125307b4ac1b2c67aa43ed38517617720ac96cbd5664d7a0f0aa32e1b1", // Don't think this matters
                            "msg":"prepared_block_for_proposing",
                            "number":best_block_n.to_string() // seems to be a string, not a number in the "real" JSON
                        },
                        "ts":now_iso()
                    })?;
                    send_msg!({
                        "id":id,
                        "payload":{
                            "best":block_hash(best_block_n),
                            "height":best_block_n,
                            "msg":"block.import",
                            "origin":"Own"
                        },
                        "ts":now_iso()
                    })?;
                    best_block_n += 1;

                },
                // Periodic updates on system state:
                _ = system_interval_every.tick() => {

                    send_msg!({
                        "id":id,
                        "payload":{
                            "best":block_hash(best_block_n),
                            "finalized_hash":block_hash(finalized_block_n),
                            "finalized_height":finalized_block_n,
                            "height":best_block_n,
                            "msg":"system.interval",
                            "txcount":0,
                            "used_state_cache_size":870775
                        },
                        "ts":now_iso()
                    })?;
                    send_msg!({
                        "id":id,
                        "payload":{
                            "bandwidth_download":0,
                            "bandwidth_upload":0,
                            "msg":"system.interval",
                            "peers":0
                        },
                        "ts":now_iso()
                    })?;

                },
                // Finalise a block:
                _ = finalised_every.tick() => {

                    send_msg!({
                        "id":1,
                        "payload":{
                            "hash":block_hash(finalized_block_n),
                            "msg":"afg.finalized_blocks_up_to",
                            "number":finalized_block_n.to_string(), // string in "real" JSON.
                        },
                        "ts":now_iso()
                    })?;
                    send_msg!({
                        "id":1,
                        "payload":{
                            "best":block_hash(finalized_block_n),
                            "height":finalized_block_n.to_string(), // string in "real" JSON.
                            "msg":"notify.finalized"
                        },
                        "ts":now_iso()
                    })?;
                    finalized_block_n += 1;

                },
            };
        }
    }
}

fn now_iso() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap()
}

/// Spread the u64 across the resulting u256 hash so that it's
/// more visible in the UI.
fn block_hash(n: u64) -> BlockHash {
    let a: [u8; 32] = unsafe { std::mem::transmute([n, n, n, n]) };
    BlockHash::from(a)
}
