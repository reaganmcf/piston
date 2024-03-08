use std::collections::HashMap;
use xtra::prelude::*;

use log::debug;

use crate::models::{SecurityId, Tick};

#[derive(Actor)]
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

pub struct GetLatestPrice(pub SecurityId);

pub struct GetSecurity(SecurityId);

impl Handler<Tick> for SecurityCache {
    type Return = ();

    async fn handle(&mut self, msg: Tick, _ctx: &mut Context<Self>) -> Self::Return {
        debug!("got tick data! {:?}", msg);
        self.latest_price.insert(msg.security_id, msg.price);
    }
}

impl Handler<GetLatestPrice> for SecurityCache {
    type Return = Option<f64>;

    async fn handle(&mut self, msg: GetLatestPrice, _ctx: &mut Context<Self>) -> Self::Return {
        self.latest_price.get(&msg.0).copied()
    }
}
