use actix::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Clone)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub city: String,
}

pub struct Locator {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, Location>>>,
}

impl Locator {
    pub fn new(cache: Arc<RwLock<HashMap<String, Location>>>) -> Self {
        Locator {
            client: reqwest::Client::new(),
            cache: cache,
        }
    }
}

impl Actor for Locator {
    type Context = SyncContext<Self>;
}

pub struct Post {
    pub ip: String,
}

impl Message for Post {
    type Result = Option<Location>;
}

impl Handler<Post> for Locator {
    type Result = Option<Location>;

    fn handle(&mut self, msg: Post, _: &mut Self::Context) -> Option<Location> {
        if let Ok(cache) = self.cache.read() {
            if cache.contains_key(&msg.ip) {
                if let Some(location) = cache.get(&msg.ip) {
                    return Some((*location).clone())
                }
            }
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
                    if let Ok(mut cache) = self.cache.write() {   
                        cache.insert(msg.ip, location.clone());
                        return Some(location);
                    }
                }
                Err(err) => {
                    warn!("JSON error for ip location: {:?}", err);
                }
            }
        }
        
        None
    }
}
