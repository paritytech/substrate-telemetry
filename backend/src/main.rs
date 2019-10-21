#[macro_use]
extern crate log;

use std::net::SocketAddrV4;

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
use aggregator::Aggregator;
use crate::util::Locator;

/// Entry point for connecting nodes
fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    locator: web::Data<Addr<crate::util::Locator>>,
) -> Result<HttpResponse, Error> {
    let ip = req.connection_info().remote().and_then(|addr| {
        addr.parse::<SocketAddrV4>().ok().map(|socket| *socket.ip())
    });

    let mut res = ws::handshake(&req)?;

    Ok(res.streaming(ws::WebsocketContext::with_codec(
        NodeConnector::new(aggregator.get_ref().clone(), locator.get_ref().clone().recipient(), ip),
        stream,
        Codec::new().max_size(512 * 1024), // 512kb frame limit
    )))
}

/// Entry point for connecting feeds
fn feed_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
    _locator: web::Data<Addr<crate::util::Locator>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        FeedConnector::new(aggregator.get_ref().clone()),
        &req,
        stream,
    )
}

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).expect("Must be able to start a logger");

    let sys = System::new("substrate-telemetry");
    let aggregator = Aggregator::new().start();
    let locator = SyncArbiter::start(4, move || Locator::new());

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .data(locator.clone())
            .service(web::resource("/submit").route(web::get().to(node_route)))
            .service(web::resource("/feed").route(web::get().to(feed_route)))

    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
