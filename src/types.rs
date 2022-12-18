use std::path::PathBuf;

use crate::TradeSide;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(short = 'f')]
    pub file_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BbBandConfig {
    pub from: (i32, u32, u32), // y, m , d
    pub to: (i32, u32, u32),   // y, m , d
    pub initial_captial: f64,
    pub take_profit_percentage: f64,
    pub stop_loss_percentage: f64,
    pub fee_rate: f64,
    pub leverage: u64,
    pub strategy_type: StrategyType,
    pub entry_protion: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kline {
    pub open_time: i64,
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BollingerBand {
    pub up: f64,
    pub sma: f64,
    pub down: f64,
    pub dev: f64,
    pub date_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum StrategyType {
    Single,
    Compound,
}

pub struct OrderSpec {
    pub position: f64,
    pub side: TradeSide,
}

pub struct TradeLog {
    pub entry_side: i64,
    pub entry_price: f64,
    pub entry_size: f64,
    pub exit_price: f64,
}
