use std::time::Duration;

use crate::{models::*, security_cache::SecurityCache};
use log::{debug, error, info};
use rand::{rngs::ThreadRng, Rng};
use xtra::prelude::*;

#[derive(Clone)]
pub struct TickFeed {
    timescale: Duration,
    security_cache: Address<SecurityCache>,
}

impl TickFeed {
    pub fn new(security_cache: Address<SecurityCache>, timescale: Duration) -> Self {
        Self {
            timescale,
            security_cache,
        }
    }
}

impl Actor for TickFeed {
    type Stop = ();

    async fn started(&mut self, _mailbox: &Mailbox<Self>) -> Result<(), Self::Stop> {
        info!("Started TickFeed");
        let act = self.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(act.timescale).await;
                debug!("Sending tick data for AAPL...");

                let tick = Tick {
                    security_id: AAPL.id,
                    price: ThreadRng::default().gen_range(100f64..200f64),
                };
                if let Err(e) = act.security_cache.send(tick).await {
                    error!("Failed to send, {:?}", e);
                }

                debug!(" Complete!");
            }
        });

        Ok(())
    }

    async fn stopped(self) -> Self::Stop {}
}
