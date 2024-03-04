use std::time::Duration;

use actix::prelude::*;
use log::info;
use rand::{rngs::ThreadRng, Rng};

use crate::models::*;

pub struct TradeFeed {
    rng: ThreadRng,
    timescale: Duration,
    subs: Vec<Addr<Portfolio>>,
    next_trade_id: u32,
}

impl TradeFeed {
    pub fn new(subs: Vec<Addr<Portfolio>>, timescale: Duration) -> Self {
        Self {
            rng: rand::thread_rng(),
            next_trade_id: 0,
            timescale,
            subs,
        }
    }
}

impl Actor for TradeFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TradeFeed");
        let timescale = self.timescale.clone();

        ctx.run_interval(timescale, move |act, ctx| {
            let security = match act.rng.gen_range(0..=1) {
                0 => AAPL.clone(),
                1 => TSLA.clone(),
                _ => panic!("out of range"),
            };

            for sub in &act.subs {
                // Mock open trade event
                let trade_id = act.next_trade_id;
                let buy = Trade::Open(Position {
                    id: trade_id,
                    security: security.clone(),
                    size: 10,
                    unrealized_pnl: 0.0,
                });

                act.next_trade_id += 1;

                sub.try_send(buy).expect("failed to send");

                let subs = act.subs.clone();

                // Schedule the corresponding close trade event after another interval
                ctx.run_later(timescale, move |_, _| {
                    for sub_inner in subs {
                        let sell = Trade::Close(trade_id);

                        sub_inner.try_send(sell).expect("failed to send");
                    }
                });
            }
        });
    }
}
