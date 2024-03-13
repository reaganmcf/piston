use crate::{models::*, portfolio::Portfolio, security_cache::SecurityCache};
use actix::prelude::*;
use log::{debug, error, info};
use piston_shared::*;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, Rng};
use std::{collections::HashMap, sync::RwLock, time::Duration};

pub struct TradeFeed {
    rng: ThreadRng,
    timescale: Duration,
    portfolios: HashMap<String, Addr<Portfolio>>,
    security_cache: &'static RwLock<SecurityCache>,
    internal_next_trade_id: u32,
}

impl TradeFeed {
    pub fn new(
        portfolios: HashMap<String, Addr<Portfolio>>,
        security_cache: &'static RwLock<SecurityCache>,
        timescale: Duration,
    ) -> Self {
        Self {
            rng: rand::thread_rng(),
            timescale,
            portfolios,
            security_cache,
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
        self.security_cache
            .read()
            .expect("Failed to read security cache")
            .get_random_security()
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

    fn schedule_trade_generation(&mut self, ctx: &mut Context<Self>) {
        let batch_size = 5;

        ctx.run_later(self.timescale, move |act, ctx| {
            // Generate a batch of trades instead of a single one
            for _ in 0..batch_size {
                let (portfolio_code, sub) = act.pick_random_portfolio();
                let position = act.gen_mock_position();
                let trade = Trade {
                    portfolio_code: portfolio_code.clone(),
                    trade_type: TradeType::Open(position.clone()),
                };

                match sub.try_send(trade) {
                    Ok(_) => debug!("Trade sent successfully"),
                    Err(e) => error!("Failed to send trade: {}", e),
                };

                // Dynamically schedule the closing of the position
                let when_to_sell = act.gen_duration();
                ctx.run_later(when_to_sell, move |_act, _ctx| {
                    let sell = Trade {
                        portfolio_code: portfolio_code.clone(),
                        trade_type: TradeType::Close(position.id),
                    };

                    match sub.try_send(sell) {
                        Ok(_) => debug!(
                            "Successfully scheduled position close for position ID {}",
                            position.id
                        ),
                        Err(e) => error!(
                            "Failed to schedule position close for position ID {}: {}",
                            position.id, e
                        ),
                    };
                });
            }

            // Reschedule the next trade generation
            act.schedule_trade_generation(ctx);
        });
    }
}

impl Actor for TradeFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started TradeFeed");
        self.schedule_trade_generation(ctx);
    }
}
