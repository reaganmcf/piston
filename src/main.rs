use actix::prelude::*;
use dotenv::dotenv;
use lazy_static::lazy_static;
use security_cache::{SecurityCache, SecurityCacheActor};
use stats::PortfolioStatsFeed;
use std::{collections::HashMap, sync::RwLock, time::Duration};
use tick_feed::TickFeed;
use trade_feed::TradeFeed;

mod models;
mod portfolio;
mod security_cache;
mod stats;
mod tick_feed;
mod trade_feed;

use portfolio::Portfolio;

lazy_static! {
    static ref SECURITY_CACHE: RwLock<SecurityCache> = RwLock::new(SecurityCache::new());
}

fn main() {
    dotenv().ok();
    env_logger::init();
    let system = System::new();

    let timescale = Duration::from_millis(1);

    system.block_on(async {
        let security_cache_actor = SecurityCacheActor::new(&SECURITY_CACHE).start();

        let portfolios = get_portfolios(&SECURITY_CACHE);
        let portfolio_addr_map: HashMap<_, _> = portfolios
            .into_iter()
            .map(|p| (p.code.clone(), p.start()))
            .collect();
        let portfolio_addrs: Vec<_> = portfolio_addr_map.values().cloned().collect();

        TickFeed::new(security_cache_actor, timescale).start();
        TradeFeed::new(portfolio_addr_map, timescale).start();
        PortfolioStatsFeed::new(portfolio_addrs).start()
    });

    system.run().expect("Failed to run the system");
}

fn get_portfolios(security_cache: &'static RwLock<SecurityCache>) -> Vec<Portfolio> {
    vec![
        Portfolio::new(String::from("RMCF"), security_cache),
        Portfolio::new(String::from("ATAR"), security_cache),
        Portfolio::new(String::from("COLT"), security_cache),
    ]
}
