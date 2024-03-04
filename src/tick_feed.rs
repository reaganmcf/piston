use std::time::Duration;

use crate::models::*;
use actix::prelude::*;
use log::{debug, error, info};
use rand::{rngs::ThreadRng, Rng};

pub struct TickFeed {
    rng: ThreadRng,
    timescale: Duration,
    subs: Vec<Addr<Portfolio>>,
}

impl TickFeed {
    pub fn new(portfolios: Vec<Addr<Portfolio>>, timescale: Duration) -> Self {
        Self {
            rng: rand::thread_rng(),
            timescale,
            subs: portfolios,
        }
    }
}

impl Actor for TickFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TickFeed");

        ctx.run_interval(self.timescale, |act, _| {
            for subscriber in &act.subs {
                debug!("Sending tick data for AAPL...");
                if let Err(e) = subscriber.try_send(Tick {
                    security: AAPL.clone(),
                    price: act.rng.gen_range(100.0f64..200f64),
                }) {
                    error!("Failed to send, {:?}", e);
                };
                debug!(" Complete!");
            }
        });
    }
}
