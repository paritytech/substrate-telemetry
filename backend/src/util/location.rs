use std::net::Ipv4Addr;
use std::sync::Arc;

use actix::prelude::*;
use rustc_hash::FxHashMap;
use parking_lot::RwLock;
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct IPApiLocate {
    city: Box<str>,
    loc: Box<str>,
}

impl IPApiLocate {
    fn into_node_location(self) -> Option<NodeLocation> {
        let IPApiLocate { city, loc } = self;

        let mut loc = loc.split(",").map(|n| n.parse());

        let latitude = loc.next()?.ok()?;
        let longitude = loc.next()?.ok()?;

        // Guarantee that the iterator has been exhausted
        if loc.next().is_some() {
            return None;
        }

        Some(NodeLocation {
            latitude,
            longitude,
            city,
        })
    }
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

        let location = match self.iplocate_ipapi_co(ip).and_then(|location| match location {
            Some(location) => Ok(Some(location)),
            None => self.iplocate_ipinfo_io(ip),
        }) {
            Ok(location) => location,
            Err(err) => return debug!("POST error for ip location: {:?}", err),
        };

        self.cache.write().insert(ip, location.clone());

        if let Some(location) = location {
            chain.do_send(LocateNode { nid, location });
        }
    }
}

impl Locator {
    fn iplocate_ipapi_co(&self, ip: Ipv4Addr) -> Result<Option<Arc<NodeLocation>>, reqwest::Error> {
        let ip_req = format!("https://ipapi.co/{}/json", ip);
        let mut response = self.client.post(&ip_req).send()?;

        let location = match response.json::<NodeLocation>() {
            Ok(location) => Some(Arc::new(location)),
            Err(err) => {
                debug!("JSON error for ip location: {:?}", err);
                None
            }
        };

        Ok(location)
    }

    fn iplocate_ipinfo_io(&self, ip: Ipv4Addr) -> Result<Option<Arc<NodeLocation>>, reqwest::Error> {
        let ip_req = format!("https://ipinfo.io/{}/json", ip);
        let mut response = self.client.post(&ip_req).send()?;

        let location = match response.json::<IPApiLocate>() {
            Ok(location) => location.into_node_location().map(Arc::new),
            Err(err) => {
                debug!("JSON error for ip location: {:?}", err);
                None
            }
        };

        Ok(location)
    }
}