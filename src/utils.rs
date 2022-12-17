use crate::types::{BollingerBand, Kline};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

pub fn sma(current_index: usize, days: usize, klines: &Vec<Kline>) -> Option<f64> {
    if current_index >= days - 1 {
        let mut close_sum = 0.;
        let start_index = current_index - (days - 1);
        for index in start_index..=current_index {
            close_sum += &klines[index].close;
        }
        let sma = close_sum / days as f64;
        Some(sma)
    } else {
        None
    }
}

pub fn deviation(current_index: usize, days: usize, klines: &Vec<Kline>) -> Option<f64> {
    if let Some(sma) = sma(current_index, days, klines) {
        let start_index = current_index - (days - 1);
        let mut diff_sum = 0.;
        for index in start_index..=current_index {
            let diff = klines[index].close - sma;
            diff_sum += diff * diff;
        }
        let dev = f64::sqrt(diff_sum / days as f64);
        Some(dev)
    } else {
        None
    }
}

pub fn bollinger_band(
    current_index: usize,
    days: usize,
    width: f64,
    klines: &Vec<Kline>,
) -> Option<BollingerBand> {
    let sma_opt = sma(current_index, days, klines);
    let dev_opt = deviation(current_index, days, klines);
    if sma_opt.is_some() && dev_opt.is_some() {
        let sma = sma_opt.unwrap();
        let dev = dev_opt.unwrap();
        let timestamp_sec = klines[current_index].close_time / 1000;
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

pub fn calculate_fee(fee_rate: f64, price: f64, size: f64, leverage: f64) -> f64 {
    fee_rate * price * size * leverage
}
