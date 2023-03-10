use std::collections::VecDeque;

use crate::{
    mongo_client::MongoClient,
    types::{BacktestMetric, BbBandConfig, BollingerBand, Kline},
    TradeSide, BTCUSDT_15M, KLINE_DB, LOCAL_MONGO_CONNECTION_STRING,
};
use async_std::task;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

pub fn sma(days: usize, klines: &VecDeque<Kline>) -> Option<f64> {
    if klines.len() >= days {
        let start_index = klines.len() - days;
        let end_index = klines.len() - 1;
        let mut close_sum = 0.;
        for index in start_index..=end_index {
            close_sum += &klines[index].close;
        }
        let sma = close_sum / days as f64;
        Some(sma)
    } else {
        None
    }
}

pub fn deviation(days: usize, klines: &VecDeque<Kline>) -> Option<f64> {
    if let Some(sma) = sma(days, klines) {
        let start_index = klines.len() - days;
        let end_index = klines.len() - 1;
        let mut diff_sum = 0.;
        for index in start_index..=end_index {
            let diff = klines[index].close - sma;
            diff_sum += diff * diff;
        }
        let dev = f64::sqrt(diff_sum / days as f64);
        Some(dev)
    } else {
        None
    }
}

pub fn bollinger_band(days: usize, width: f64, klines: &VecDeque<Kline>) -> Option<BollingerBand> {
    let sma_opt = sma(days, klines);
    let dev_opt = deviation(days, klines);
    if sma_opt.is_some() && dev_opt.is_some() {
        let sma = sma_opt.unwrap();
        let dev = dev_opt.unwrap();
        let timestamp_sec = klines[days - 1].close_time / 1000;
        let naive = NaiveDateTime::from_timestamp_opt(timestamp_sec, 0).unwrap();
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        Some(BollingerBand {
            up: sma + width * dev,
            sma,
            down: sma - width * dev,
            dev,
            date_time: datetime.to_string(),
        })
    } else {
        None
    }
}

pub fn datetime_to_ts_ms(year: i32, month: u32, day: u32) -> i64 {
    let naive_date = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    DateTime::<Utc>::from_utc(naive_date, Utc).timestamp_millis()
}

pub fn calculate_fee(fee_rate: f64, price: f64, size: f64, leverage: u64) -> f64 {
    fee_rate * price * size * leverage as f64
}

pub fn get_klines_from_db(config: &BbBandConfig) -> Vec<Kline> {
    let from_ts_ms = datetime_to_ts_ms(config.from.0, config.from.1, config.from.2);
    let to_ts_ms = datetime_to_ts_ms(config.to.0, config.to.1, config.to.2);

    let mongo_clinet = task::block_on(MongoClient::new(LOCAL_MONGO_CONNECTION_STRING));
    let klines =
        task::block_on(mongo_clinet.get_klines(KLINE_DB, BTCUSDT_15M, from_ts_ms, Some(to_ts_ms)));
    klines
}

pub fn init_trade(metric: &mut BacktestMetric) {
    metric.position = 0.;
    metric.entry_price = 0.;
    metric.entry_side = TradeSide::None;
    metric.take_profit_price = 0.;
    metric.stop_loss_price = 0.;
    metric.fee = 0.;
    metric.profit = 0.;
}
