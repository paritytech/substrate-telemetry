use super::inner_loop;
use crate::find_location::find_location;
use crate::state::NodeId;
use common::id_type;
use futures::channel::mpsc;
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

struct AggregatorInternal {
    /// Shards that connect are each assigned a unique connection ID.
    /// This helps us know who to send messages back to (especially in
    /// conjunction with the [`LocalId`] that messages will come with).
    shard_conn_id: AtomicU64,
    /// Feeds that connect have their own unique connection ID, too.
    feed_conn_id: AtomicU64,
    /// Send messages in to the aggregator from the outside via this. This is
    /// stored here so that anybody holding an `Aggregator` handle can
    /// make use of it.
    tx_to_aggregator: mpsc::UnboundedSender<inner_loop::ToAggregator>,
}

impl Aggregator {
    /// Spawn a new Aggregator. This connects to the telemetry backend
    pub async fn spawn(denylist: Vec<String>) -> anyhow::Result<Aggregator> {
        let (tx_to_aggregator, rx_from_external) = mpsc::unbounded();

        // Kick off a locator task to locate nodes, which hands back a channel to make location requests
        let tx_to_locator = find_location(tx_to_aggregator.clone().with(|(node_id, msg)| {
            future::ok::<_, mpsc::SendError>(inner_loop::ToAggregator::FromFindLocation(
                node_id, msg,
            ))
        }));

        // Handle any incoming messages in our handler loop:
        tokio::spawn(Aggregator::handle_messages(
            rx_from_external,
            tx_to_locator,
            denylist,
        ));

        // Return a handle to our aggregator:
        Ok(Aggregator(Arc::new(AggregatorInternal {
            shard_conn_id: AtomicU64::new(1),
            feed_conn_id: AtomicU64::new(1),
            tx_to_aggregator,
        })))
    }

    // This is spawned into a separate task and handles any messages coming
    // in to the aggregator. If nobody is tolding the tx side of the channel
    // any more, this task will gracefully end.
    async fn handle_messages(
        rx_from_external: mpsc::UnboundedReceiver<inner_loop::ToAggregator>,
        tx_to_aggregator: mpsc::UnboundedSender<(NodeId, Ipv4Addr)>,
        denylist: Vec<String>,
    ) {
        inner_loop::InnerLoop::new(rx_from_external, tx_to_aggregator, denylist)
            .handle()
            .await;
    }

    /// Return a sink that a shard can send messages into to be handled by the aggregator.
    pub fn subscribe_shard(
        &self,
    ) -> impl Sink<inner_loop::FromShardWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let shard_conn_id = self
            .0
            .shard_conn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(inner_loop::ToAggregator::FromShardWebsocket(
                shard_conn_id.into(),
                msg,
            ))
        }))
    }

    /// Return a sink that a feed can send messages into to be handled by the aggregator.
    pub fn subscribe_feed(
        &self,
    ) -> impl Sink<inner_loop::FromFeedWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let feed_conn_id = self
            .0
            .feed_conn_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(inner_loop::ToAggregator::FromFeedWebsocket(
                feed_conn_id.into(),
                msg,
            ))
        }))
    }
}
