use piston_shared::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    Ping(Ping),
    Pong(Pong),
    PortfolioStats(PortfolioStats),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pong;
