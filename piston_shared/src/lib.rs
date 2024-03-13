use serde::{Deserialize, Serialize};

pub type SecurityId = u32;
pub type PositionId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub code: String,
    pub positions: Vec<Position>,
    pub trade_count: u32,
    pub pnl: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    pub id: SecurityId,
    pub ticker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: PositionId,
    pub security: Security,
    pub cost_basis: f64,
    pub size: u32,
    pub unrealized_pnl: f64,
}
