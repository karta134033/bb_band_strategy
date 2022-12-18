use std::{path::PathBuf, str::FromStr};

use crate::TradeSide;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(short = 'f')]
    pub config_path: PathBuf,
    #[arg(short = 'm')]
    pub mode: Mode,
    #[arg(short = 't')]
    pub hypertune_config: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Backtest,
    Hypertune,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backtest" => Ok(Mode::Backtest),
            "hypertune" => Ok(Mode::Hypertune),
            "b" => Ok(Mode::Backtest),
            "h" => Ok(Mode::Hypertune),
            _ => Err(format!("Invalid mode: {}", s)),
        }
    }
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

#[derive(Debug, Clone)]
pub struct BacktestMetric {
    pub initial_captial: f64,
    pub usd_balance: f64,
    pub position: f64,
    pub entry_price: f64,
    pub entry_side: TradeSide,
    pub take_profit_price: f64,
    pub stop_loss_price: f64,
    pub win: usize,
    pub lose: usize,
    pub total_fee: f64,
    pub total_profit: f64,
    pub max_usd: f64,
    pub min_usd: f64,
}

impl BacktestMetric {
    pub fn new(config: &BbBandConfig) -> BacktestMetric {
        BacktestMetric {
            initial_captial: config.initial_captial,
            usd_balance: config.initial_captial,
            position: 0.,
            entry_price: 0.,
            entry_side: TradeSide::None,
            take_profit_price: 0.,
            stop_loss_price: 0.,
            win: 0,
            lose: 0,
            total_fee: 0.,
            total_profit: 0.,
            max_usd: config.initial_captial,
            min_usd: config.initial_captial,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HypertuneConfig {
    pub take_profit_percentage_step: f64,
    pub stop_loss_percentage_step: f64,
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
