use super::aggregator::{Aggregator, AggregatorOpts};
use super::inner_loop;
use common::EitherSink;
use futures::{Sink, SinkExt};
use inner_loop::{FromShardWebsocket, Metrics};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AggregatorSet(Arc<AggregatorSetInner>);

pub struct AggregatorSetInner {
    aggregators: Vec<Aggregator>,
    next_idx: AtomicUsize,
    metrics: Mutex<Vec<Metrics>>,
}

impl AggregatorSet {
    /// Spawn the number of aggregators we're asked to.
    pub async fn spawn(
        num_aggregators: usize,
        opts: AggregatorOpts,
    ) -> anyhow::Result<AggregatorSet> {
        assert_ne!(num_aggregators, 0, "You must have 1 or more aggregator");

        let aggregators = futures::future::try_join_all(
            (0..num_aggregators).map(|_| Aggregator::spawn(opts.clone())),
        )
        .await?;

        let initial_metrics = (0..num_aggregators).map(|_| Metrics::default()).collect();

        let this = AggregatorSet(Arc::new(AggregatorSetInner {
            aggregators,
            next_idx: AtomicUsize::new(0),
            metrics: Mutex::new(initial_metrics),
        }));

        // Start asking for metrics:
        this.spawn_metrics_loops();

        Ok(this)
    }

    /// Spawn loops which periodically ask for metrics from each internal aggregator.
    /// Depending on how busy the aggregators are, these metrics won't necessarily be in
    /// sync with each other.
    fn spawn_metrics_loops(&self) {
        let aggregators = self.0.aggregators.clone();
        for (idx, a) in aggregators.into_iter().enumerate() {
            let inner = Arc::clone(&self.0);
            tokio::spawn(async move {
                loop {
                    let now = tokio::time::Instant::now();
                    let metrics = match a.gather_metrics().await {
                        Ok(metrics) => metrics,
                        // Any error here is unlikely and probably means that the aggregator
                        // loop has failed completely.
                        Err(e) => {
                            log::error!("Error obtaining metrics (bailing): {}", e);
                            return;
                        }
                    };

                    // Lock, update the stored metrics and drop the lock immediately.
                    // We discard any error; if something went wrong talking to the inner loop,
                    // it's probably a fatal error
                    {
                        inner.metrics.lock().unwrap()[idx] = metrics;
                    }

                    // Sleep *at least* 10 seconds. If it takes a while to get metrics back, we'll
                    // end up waiting longer between requests.
                    tokio::time::sleep_until(now + tokio::time::Duration::from_secs(10)).await;
                }
            });
        }
    }

    /// Return the latest metrics we've gathered so far from each internal aggregator.
    pub fn latest_metrics(&self) -> Vec<Metrics> {
        self.0.metrics.lock().unwrap().clone()
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
            return EitherSink::a(sub);
        }

        let mut conns: Vec<_> = self
            .0
            .aggregators
            .iter()
            .map(|a| a.subscribe_shard())
            .collect();

        let (tx, rx) = flume::unbounded::<FromShardWebsocket>();

        // Send every incoming message to all aggregators.
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv_async().await {
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

        EitherSink::b(tx.into_sink().sink_map_err(|e| anyhow::anyhow!("{}", e)))
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
