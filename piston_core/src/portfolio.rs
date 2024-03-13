use crate::{models::*, security_cache::SecurityCache, stats::PortfolioStatsEvent};
use actix::{Actor, Context, Handler};
use log::{debug, info};
use piston_ipc::{messages::IpcMessage, IpcWriter};
use piston_shared::*;
use std::{collections::HashMap, sync::RwLock};

#[derive(Debug)]
pub struct Portfolio {
    pub code: String,
    positions: HashMap<PositionId, Position>,
    pnl: f64,
    security_cache: &'static RwLock<SecurityCache>,
    trade_count: u32,

    ipc_writer: IpcWriter,
}

impl Portfolio {
    pub fn new(code: String, security_cache: &'static RwLock<SecurityCache>) -> Self {
        Self {
            code,
            security_cache,
            positions: HashMap::default(),
            pnl: 0f64,
            trade_count: 0,

            ipc_writer: IpcWriter::new().expect("Failed to create ipc writer"),
        }
    }

    pub fn recalculate_positions(&mut self) {
        for p in self.positions.values_mut() {
            let cache = self
                .security_cache
                .read()
                .expect("could not read security cache");

            let latest_price = cache
                .get_latest_price(p.security.id)
                .expect("Unknown price for secrurity");

            p.unrealized_pnl = (f64::from(p.size) * latest_price) - p.cost_basis;
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
    type Result = ();

    fn handle(&mut self, _msg: PortfolioStatsEvent, _: &mut Self::Context) -> Self::Result {
        self.recalculate_positions();
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

        self.ipc_writer
            .send(&IpcMessage::PortfolioStats(PortfolioStats {
                code: self.code.clone(),
                positions: self.positions.values().cloned().collect(),
                trade_count: self.trade_count,
                pnl: self.pnl,
                unrealized_pnl,
            }))
            .expect("Failed to send portfolio stats");
    }
}
