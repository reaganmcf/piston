use actix::prelude::*;
use dotenv::dotenv;
use log::{debug, info};
use stats::{PortfolioStatsEvent, PortfolioStatsFeed};
use std::{collections::HashMap, time::Duration};
use tick_feed::TickFeed;
use trade_feed::TradeFeed;

mod models;
mod stats;
mod tick_feed;
mod trade_feed;
use models::*;

impl Portfolio {
    pub fn new(code: String) -> Self {
        Self {
            code,
            positions: HashMap::default(),
            pnl: 0f64,
            trade_count: 0,
        }
    }
}

impl Actor for Portfolio {
    type Context = Context<Self>;
}

impl Handler<Tick> for Portfolio {
    type Result = ();

    fn handle(&mut self, msg: Tick, _ctx: &mut Self::Context) -> Self::Result {
        debug!("got tick data! {:?}", msg);

        self.positions.iter_mut().for_each(|(_, p)| {
            if p.security.id == msg.security.id {
                p.unrealized_pnl = msg.price * f64::from(p.size);
            }
        })
    }
}

impl Handler<Trade> for Portfolio {
    type Result = ();

    fn handle(&mut self, msg: Trade, _: &mut Self::Context) -> Self::Result {
        if msg.portfolio_code != self.code { return }

        debug!("Got trade message, {:#?}", msg);
        self.trade_count += 1;

        match msg.trade_type {
            TradeType::Open(pos) => {
                debug!(
                    "{} has entered a new {} position",
                    self.code, &pos.security.ticker
                );
                self.positions.insert(pos.id, pos);
            }
            TradeType::Close(pos_id) => match self.positions.remove(&pos_id) {
                None => panic!("Closing a position that does not exist"),
                Some(p) => {
                    self.pnl += p.unrealized_pnl;
                    debug!(
                        "{} has closed their {} position",
                        self.code, p.security.ticker
                    );
                }
            },
        }
    }
}

impl Handler<PortfolioStatsEvent> for Portfolio {
    type Result = ();

    fn handle(&mut self, _msg: PortfolioStatsEvent, _: &mut Self::Context) -> Self::Result {
        let unrealized_pnl = self
            .positions
            .iter()
            .fold(0f64, |acc, p| acc + p.1.unrealized_pnl);

        info!(
            "STATS: {}, {} posititons, {} total trades, realized: {}, unrealized: {}",
            self.code,
            self.positions.len(),
            self.trade_count,
            self.pnl,
            unrealized_pnl
        );
    }
}

fn main() {
    dotenv().ok();
    env_logger::init();
    let system = System::new();

    let timescale = Duration::from_millis(1);

    system.block_on(async {
        let portfolios = get_portfolios();
        let portfolio_addr_map: HashMap<_, _> = portfolios
            .into_iter()
            .map(|p| (p.code.clone(), p.start()))
            .collect();
        let portfolio_addrs: Vec<_> = portfolio_addr_map.values().into_iter().cloned().collect();

        TickFeed::new(portfolio_addrs.clone(), timescale).start();
        TradeFeed::new(portfolio_addr_map, timescale).start();
        PortfolioStatsFeed::new(portfolio_addrs).start()
    });

    system.run().expect("Failed to run the system");
}

fn get_portfolios() -> Vec<Portfolio> {
    vec![
        Portfolio::new(String::from("RMCF")),
        Portfolio::new(String::from("ATAR")),
        Portfolio::new(String::from("COLT")),
    ]
}
