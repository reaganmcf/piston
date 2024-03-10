use std::sync::RwLock;

use actix::prelude::*;
use moka::sync::Cache;

use actix::Context;
use log::debug;

use crate::models::{Security, SecurityId, Tick};

#[derive(Debug)]
pub struct SecurityCache {
    last_price: Cache<SecurityId, f64>,
}

pub struct SecurityCacheActor {
    inner: &'static RwLock<SecurityCache>,
}

impl SecurityCache {
    pub fn new() -> Self {
        Self {
            last_price: Cache::new(10_000),
        }
    }

    pub fn get_latest_price(&self, id: SecurityId) -> Option<f64> {
        self.last_price.get(&id)
    }

    pub fn set_last_price(&mut self, id: SecurityId, price: f64) {
        self.last_price.insert(id, price);
    }
}

impl SecurityCacheActor {
    pub fn new(security_cache: &'static RwLock<SecurityCache>) -> Self {
        Self {
            inner: security_cache,
        }
    }
}

#[derive(Message)]
#[rtype(result = "Option<f64>")]
pub struct GetLatestPrice(pub SecurityId);

#[derive(Message)]
#[rtype(result = "Security")]
pub struct GetSecurity(SecurityId);

impl Actor for SecurityCacheActor {
    type Context = Context<Self>;
}

impl Handler<Tick> for SecurityCacheActor {
    type Result = ();

    fn handle(&mut self, msg: Tick, _ctx: &mut Self::Context) -> Self::Result {
        debug!("got tick data! {:?}", msg);
        self.inner
            .write()
            .expect("failed to get the lock")
            .set_last_price(msg.security_id, msg.price);
    }
}
