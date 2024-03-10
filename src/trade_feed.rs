use std::{collections::HashMap, time::Duration};

use actix::prelude::*;
use log::info;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, Rng};

use crate::{models::*, portfolio::Portfolio};

pub struct TradeFeed {
    rng: ThreadRng,
    timescale: Duration,
    portfolios: HashMap<String, Addr<Portfolio>>,
    internal_next_trade_id: u32,
}

impl TradeFeed {
    pub fn new(portfolios: HashMap<String, Addr<Portfolio>>, timescale: Duration) -> Self {
        Self {
            rng: rand::thread_rng(),
            timescale,
            portfolios,
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

    fn gen_portfolio_code(&mut self) -> String {
        let portfolio_codes: Vec<_> = self.portfolios.keys().collect();
        let idx = self.rng.gen_range(0..self.portfolios.len());

        portfolio_codes[idx].clone()
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

    fn pick_random_portfolio(&mut self) -> (String, Addr<Portfolio>) {
        let portfolio_codes: Vec<_> = self.portfolios.keys().collect();
        let idx = self.rng.gen_range(0..self.portfolios.len());
        let key = portfolio_codes[idx].clone();
        let cloned = key.clone();

        (key, self.portfolios[&cloned].clone())
    }
}

impl Actor for TradeFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TradeFeed");
        let timescale = self.timescale.clone();

        ctx.run_interval(timescale, move |act, ctx| {
            let (portfolio_code, sub) = act.pick_random_portfolio();
            // Mock open trade event
            let position = act.gen_mock_position();
            let buy = Trade {
                portfolio_code: portfolio_code.clone(),
                trade_type: TradeType::Open(position.clone()),
            };

            sub.try_send(buy).expect("failed to send buy");

            let when_to_sell = act.gen_duration();
            // Schedule the corresponding close trade event after another interval
            ctx.run_later(when_to_sell, move |_, _| {
                let sell = Trade {
                    portfolio_code,
                    trade_type: TradeType::Close(position.id),
                };

                sub.try_send(sell).expect("failed to send");
            });
        });
    }
}
