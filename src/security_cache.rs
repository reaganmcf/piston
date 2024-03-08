use actix::prelude::*;
use std::collections::HashMap;

use actix::Context;
use log::debug;

use crate::models::{Security, SecurityId, Tick};

pub struct SecurityCache {
    latest_price: HashMap<SecurityId, f64>,
}

impl SecurityCache {
    pub fn new() -> Self {
        Self {
            latest_price: Default::default(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Option<f64>")]
pub struct GetLatestPrice(pub SecurityId);

#[derive(Message)]
#[rtype(result = "Security")]
pub struct GetSecurity(SecurityId);

impl Actor for SecurityCache {
    type Context = Context<Self>;
}

impl Handler<Tick> for SecurityCache {
    type Result = ();

    fn handle(&mut self, msg: Tick, _ctx: &mut Self::Context) -> Self::Result {
        debug!("got tick data! {:?}", msg);
        self.latest_price.insert(msg.security_id, msg.price);
    }
}

impl Handler<GetLatestPrice> for SecurityCache {
    type Result = Option<f64>;

    fn handle(&mut self, msg: GetLatestPrice, _ctx: &mut Self::Context) -> Self::Result {
        self.latest_price.get(&msg.0).copied()
    }
}
