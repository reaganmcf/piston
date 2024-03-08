use actix::{Message, Addr};
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::security_cache::SecurityCache;

pub type SecurityId = u32;
pub type PositionId = u32;

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub code: String,
    pub positions: HashMap<PositionId, Position>,
    pub pnl: f64,
    pub security_cache: Addr<SecurityCache>,
    pub trade_count: u32,
}

#[derive(Debug, Clone)]
pub struct Security {
    pub id: SecurityId,
    pub ticker: String,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub id: PositionId,
    pub security: Security,
    pub cost_basis: f64,
    pub size: u32,
    pub unrealized_pnl: f64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Tick {
    pub security_id: SecurityId,
    pub price: f64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Trade {
    pub portfolio_code: String,
    pub trade_type: TradeType
}

#[derive(Debug)]
pub enum TradeType {
    Open(Position),
    Close(PositionId),
}


// TODO - these should be fed through into some global cache
lazy_static! {
    pub static ref AAPL: Security = Security {
        ticker: String::from("AAPL"),
        id: 0
    };
    pub static ref TSLA: Security = Security {
        ticker: String::from("TSLA"),
        id: 1
    };
}
