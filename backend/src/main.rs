#[macro_use]
extern crate log;

use actix::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error};
use actix_web_actors::ws;

mod aggregator;
mod chain;
mod node;
mod feed;
mod util;

use node::connector::NodeConnector;
use feed::connector::FeedConnector;
use aggregator::Aggregator;

/// Entry point for connecting nodes
fn node_route(
    req: HttpRequest,
    stream: web::Payload,
    aggregator: web::Data<Addr<Aggregator>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        NodeConnector::new(aggregator.get_ref().clone()),
        &req,
        stream,
    )
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

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).expect("Must be able to start a logger");

    let sys = System::new("substrate-telemetry");
    let aggregator = Aggregator::new().start();

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .service(web::resource("/submit").route(web::get().to(node_route)))
            .service(web::resource("/feed").route(web::get().to(feed_route)))
    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
