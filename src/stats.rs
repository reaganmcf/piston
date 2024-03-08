use std::time::Duration;

use log::info;
use xtra::prelude::*;

use crate::models::*;

#[derive(Clone)]
pub struct PortfolioStatsFeed {
    subs: Vec<Address<Portfolio>>,
}

impl PortfolioStatsFeed {
    pub fn new(portfolios: Vec<Address<Portfolio>>) -> Self {
        Self { subs: portfolios }
    }
}

impl Actor for PortfolioStatsFeed {
    type Stop = ();

    async fn started(&mut self, _mailbox: &Mailbox<Self>) -> Result<(), Self::Stop> {
        info!("Started PortfolioStatsFeed");

        let act = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;

                for sub in &act.subs {
                    sub.send(PortfolioStatsEvent {})
                        .await
                        .expect("Failed to send Stats event to portfolio");
                }
            }
        });

        Ok(())
    }

    async fn stopped(self) -> Self::Stop {}
}

pub struct PortfolioStatsEvent;
