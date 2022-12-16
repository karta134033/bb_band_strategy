use std::collections::VecDeque;

use crate::types::Kline;

pub fn sma(days: i64, klines: VecDeque<Kline>) {
    klines.iter().for_each(|kline| {
        println!("{:?}", kline);
    });
}
