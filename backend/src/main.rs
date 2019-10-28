#[macro_use]
extern crate log;

use std::net::SocketAddrV4;

use actix::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error};
use actix_web_actors::ws;
use actix_http::ws::Codec;
use bytes::Bytes;

mod types;
mod aggregator;
mod chain;
mod node;
mod feed;
mod util;

use node::connector::NodeConnector;
use feed::connector::FeedConnector;
use aggregator::{Aggregator, GetNetworkState};
use util::{Locator, LocatorFactory};
use types::NodeId;

/// Entry point for connecting nodes
fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    locator: web::Data<Addr<Locator>>,
) -> Result<HttpResponse, Error> {
    let ip = req.connection_info().remote().and_then(|addr| {
        addr.parse::<SocketAddrV4>().ok().map(|socket| *socket.ip())
    });

    let mut res = ws::handshake(&req)?;
    let aggregator = aggregator.get_ref().clone();
    let locator = locator.get_ref().clone().recipient();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(aggregator, locator, ip),
        stream,
        Codec::new().max_size(1024 * 1024), // 1mb frame limit
    )))
}

/// Entry point for connecting feeds
fn feed_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        FeedConnector::new(aggregator.get_ref().clone()),
        &req,
        stream,
    )
}

fn state_route(
    path: web::Path<(Box<str>, NodeId)>,
    aggregator: web::Data<Addr<Aggregator>>
) -> impl Future<Item = HttpResponse, Error = Error> {
    let (chain, nid) = path.into_inner();

    aggregator
        .send(GetNetworkState(chain, nid))
        .flatten()
        .from_err()
        .and_then(|data| {
            match data.and_then(|nested| nested).and_then(get_network_state) {
                Some(body) => HttpResponse::Ok().content_type("application/json").body(body),
                None => HttpResponse::Ok().body("Node has disconnected or has not submitted its network state yet"),
            }
        })
}

fn get_network_state(raw: Bytes) -> Option<Bytes> {
    const NEEDLE: &[u8] = b"\"network_state\":";

    let mut search: &[u8] = &*raw;
    let mut offset = 0;

    while !search.starts_with(NEEDLE) {
        offset += 1;
        search = search.get(1..)?;
    }

    let end = raw.len() - 2;
    let start = offset + NEEDLE.len();

    if start > end {
        return None;
    }

    Some(raw.slice(start, end))
}

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).expect("Must be able to start a logger");

    let sys = System::new("substrate-telemetry");
    let aggregator = Aggregator::new().start();
    let factory = LocatorFactory::new();
    let locator = SyncArbiter::start(4, move || factory.create());

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .data(locator.clone())
            .service(web::resource("/submit").route(web::get().to(node_route)))
            .service(web::resource("/feed").route(web::get().to(feed_route)))
            .service(web::resource("/network_state/{chain}/{nid}").route(web::get().to_async(state_route)))
            .service(web::resource("/network_state/{chain}/{nid}/").route(web::get().to_async(state_route)))
    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
