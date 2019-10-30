use std::net::Ipv4Addr;
use std::sync::Arc;

use actix::prelude::*;
use rustc_hash::FxHashMap;
use parking_lot::RwLock;

use crate::chain::{Chain, LocateNode};
use crate::types::{NodeId, NodeLocation};

#[derive(Clone)]
pub struct Locator {
    client: reqwest::Client,
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, Option<Arc<NodeLocation>>>>>,
}

pub struct LocatorFactory {
    cache: Arc<RwLock<FxHashMap<Ipv4Addr, Option<Arc<NodeLocation>>>>>,
}

impl LocatorFactory {
    pub fn new() -> Self {
        let mut cache = FxHashMap::default();

        // Default entry for localhost
        cache.insert(
            Ipv4Addr::new(127, 0, 0, 1),
            Some(Arc::new(NodeLocation { latitude: 52.5166667, longitude: 13.4, city: "Berlin".into() })),
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

        if let Some(item) = self.cache.read().get(&ip) {
            if let Some(location) = item {
                return chain.do_send(LocateNode { nid, location: location.clone() });
            }

            return
        }

        let ip_req = format!("https://ipapi.co/{}/json", ip);
        let mut response = match self.client.post(&ip_req).send() {
            Ok(response) => response,
            Err(err) => return warn!("POST error for ip location: {:?}", err),
        };

        let location = match response.json::<NodeLocation>() {
            Ok(location) => Some(Arc::new(location)),
            Err(err) => {
                warn!("JSON error for ip location: {:?}", err);
                None
            }
        };

        self.cache.write().insert(ip, location.clone());

        if let Some(location) = location {
            chain.do_send(LocateNode { nid, location });
        }
    }
}
