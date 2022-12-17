use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq)]
pub enum StrategyType {
    Single,
    Compound,
}

pub struct OrderSpec {
    pub position: f64,
    pub side: i64,
}
