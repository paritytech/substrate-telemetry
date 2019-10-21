use std::net::Ipv4Addr;

use actix::prelude::*;
use serde::Deserialize;
use serde::ser::{Serialize, SerializeTuple, Serializer};
use rustc_hash::FxHashMap;

use crate::chain::{Chain, LocateNode};
use crate::types::NodeId;

/// Localhost IPv4
pub const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

#[derive(Deserialize, Clone)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Box<str>,
}

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.latitude)?;
        tup.serialize_element(&self.longitude)?;
        tup.serialize_element(&self.city)?;
        tup.end()
    }
}

pub struct Locator {
    client: reqwest::Client,
    cache: FxHashMap<Ipv4Addr, Location>,
}

impl Locator {
    pub fn new() -> Self {
        Locator {
            client: reqwest::Client::new(),
            cache: FxHashMap::default(),
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

        println!("! New location request {}", ip);

        if ip == LOCALHOST {
            let _ = chain.do_send(LocateNode {
                nid,
                location: Location { latitude: 52.5166667, longitude: 13.4, city: "Berlin".into() },
            });
            return;
        }

        if let Some(location) = self.cache.get(&ip).cloned() {
            let _ = chain.do_send(LocateNode { nid, location });
            return;
        }

        let ip_req = format!("https://ipapi.co/{}/json", ip);
        let response = self.client
            .post(&ip_req)
            .send();

        if let Err(error) = response {
            warn!("POST error for ip location: {:?}", error);
        } else if let Ok(mut response) = response {
            match response.json::<Location>() {
                Ok(location) => {
                    self.cache.insert(ip, location.clone());

                    chain.do_send(LocateNode { nid, location });
                }
                Err(err) => {
                    warn!("JSON error for ip location: {:?}", err);
                }
            }
        }
    }
}
