use std::mem;
use std::time::{Duration, Instant};

use crate::aggregator::{AddNode, Aggregator};
use crate::chain::{Chain, RemoveNode, UpdateNode};
use crate::shard::ShardMessage;
use crate::types::NodeId;
use crate::util::{DenseMap, Hash};
use actix::prelude::*;
use actix_http::ws::Item;
use actix_web_actors::ws::{self, CloseReason};
use bincode::Options;
use bytes::{Bytes, BytesMut};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Continuation buffer limit, 10mb
const CONT_BUF_LIMIT: usize = 10 * 1024 * 1024;

pub struct ShardConnector {
    /// Client must send ping at least once every 60 seconds (CLIENT_TIMEOUT),
    hb: Instant,
    /// Aggregator actor address
    aggregator: Addr<Aggregator>,
    /// Genesis hash of the chain this connection will be submitting data for
    genesis_hash: Hash,
    /// Chain address to which this multiplex connector is delegating messages
    chain: Option<Addr<Chain>>,
    /// Mapping `ShardConnId` to `NodeId`
    nodes: DenseMap<NodeId>,
    /// Buffer for constructing continuation messages
    contbuf: BytesMut,
}

impl Actor for ShardConnector {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if let Some(ref chain) = self.chain {
            for (_, nid) in self.nodes.iter() {
                chain.do_send(RemoveNode(*nid))
            }
        }
    }
}

impl ShardConnector {
    pub fn new(aggregator: Addr<Aggregator>, genesis_hash: Hash) -> Self {
        Self {
            hb: Instant::now(),
            aggregator,
            genesis_hash,
            chain: None,
            nodes: DenseMap::new(),
            contbuf: BytesMut::new(),
        }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // stop actor
                ctx.close(Some(CloseReason {
                    code: ws::CloseCode::Abnormal,
                    description: Some("Missed heartbeat".into()),
                }));
                ctx.stop();
            }
        });
    }

    fn handle_message(&mut self, msg: ShardMessage, ctx: &mut <Self as Actor>::Context) {
        let ShardMessage { conn_id, payload } = msg;

        // TODO: get `NodeId` for `ShardConnId` and proxy payload to `self.chain`.
    }

    fn start_frame(&mut self, bytes: &[u8]) {
        if !self.contbuf.is_empty() {
            log::error!("Unused continuation buffer");
            self.contbuf.clear();
        }
        self.continue_frame(bytes);
    }

    fn continue_frame(&mut self, bytes: &[u8]) {
        if self.contbuf.len() + bytes.len() <= CONT_BUF_LIMIT {
            self.contbuf.extend_from_slice(&bytes);
        } else {
            log::error!("Continuation buffer overflow");
            self.contbuf = BytesMut::new();
        }
    }

    fn finish_frame(&mut self) -> Bytes {
        mem::replace(&mut self.contbuf, BytesMut::new()).freeze()
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ShardConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();

        let data = match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                return;
            }
            Ok(ws::Message::Pong(_)) => return,
            Ok(ws::Message::Text(text)) => text.into_bytes(),
            Ok(ws::Message::Binary(data)) => data,
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            Ok(ws::Message::Nop) => return,
            Ok(ws::Message::Continuation(cont)) => match cont {
                Item::FirstText(bytes) | Item::FirstBinary(bytes) => {
                    self.start_frame(&bytes);
                    return;
                }
                Item::Continue(bytes) => {
                    self.continue_frame(&bytes);
                    return;
                }
                Item::Last(bytes) => {
                    self.continue_frame(&bytes);
                    self.finish_frame()
                }
            },
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
        };

        match bincode::options().deserialize(&data) {
            Ok(msg) => self.handle_message(msg, ctx),
            #[cfg(debug)]
            Err(err) => {
                log::warn!("Failed to parse shard message: {}", err,)
            }
            #[cfg(not(debug))]
            Err(_) => (),
        }
    }
}
