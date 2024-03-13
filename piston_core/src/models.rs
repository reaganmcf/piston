use actix::Message;
use lazy_static::lazy_static;
use piston_shared::*;
use std::{collections::HashMap, sync::Arc};

use crate::security_cache::SecurityCache;

// TODO - probably should be refactored to be in shared
#[derive(Debug, Clone)]
pub struct Portfolio {
    pub code: String,
    pub positions: HashMap<PositionId, Position>,
    pub pnl: f64,
    pub security_cache: Arc<SecurityCache>,
    pub trade_count: u32,
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
    pub trade_type: TradeType,
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
    pub static ref SECURITY_UNIVERSE: Vec<Security> = vec![
        Security { id: 1, ticker: "AAPL".to_string() }, // Apple Inc.
        Security { id: 2, ticker: "MSFT".to_string() }, // Microsoft Corporation
        Security { id: 3, ticker: "AMZN".to_string() }, // Amazon.com, Inc.
        Security { id: 4, ticker: "GOOGL".to_string() }, // Alphabet Inc. (Google)
        Security { id: 5, ticker: "FB".to_string() },    // Facebook, Inc.
        Security { id: 6, ticker: "BRK.A".to_string() }, // Berkshire Hathaway Inc.
        Security { id: 7, ticker: "V".to_string() },     // Visa Inc.
        Security { id: 8, ticker: "TSLA".to_string() },  // Tesla, Inc.
        Security { id: 9, ticker: "JNJ".to_string() },   // Johnson & Johnson
        Security { id: 10, ticker: "WMT".to_string() },  // Walmart Inc.
        Security { id: 11, ticker: "JPM".to_string() },  // JPMorgan Chase & Co.
        Security { id: 12, ticker: "MA".to_string() },   // Mastercard Incorporated
        Security { id: 13, ticker: "PG".to_string() },   // The Procter & Gamble Company
        Security { id: 14, ticker: "UNH".to_string() },  // UnitedHealth Group Incorporated
        Security { id: 15, ticker: "DIS".to_string() },  // The Walt Disney Company
        Security { id: 16, ticker: "NVDA".to_string() }, // NVIDIA Corporation
        Security { id: 17, ticker: "HD".to_string() },   // The Home Depot, Inc.
        Security { id: 18, ticker: "PYPL".to_string() }, // PayPal Holdings, Inc.
        Security { id: 19, ticker: "BAC".to_string() },  // Bank of America Corporation
        Security { id: 20, ticker: "VZ".to_string() },   // Verizon Communications Inc.
        Security { id: 21, ticker: "ADBE".to_string() }, // Adobe Inc.
        Security { id: 22, ticker: "CMCSA".to_string() },// Comcast Corporation
        Security { id: 23, ticker: "NFLX".to_string() }, // Netflix, Inc.
        Security { id: 24, ticker: "KO".to_string() },   // The Coca-Cola Company
        Security { id: 25, ticker: "NKE".to_string() },  // NIKE, Inc.
    ];
}
