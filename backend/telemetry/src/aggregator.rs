use common::{internal_messages::{GlobalId, LocalId}, node, assign_id::AssignId};
use std::{str::FromStr, sync::Arc};
use std::sync::atomic::AtomicU64;
use futures::channel::{ mpsc, oneshot };
use futures::{ Sink, SinkExt, StreamExt };
use tokio::net::TcpStream;
use tokio_util::compat::{ TokioAsyncReadCompatExt };
use std::collections::{ HashMap, HashSet };
use crate::state::State;

/// A unique Id is assigned per websocket connection (or more accurately,
/// per feed socket and per shard socket). This can be combined with the
/// [`LocalId`] of messages to give us a global ID.
type ConnId = u64;

/// Incoming messages come via subscriptions, and end up looking like this.
#[derive(Debug)]
enum ToAggregator {
    FromShardWebsocket(ConnId, FromShardWebsocket),
    FromFeedWebsocket(ConnId, FromFeedWebsocket),
}

/// An incoming shard connection can send these messages to the aggregator.
#[derive(Debug)]
pub enum FromShardWebsocket {
    /// When the socket is opened, it'll send this first
    /// so that we have a way to communicate back to it.
    Initialize {
        channel: mpsc::Sender<ToShardWebsocket>,
    },
    /// Tell the aggregator about a new node.
    Add {
        local_id: LocalId,
        ip: Option<std::net::IpAddr>,
        node: common::types::NodeDetails,
    },
    /// Update/pass through details about a node.
    Update {
        local_id: LocalId,
        payload: node::Payload
    }
}

/// The aggregator can these messages back to a shard connection.
#[derive(Debug)]
pub enum ToShardWebsocket {
    /// Mute messages to the core by passing the shard-local ID of them.
    Mute {
        local_id: LocalId
    }
}

/// An incoming feed connection can send these messages to the aggregator.
#[derive(Debug)]
pub enum FromFeedWebsocket {
    /// When the socket is opened, it'll send this first
    /// so that we have a way to communicate back to it.
    Initialize {
        channel: mpsc::Sender<ToFeedWebsocket>,
    },
    /// The feed can subscribe to a chain to receive
    /// messages relating to it.
    Subscribe {
        chain: Box<str>
    },
    /// The feed wants finality info for the chain, too.
    SendFinality {
        chain: Box<str>
    },
    /// The feed doesn't want any more finality info for the chain.
    NoMoreFinality {
        chain: Box<str>
    },
    /// An explicit ping message.
    Ping {
        chain: Box<str>
    }
}

// The frontend sends text based commands; parse them into these messages:
impl FromStr for FromFeedWebsocket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, chain) = match s.find(':') {
            Some(idx) => (&s[..idx], s[idx+1..].into()),
            None => return Err(anyhow::anyhow!("Expecting format `CMD:CHAIN_NAME`"))
        };
        match cmd {
            "ping" => Ok(FromFeedWebsocket::Ping { chain }),
            "subscribe" => Ok(FromFeedWebsocket::Subscribe { chain }),
            "send-finality" => Ok(FromFeedWebsocket::SendFinality { chain }),
            "no-more-finality" => Ok(FromFeedWebsocket::NoMoreFinality { chain }),
            _ => return Err(anyhow::anyhow!("Command {} not recognised", cmd))
        }
    }
}

/// The aggregator can these messages back to a feed connection.
#[derive(Debug)]
pub enum ToFeedWebsocket {

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
    tx_to_aggregator: mpsc::Sender<ToAggregator>
}

impl Aggregator {
    /// Spawn a new Aggregator. This connects to the telemetry backend
    pub async fn spawn(denylist: Vec<String>) -> anyhow::Result<Aggregator> {
        let (tx_to_aggregator, rx_from_external) = mpsc::channel(10);

        // Handle any incoming messages in our handler loop:
        tokio::spawn(Aggregator::handle_messages(rx_from_external, denylist));

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
    async fn handle_messages(mut rx_from_external: mpsc::Receiver<ToAggregator>, denylist: Vec<String>) {

        let mut nodes_state = State::new();

        // Maintain mappings from the shard connection ID and local ID of messages to a global ID
        // that uniquely identifies nodes in our node state.
        let mut to_global_id = AssignId::new();

        // Temporary: if we drop channels to shards, they will be booted:
        let mut to_shards = vec![];

        // Now, loop and receive messages to handle.
        while let Some(msg) = rx_from_external.next().await {
            match msg {
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Initialize { channel }) => {

                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Ping { chain }) => {

                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::Subscribe { chain }) => {

                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::SendFinality { chain }) => {

                },
                ToAggregator::FromFeedWebsocket(feed_conn_id, FromFeedWebsocket::NoMoreFinality { chain }) => {

                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Initialize { channel }) => {
                    to_shards.push(channel);
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Add { local_id, ip, node }) => {
                    let global_id = to_global_id.assign_id((shard_conn_id, local_id));
                },
                ToAggregator::FromShardWebsocket(shard_conn_id, FromShardWebsocket::Update { local_id, payload }) => {
                    let global_id = match to_global_id.get_id(&(shard_conn_id, local_id)) {
                        Some(id) => id,
                        None => continue
                    };
                },
            }
        }
    }

    /// Return a sink that a shard can send messages into to be handled by the aggregator.
    pub fn subscribe_shard(&self) -> impl Sink<FromShardWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let shard_conn_id: ConnId = self.0.shard_conn_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(ToAggregator::FromShardWebsocket(shard_conn_id, msg))
        }))
    }

    /// Return a sink that a feed can send messages into to be handled by the aggregator.
    pub fn subscribe_feed(&self) -> impl Sink<FromFeedWebsocket, Error = anyhow::Error> + Unpin {
        // Assign a unique aggregator-local ID to each connection that subscribes, and pass
        // that along with every message to the aggregator loop:
        let feed_conn_id: ConnId = self.0.feed_conn_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tx_to_aggregator = self.0.tx_to_aggregator.clone();

        // Calling `send` on this Sink requires Unpin. There may be a nicer way than this,
        // but pinning by boxing is the easy solution for now:
        Box::pin(tx_to_aggregator.with(move |msg| async move {
            Ok(ToAggregator::FromFeedWebsocket(feed_conn_id, msg))
        }))
    }

}