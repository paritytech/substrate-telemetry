use std::net::Ipv4Addr;
use std::sync::Arc;

use actix::prelude::*;
use serde::Deserialize;
use rustc_hash::FxHashMap;
use parking_lot::RwLock;

use crate::chain::{Chain, LocateNode};
use crate::types::{NodeId, NodeLocation};

// Having a custom type here because serde can't deserialize to Arc<str>
#[derive(Deserialize)]
struct Location {
    latitude: f32,
    longitude: f32,
    city: Box<str>,
}

impl From<Location> for NodeLocation {
    fn from(loc: Location) -> NodeLocation {
        NodeLocation {
            latitude: loc.latitude,
            longitude: loc.longitude,
            city: loc.city.into(),
        }
    }
}

#[derive(Clone)]
pub struct Locator {
    client: reqwest::Client,
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, NodeLocation>>>,
}

pub struct LocatorFactory {
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, NodeLocation>>>,
}

impl LocatorFactory {
    pub fn new() -> Self {
        let mut cache = FxHashMap::default();

        // Default entry for localhost
        cache.insert(
            Ipv4Addr::new(127, 0, 0, 1),
            NodeLocation { latitude: 52.5166667, longitude: 13.4, city: "Berlin".into() },
        );

        LocatorFactory {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    pub fn create(&self) -> Locator {
        Locator {
            client: reqwest::Client::new(),
            cache: self.cache.clone(),
        }
    }
}

impl Actor for Locator {
    type Context = SyncContext<Self>;
}

#[derive(Message)]
pub struct LocateRequest {
    pub ip: Ipv4Addr,
    pub nid: NodeId,
    pub chain: Addr<Chain>,
}

impl Handler<LocateRequest> for Locator {
    type Result = ();

    fn handle(&mut self, msg: LocateRequest, _: &mut Self::Context) {
        let LocateRequest { ip, nid, chain } = msg;

        if let Some(location) = self.cache.read().get(&ip).cloned() {
            return chain.do_send(LocateNode { nid, location });
        }

        let ip_req = format!("https://ipapi.co/{}/json", ip);
        let mut response = match self.client.post(&ip_req).send() {
            Ok(response) => response,
            Err(err) => return warn!("POST error for ip location: {:?}", err),
        };

        let location = match response.json::<Location>() {
            Ok(location) => NodeLocation::from(location),
            Err(err) => return warn!("JSON error for ip location: {:?}", err),
        };

        self.cache.write().insert(ip, location.clone());

        chain.do_send(LocateNode { nid, location });
    }
}
