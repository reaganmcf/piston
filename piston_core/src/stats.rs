use actix::prelude::*;
use log::info;
use std::time::Duration;

use crate::portfolio::Portfolio;

pub struct PortfolioStatsFeed {
    subs: Vec<Addr<Portfolio>>,
}

impl PortfolioStatsFeed {
    pub fn new(portfolios: Vec<Addr<Portfolio>>) -> Self {
        Self { subs: portfolios }
    }
}

impl Actor for PortfolioStatsFeed {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started PortfolioStatsFeed");
        ctx.run_interval(Duration::from_secs(1), move |act, _| {
            for sub in &act.subs {
                sub.do_send(PortfolioStatsEvent {});
            }
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PortfolioStatsEvent;
