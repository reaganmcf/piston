use std::{collections::HashMap, time::Duration};

use log::info;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, Rng};
use xtra::prelude::*;

use crate::models::*; //

#[derive(Clone)]
pub struct TradeFeed {
    timescale: Duration,
    portfolios: HashMap<String, Address<Portfolio>>,
    internal_next_trade_id: u32,
}

impl TradeFeed {
    pub fn new(portfolios: HashMap<String, Address<Portfolio>>, timescale: Duration) -> Self {
        Self {
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

    fn gen_portfolio_code(&self) -> String {
        let portfolio_codes: Vec<_> = self.portfolios.keys().collect();
        let mut rng = ThreadRng::default();
        let idx = rng.gen_range(0..self.portfolios.len());

        portfolio_codes[idx].clone()
    }

    fn gen_security(&self) -> Security {
        AAPL.clone()
    }

    fn gen_price(&self) -> f64 {
        Uniform::new(50f64, 1000f64).sample(&mut ThreadRng::default())
    }

    fn gen_size(&self) -> u32 {
        Uniform::new(1, 500).sample(&mut ThreadRng::default())
    }

    fn next_trade_id(&mut self) -> u32 {
        let id = self.internal_next_trade_id;
        self.internal_next_trade_id += 1;

        id
    }

    fn gen_duration(&self) -> Duration {
        let seconds = Uniform::new(1, 50).sample(&mut ThreadRng::default());

        Duration::from_millis(seconds)
    }

    fn pick_random_portfolio(&self) -> (String, Address<Portfolio>) {
        let portfolio_codes: Vec<_> = self.portfolios.keys().collect();
        let idx = ThreadRng::default().gen_range(0..self.portfolios.len());
        let key = portfolio_codes[idx].clone();
        let cloned = key.clone();

        (key, self.portfolios[&cloned].clone())
    }
}

impl Actor for TradeFeed {
    type Stop = ();

    async fn started(&mut self, _mailbox: &Mailbox<Self>) -> Result<(), ()> {
        info!("Started TradeFeed");
        let timescale = self.timescale.clone();

        let mut act = self.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(timescale).await;

                let (portfolio_code, sub) = act.pick_random_portfolio();
                // Mock open trade event
                let position = act.gen_mock_position();
                let buy = Trade {
                    portfolio_code: portfolio_code.clone(),
                    trade_type: TradeType::Open(position.clone()),
                };

                sub.send(buy).await.expect("failed to send buy");

                let when_to_sell = act.gen_duration();
                // Schedule the corresponding close trade event after another interval
                tokio::time::sleep(when_to_sell).await;
                let sell = Trade {
                    portfolio_code,
                    trade_type: TradeType::Close(position.id),
                };

                sub.send(sell).await.expect("failed to send");
            }
        });

        Ok(())
    }

    async fn stopped(self) -> Self::Stop {}
}
