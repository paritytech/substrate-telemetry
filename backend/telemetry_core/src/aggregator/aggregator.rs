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

use super::inner_loop;
use crate::find_location::find_location;
use crate::state::NodeId;
use common::id_type;
use futures::{future, Sink, SinkExt};
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

id_type! {
    /// A unique Id is assigned per websocket connection (or more accurately,
    /// per feed socket and per shard socket). This can be combined with the
    /// [`LocalId`] of messages to give us a global ID.
    pub struct ConnId(u64)
}

#[derive(Clone)]
pub struct Aggregator(Arc<AggregatorInternal>);

/// Options to configure the aggregator loop(s)
#[derive(Debug, Clone)]
pub struct AggregatorOpts {
    /// Any node from these chains is muted
    pub denylist: Vec<String>,
    /// If our incoming message queue exceeds this length, we start
    /// dropping non-essential messages.
    pub max_queue_len: usize,
    /// How many nodes from third party chains are allowed to connect
    /// before we prevent connections from them.
    pub max_third_party_nodes: usize,
}

struct AggregatorInternal {
    /// Shards that connect are each assigned a unique connection ID.
    /// This helps us know who to send messages back to (especially in
    /// conjunction with the `ShardNodeId` that messages will come with).
    shard_conn_id: AtomicU64,
    /// Feeds that connect have their own unique connection ID, too.
    feed_conn_id: AtomicU64,
    /// Send messages in to the aggregator from the outside via this. This is
    /// stored here so that anybody holding an `Aggregator` handle can
    /// make use of it.
    tx_to_aggregator: flume::Sender<inner_loop::ToAggregator>,
}

impl Aggregator {
    /// Spawn a new Aggregator. This connects to the telemetry backend
    pub async fn spawn(opts: AggregatorOpts) -> anyhow::Result<Aggregator> {
        let (tx_to_aggregator, rx_from_external) = flume::unbounded();

        // Kick off a locator task to locate nodes, which hands back a channel to make location requests
        let tx_to_locator =
            find_location(tx_to_aggregator.clone().into_sink().with(|(node_id, msg)| {
                future::ok::<_, flume::SendError<_>>(inner_loop::ToAggregator::FromFindLocation(
                    node_id, msg,
                ))
            }));

        // Handle any incoming messages in our handler loop:
        tokio::spawn(Aggregator::handle_messages(
            rx_from_external,
            tx_to_locator,
            opts.max_queue_len,
            opts.denylist,
            opts.max_third_party_nodes,
        ));

        // Return a handle to our aggregator:
        Ok(Aggregator(Arc::new(AggregatorInternal {
            shard_conn_id: AtomicU64::new(1),
            feed_conn_id: AtomicU64::new(1),
            tx_to_aggregator,
        })))
    }

    /// This is spawned into a separate task and handles any messages coming
    /// in to the aggregator. If nobody is holding the tx side of the channel
    /// any more, this task will gracefully end.
    async fn handle_messages(
        rx_from_external: flume::Receiver<inner_loop::ToAggregator>,
        tx_to_aggregator: flume::Sender<(NodeId, Ipv4Addr)>,
        max_queue_len: usize,
        denylist: Vec<String>,
        max_third_party_nodes: usize,
    ) {
        inner_loop::InnerLoop::new(
            tx_to_aggregator,
            denylist,
            max_queue_len,
            max_third_party_nodes,
        )
        .handle(rx_from_external)
        .await;
    }

    /// Gather metrics from our aggregator loop
    pub async fn gather_metrics(&self) -> anyhow::Result<inner_loop::Metrics> {
        let (tx, rx) = flume::unbounded();
        let msg = inner_loop::ToAggregator::GatherMetrics(tx);

        self.0.tx_to_aggregator.send_async(msg).await?;

        let metrics = rx.recv_async().await?;
        Ok(metrics)
    }

    /// Return a sink that a shard can send messages into to be handled by the aggregator.
    pub fn subscribe_shard(
        &self,
    ) -> impl Sink<inner_loop::FromShardWebsocket, Error = anyhow::Error> + Send + Sync + Unpin + 'static
    {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let shard_conn_id = self
            .0
            .shard_conn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.into_sink().with(move |msg| async move {
            Ok(inner_loop::ToAggregator::FromShardWebsocket(
                shard_conn_id.into(),
                msg,
            ))
        }))
    }

    /// Return a sink that a feed can send messages into to be handled by the aggregator.
    pub fn subscribe_feed(
        &self,
    ) -> (
        u64,
        impl Sink<inner_loop::FromFeedWebsocket, Error = anyhow::Error> + Send + Sync + Unpin + 'static,
    ) {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let feed_conn_id = self
            .0
            .feed_conn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        (
            feed_conn_id,
            Box::pin(tx_to_aggregator.into_sink().with(move |msg| async move {
                Ok(inner_loop::ToAggregator::FromFeedWebsocket(
                    feed_conn_id.into(),
                    msg,
                ))
            })),
        )
    }
}
