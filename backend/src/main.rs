#[macro_use]
extern crate log;

use actix::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error};
use actix_web_actors::ws;

mod node_connector;
mod node_message;
mod aggregator;
mod chain;
mod node;
mod util;

use node_connector::NodeConnector;
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

fn main() -> std::io::Result<()> {
    env_logger::init();

    let sys = System::new("substrate-telemetry");
    let aggregator = Aggregator::new().start();

    HttpServer::new(move || {
        App::new()
            .data(aggregator.clone())
            .service(web::resource("/submit").route(web::get().to(node_route)))
    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
