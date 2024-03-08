use std::time::Duration;

use crate::{
    models::*,
    security_cache::SecurityCache,
};
use actix::prelude::*;
use log::{debug, error, info};
use rand::{rngs::ThreadRng, Rng};

pub struct TickFeed {
    rng: ThreadRng,
    timescale: Duration,
    security_cache: Addr<SecurityCache>,
}

impl TickFeed {
    pub fn new(security_cache: Addr<SecurityCache>, timescale: Duration) -> Self {
        Self {
            rng: rand::thread_rng(),
            timescale,
            security_cache,
        }
    }
}

impl Actor for TickFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TickFeed");

        ctx.run_interval(self.timescale, |act, _| {
            debug!("Sending tick data for AAPL...");
            if let Err(e) = act.security_cache.try_send(Tick {
                security_id: AAPL.id,
                price: act.rng.gen_range(100.0f64..200f64),
            }) {
                error!("Failed to send, {:?}", e);
            };
            debug!(" Complete!");
        });
    }
}
