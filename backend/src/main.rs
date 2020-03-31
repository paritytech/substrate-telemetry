#[macro_use]
extern crate log;

use std::net::Ipv4Addr;

use actix::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error};
use actix_web_actors::ws;
use actix_http::ws::Codec;

mod types;
mod aggregator;
mod chain;
mod node;
mod feed;
mod util;

use node::connector::NodeConnector;
use feed::connector::FeedConnector;
use aggregator::{Aggregator, GetNetworkState, GetHealth};
use util::{Locator, LocatorFactory};
use types::NodeId;

/// Entry point for connecting nodes
fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    locator: web::Data<Addr<Locator>>,
) -> Result<HttpResponse, Error> {
    let ip = req.connection_info().remote().and_then(|mut addr| {
        if let Some(port_idx) = addr.find(":") {
            addr = &addr[..port_idx];
        }
        addr.parse::<Ipv4Addr>().ok()
    });

    let mut res = ws::handshake(&req)?;
    let aggregator = aggregator.get_ref().clone();
    let locator = locator.get_ref().clone().recipient();

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(aggregator, locator, ip),
        stream,
        Codec::new().max_size(10 * 1024 * 1024), // 10mb frame limit
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
            match data.and_then(|nested| nested) {
                Some(body) => HttpResponse::Ok().content_type("application/json").body(body),
                None => HttpResponse::Ok().body("Node has disconnected or has not submitted its network state yet"),
            }
        })
}

fn health(
    aggregator: web::Data<Addr<Aggregator>>
) -> impl Future<Item = HttpResponse, Error = Error> {
    aggregator
        .send(GetHealth)
        .from_err()
        .and_then(|count| {
            let body = format!("Connected chains: {}", count);

            HttpResponse::Ok().body(body)
        })
}

/// Telemetry entry point. Listening by default on 127.0.0.1:8000.
/// This can be changed using the `PORT` and `BIND` ENV variables.
fn main() -> std::io::Result<()> {
    use web::{resource, get};

    simple_logger::init_with_level(log::Level::Info).expect("Must be able to start a logger");

    let sys = System::new("substrate-telemetry");
    let aggregator = Aggregator::new().start();
    let factory = LocatorFactory::new();
    let locator = SyncArbiter::start(4, move || factory.create());

    let port = std::env::var("PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8000u16);
    let bind_address = std::env::var("BIND").ok().unwrap_or("127.0.0.1".to_string());

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .data(locator.clone())
            .service(resource("/submit").route(get().to(node_route)))
            .service(resource("/submit/").route(get().to(node_route)))
            .service(resource("/feed").route(get().to(feed_route)))
            .service(resource("/feed/").route(get().to(feed_route)))
            .service(resource("/network_state/{chain}/{nid}").route(get().to_async(state_route)))
            .service(resource("/network_state/{chain}/{nid}/").route(get().to_async(state_route)))
            .service(resource("/health").route(get().to_async(health)))
            .service(resource("/health/").route(get().to_async(health)))
    })
    .bind(format!("{}:{}", bind_address, port))?
    .start();

    sys.run()
}
