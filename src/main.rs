use dotenv::dotenv;
use futures::{stream::repeat, StreamExt};
use log::{debug, error, info, warn};
use security_cache::{GetLatestPrice, SecurityCache};
use stats::{PortfolioStatsEvent, PortfolioStatsFeed};
use std::{collections::HashMap, time::Duration};
use tick_feed::TickFeed;
use trade_feed::TradeFeed;
use xtra::prelude::*;

mod models;
mod security_cache;
mod stats;
mod tick_feed;
mod trade_feed;
use models::*;

impl Portfolio {
    pub fn new(code: String, security_cache: Address<SecurityCache>) -> Self {
        Self {
            code,
            security_cache,
            positions: HashMap::default(),
            pnl: 0f64,
            trade_count: 0,
        }
    }
}

impl Handler<Trade> for Portfolio {
    type Return = ();

    async fn handle(&mut self, msg: Trade, _: &mut Context<Self>) -> Self::Return {
        if msg.portfolio_code != self.code {
            return;
        }

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
    type Return = ();

    async fn handle(
        &mut self,
        _msg: PortfolioStatsEvent,
        _ctx: &mut Context<Self>,
    ) -> Self::Return {
        let num_positions = self.positions.len();

        for pos in &mut self.positions.values_mut() {
            let price = self
                .security_cache
                .send(GetLatestPrice(pos.security.id))
                .await
                .expect("Failed to get the price from security cache")
                .expect("Unknown security id");

            let new_price = f64::from(pos.size) * price;
            pos.unrealized_pnl = new_price - pos.cost_basis;

            info!(
                "STATS: {}, {} posititons, {} total trades, realized: {}, unrealized: {}",
                self.code, num_positions, self.trade_count, self.pnl, pos.unrealized_pnl
            );
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let timescale = Duration::from_millis(500);

    let security_cache = xtra::spawn_tokio(SecurityCache::new(), Mailbox::unbounded());

    let portfolios = get_portfolios(security_cache.clone());
    let portfolio_addr_map: HashMap<_, _> = portfolios
        .into_iter()
        .map(|p| (p.code.clone(), xtra::spawn_tokio(p, Mailbox::unbounded())))
        .collect();
    let portfolio_addrs: Vec<Address<Portfolio>> = portfolio_addr_map.values().into_iter().cloned().collect();

    xtra::spawn_tokio(
        TickFeed::new(security_cache, timescale),
        Mailbox::unbounded(),
    );
    xtra::spawn_tokio(
        TradeFeed::new(portfolio_addr_map, timescale),
        Mailbox::unbounded(),
    );
    xtra::spawn_tokio(
        PortfolioStatsFeed::new(portfolio_addrs.clone()),
        Mailbox::unbounded(),
    );

    // repeat(Duration::from_secs(1))
    //     .then(tokio::time::sleep)
    //     .map(|_| Ok(PortfolioStatsEvent {}))
    //     .forward(portfolio_addrs.first().cloned().unwrap().into_sink())
    //     .await
    //     .unwrap();
    tokio::time::sleep(Duration::from_secs(60 * 60)).await;
}

fn get_portfolios(security_cache: Address<SecurityCache>) -> Vec<Portfolio> {
    vec![
        Portfolio::new(String::from("RMCF"), security_cache.clone()),
        Portfolio::new(String::from("ATAR"), security_cache.clone()),
        Portfolio::new(String::from("COLT"), security_cache),
    ]
}
