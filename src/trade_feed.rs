use std::time::Duration;

use actix::prelude::*;
use log::info;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, Rng};

use crate::models::*;

pub struct TradeFeed {
    rng: ThreadRng,
    timescale: Duration,
    subs: Vec<Addr<Portfolio>>,
    internal_next_trade_id: u32,
}

impl TradeFeed {
    pub fn new(subs: Vec<Addr<Portfolio>>, timescale: Duration) -> Self {
        Self {
            rng: rand::thread_rng(),
            timescale,
            subs,
            internal_next_trade_id: 0,
        }
    }

    fn gen_mock_position(&mut self) -> Position {
        let size = self.gen_size();
        let price = self.gen_price();
        Position {
            id: self.next_trade_id(),
            security: self.gen_security(),
            size: self.gen_size(),
            cost_basis: price * f64::from(size),
            unrealized_pnl: 0f64,
        }
    }

    fn gen_security(&self) -> Security {
        AAPL.clone()
    }

    fn gen_price(&mut self) -> f64 {
        Uniform::new(50f64, 1000f64).sample(&mut self.rng)
    }

    fn gen_size(&mut self) -> u32 {
        Uniform::new(1, 500).sample(&mut self.rng)
    }

    fn next_trade_id(&mut self) -> u32 {
        let id = self.internal_next_trade_id;
        self.internal_next_trade_id += 1;

        id
    }

    fn gen_duration(&mut self) -> Duration {
        let seconds = Uniform::new(1, 50).sample(&mut self.rng);

        Duration::from_millis(seconds)
    }
}

impl Actor for TradeFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TradeFeed");
        let timescale = self.timescale.clone();

        ctx.run_interval(timescale, move |act, ctx| {
            let subs = act.subs.clone();
            for sub in subs {
                // Mock open trade event
                let position = act.gen_mock_position();
                let buy = Trade::Open(position.clone());

                sub.try_send(buy).expect("failed to send buy");

                let subs = act.subs.clone();
                let when_to_sell = act.gen_duration();
                // Schedule the corresponding close trade event after another interval
                ctx.run_later(when_to_sell, move |_, _| {
                    for sub_inner in subs {
                        let sell = Trade::Close(position.id);

                        sub_inner.try_send(sell).expect("failed to send");
                    }
                });
            }
        });
    }
}
