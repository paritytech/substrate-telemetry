use std::net::Ipv4Addr;

use actix::prelude::*;
use serde::Deserialize;
use serde::ser::{Serialize, SerializeTuple, Serializer};
use rustc_hash::FxHashMap;

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

pub struct Post {
    pub ip: Ipv4Addr,
}

impl Message for Post {
    type Result = Option<Location>;
}

impl Handler<Post> for Locator {
    type Result = Option<Location>;

    fn handle(&mut self, msg: Post, _: &mut Self::Context) -> Option<Location> {
        if let Some(location) = self.cache.get(&msg.ip) {
            return Some(location.clone())
        }

        let ip_req = format!("https://ipapi.co/{}/json", msg.ip);
        let response = self.client
            .post(&ip_req)
            .send();

        if let Err(error) = response {
            warn!("POST error for ip location: {:?}", error);
        } else if let Ok(mut response) = response {
            match response.json::<Location>() {
                Ok(location) => {
                    self.cache.insert(msg.ip, location.clone());

                    return Some(location);
                }
                Err(err) => {
                    warn!("JSON error for ip location: {:?}", err);
                }
            }
        }

        None
    }
}
