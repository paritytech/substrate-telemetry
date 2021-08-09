use super::aggregator::Aggregator;
use super::inner_loop;
use futures::{Sink, SinkExt, StreamExt};
use inner_loop::FromShardWebsocket;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use common::EitherSink;

#[derive(Clone)]
pub struct AggregatorSet(Arc<AggregatorSetInner>);

pub struct AggregatorSetInner {
    aggregators: Vec<Aggregator>,
    next_idx: AtomicUsize,
}

impl AggregatorSet {
    /// Spawn the number of aggregators we're asked to.
    pub async fn spawn(
        num_aggregators: usize,
        denylist: Vec<String>,
    ) -> anyhow::Result<AggregatorSet> {
        assert_ne!(num_aggregators, 0, "You must have 1 or more aggregator");

        let aggregators = futures::future::try_join_all(
            (0..num_aggregators).map(|_| Aggregator::spawn(denylist.clone())),
        )
        .await?;

        Ok(AggregatorSet(Arc::new(AggregatorSetInner {
            aggregators,
            next_idx: AtomicUsize::new(0),
        })))
    }

    /// Return a sink that a shard can send messages into to be handled by all aggregators.
    pub fn subscribe_shard(
        &self,
    ) -> impl Sink<inner_loop::FromShardWebsocket, Error = anyhow::Error> + Send + Sync + Unpin + 'static
    {
        // Special case 1 aggregator to avoid the extra indirection and so on
        // if we don't actually need it.
        if self.0.aggregators.len() == 1 {
            let sub = self.0.aggregators[0].subscribe_shard();
            return EitherSink::a(sub)
        }

        let mut conns: Vec<_> = self
            .0
            .aggregators
            .iter()
            .map(|a| a.subscribe_shard())
            .collect();

        // Send every incoming message to all aggregators.
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<FromShardWebsocket>();
        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                for conn in &mut conns {
                    // Unbounded channel under the hood, so this await
                    // shouldn't ever need to yield.
                    if let Err(e) = conn.send(msg.clone()).await {
                        log::error!("Aggregator connection has failed: {}", e);
                        return;
                    }
                }
            }
        });

        EitherSink::b(tx.sink_map_err(|e| anyhow::anyhow!("{}", e)))
    }

    /// Return a sink that a feed can send messages into to be handled by a single aggregator.
    pub fn subscribe_feed(
        &self,
    ) -> (
        u64,
        impl Sink<inner_loop::FromFeedWebsocket, Error = anyhow::Error> + Send + Sync + Unpin + 'static,
    ) {
        let last_val = self.0.next_idx.fetch_add(1, Ordering::Relaxed);
        let this_idx = (last_val + 1) % self.0.aggregators.len();

        self.0.aggregators[this_idx].subscribe_feed()
    }
}
