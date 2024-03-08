use actix::prelude::*;
use dotenv::dotenv;
use futures::future::try_join_all;
use log::{debug, error, info};
use security_cache::{GetLatestPrice, SecurityCache};
use stats::{PortfolioStatsEvent, PortfolioStatsFeed};
use std::{collections::HashMap, future::IntoFuture, time::Duration};
use tick_feed::TickFeed;
use trade_feed::TradeFeed;

mod models;
mod security_cache;
mod stats;
mod tick_feed;
mod trade_feed;
use models::*;

impl Portfolio {
    pub fn new(code: String, security_cache: Addr<SecurityCache>) -> Self {
        Self {
            code,
            security_cache,
            positions: HashMap::default(),
            pnl: 0f64,
            trade_count: 0,
        }
    }
}

impl Actor for Portfolio {
    type Context = Context<Self>;
}

impl Handler<Trade> for Portfolio {
    type Result = ();

    fn handle(&mut self, msg: Trade, _: &mut Self::Context) -> Self::Result {
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
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: PortfolioStatsEvent, ctx: &mut Self::Context) -> Self::Result {
        let security_cache = self.security_cache.clone();
        let price_futures = self
            .positions
            .clone()
            .into_values()
            .map(move |p| security_cache.send(GetLatestPrice(p.security.id)));

        Box::pin(async move {
            let prices = try_join_all(price_futures);

            match prices.await {
                Ok(p) => info!("got prices!!!!, {:?}", p),
                Err(e) => error!("{}", e),
            }
        })

        // for (_, p) in &mut self.positions {
        //     let price = self
        //         .security_cache
        //         .send(GetLatestPrice(p.id))
        //         .into_actor(self)
        //         .then(|result, act, _| {
        //             match result {
        //                 Ok(price) => {
        //                     let new_price = f64::from(p.size) * price.expect("no such security found");
        //                     p.unrealized_pnl = new_price - p.cost_basis;
        //                 },
        //                 Err(e) => error!("{}", e)
        //             }
        //         });

        //     ctx.wait(price)?;
        // }
    }
}

fn main() {
    dotenv().ok();
    env_logger::init();
    let system = System::new();

    let timescale = Duration::from_millis(1);

    system.block_on(async {
        let security_cache = SecurityCache::new().start();

        let portfolios = get_portfolios(security_cache.clone());
        let portfolio_addr_map: HashMap<_, _> = portfolios
            .into_iter()
            .map(|p| (p.code.clone(), p.start()))
            .collect();
        let portfolio_addrs: Vec<_> = portfolio_addr_map.values().into_iter().cloned().collect();

        TickFeed::new(security_cache, timescale).start();
        TradeFeed::new(portfolio_addr_map, timescale).start();
        PortfolioStatsFeed::new(portfolio_addrs).start()
    });

    system.run().expect("Failed to run the system");
}

fn get_portfolios(security_cache: Addr<SecurityCache>) -> Vec<Portfolio> {
    vec![
        Portfolio::new(String::from("RMCF"), security_cache.clone()),
        Portfolio::new(String::from("ATAR"), security_cache.clone()),
        Portfolio::new(String::from("COLT"), security_cache),
    ]
}
