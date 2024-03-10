use std::sync::RwLock;

use actix::prelude::*;
use actix::Context;
use log::debug;
use moka::sync::Cache;
use rand::{prelude::SliceRandom, thread_rng, Rng};

use crate::models::{Security, SecurityId, Tick};

#[derive(Debug)]
pub struct SecurityCache {
    securities: Cache<SecurityId, Security>,
    last_price: Cache<SecurityId, f64>,
    // used for rng
}

impl SecurityCache {
    pub fn new(securities: Vec<Security>) -> Self {
        let securities_cache = Cache::<SecurityId, Security>::new(512);
        let last_price_cache = Cache::<SecurityId, f64>::new(512);
        let mut rng = thread_rng();

        for sec in securities.into_iter() {
            last_price_cache.insert(sec.id, rng.gen_range(100.0f64..200f64));
            securities_cache.insert(sec.id, sec);
        }


        Self {
            securities: securities_cache,
            last_price: last_price_cache,
        }
    }

    pub fn get_latest_price(&self, id: SecurityId) -> Option<f64> {
        self.last_price.get(&id)
    }

    pub fn set_last_price(&mut self, id: SecurityId, price: f64) {
        self.last_price.insert(id, price);
    }

    pub fn get_security(&self, id: SecurityId) -> Option<Security> {
        self.securities.get(&id)
    }

    pub fn get_random_security(&self) -> Security {
        // TODO derive this from the count
        let mut rng = thread_rng();
        let random_id = self
            .securities
            .into_iter()
            .map(|(key, _)| *key)
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .copied()
            .expect("choose random is out of bounds");

        self.securities
            .get(&random_id)
            .expect("Generated id from the keys that doesn't exist anymore")
    }
}

impl SecurityCacheActor {
    pub fn new(security_cache: &'static RwLock<SecurityCache>) -> Self {
        Self {
            inner: security_cache,
        }
    }
}

pub struct SecurityCacheActor {
    inner: &'static RwLock<SecurityCache>,
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
