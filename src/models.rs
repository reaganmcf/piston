use actix::Message;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub type SecurityId = u32;
pub type PositionId = u32;

#[derive(Debug)]
pub struct Portfolio {
    pub code: String,
    pub positions: HashMap<PositionId, Position>,
    pub pnl: f64
}

#[derive(Debug, Clone)]
pub struct Security {
    pub id: SecurityId,
    pub ticker: String,
}

#[derive(Debug)]
pub struct Position {
    pub id: PositionId,
    pub security: Security,
    pub size: u32,
    pub unrealized_pnl: f64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct TickData {
    pub security: Security,
    pub price: f64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum TradeEvent {
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

pub static mut NEXT_POSITION_ID: PositionId = 0;
