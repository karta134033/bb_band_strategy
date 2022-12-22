use log::info;
use std::time::Instant;

use crate::{
    strategy_pool::bb_swing::BBSwing,
    types::{self, BacktestMetric, BbBandConfig},
};
use types::Kline;

pub fn backtest(config: &BbBandConfig, klines: &Vec<Kline>) -> BacktestMetric {
    // Variables
    let mut metric = BacktestMetric::new(config);
    let timer = Instant::now();
    metric.bb_width = config.bb_width;

    let mut bb_swing = BBSwing::new(&config);

    for index in 0..klines.len() {
        bb_swing.strategy(&mut metric, &klines[index]);
    }
    info!(
        "total_fee: {}, total_profit: {}, usd_balance: {}, max_usd: {}",
        metric.total_fee, metric.total_profit, metric.usd_balance, metric.max_usd
    );
    info!("elapsed: {}", timer.elapsed().as_secs());
    metric
}
