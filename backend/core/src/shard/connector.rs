use std::time::{Duration, Instant};

use crate::aggregator::{AddNode, Aggregator};
use crate::chain::{Chain, RemoveNode, UpdateNode};
use actix::prelude::*;
use actix_web_actors::ws::{self, CloseReason};
use bincode::Options;
use shared::types::NodeId;
use shared::util::{DenseMap, Hash};
use shared::ws::{MultipartHandler, WsMessage};
use shared::shard::ShardMessage;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

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
    /// Container for handling continuation messages
    multipart: MultipartHandler,
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
            multipart: MultipartHandler::default(),
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
        println!("{:?}", msg);
        // match msg {
        //     ShardMessage::New(ip) => (),
        //     ShardMessage::Payload { conn_id, payload } => (),
        // }

        // TODO: get `NodeId` for `ShardConnId` and proxy payload to `self.chain`.
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ShardConnector {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();

        let data = match msg.map(|msg| self.multipart.handle(msg)) {
            Ok(WsMessage::Nop) => return,
            Ok(WsMessage::Ping(msg)) => {
                ctx.pong(&msg);
                return;
            }
            Ok(WsMessage::Data(data)) => data,
            Ok(WsMessage::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            Err(error) => {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
        };

        println!("Received {} bytes", data.len());

        match bincode::options().deserialize(&data) {
            Ok(msg) => self.handle_message(msg, ctx),
            // #[cfg(debug)]
            Err(err) => {
                log::warn!("Failed to parse shard message: {}", err,)
            }
            // #[cfg(not(debug))]
            // Err(_) => (),
        }
    }
}
