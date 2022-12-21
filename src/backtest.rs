use chrono::NaiveDateTime;
use log::{info, warn};
use std::{collections::VecDeque, time::Instant};

use crate::{
    strategy,
    types::{self, BacktestMetric, BbBandConfig},
    utils, TradeSide,
};
use strategy::prev_bb_band_entry;
use types::{Kline, StrategyType};
use utils::calculate_fee;

pub fn backtest(config: &BbBandConfig, klines: &Vec<Kline>) -> BacktestMetric {
    // Config
    let initial_captial = config.initial_captial;
    let take_profit_percentage = config.take_profit_percentage;
    let stop_loss_percentage = config.stop_loss_percentage;
    let fee_rate = config.fee_rate;
    let leverage = config.leverage;
    let strategy_type = config.strategy_type;
    let entry_protion = config.entry_protion;

    // Variables
    let mut metric = BacktestMetric::new(config);
    let timer = Instant::now();
    let mut bb_bands = VecDeque::new();
    for index in 0..klines.len() {
        let bb_band = utils::bollinger_band(index, 20, 2., &klines);
        bb_bands.push_back(bb_band);

        if index >= 20 {
            let curr_kline = &klines[index];
            let curr_price = (curr_kline.high + curr_kline.low) / 2.;
            let prev_kline = &klines[index - 1];
            let prev_bb_band = bb_bands[index - 1].as_ref().unwrap();
            if metric.entry_side == TradeSide::None {
                let entry_size = if strategy_type == StrategyType::Single {
                    initial_captial.min(metric.usd_balance) * entry_protion / curr_price
                } else {
                    metric.usd_balance * entry_protion / curr_price
                };

                if let Some(order) = prev_bb_band_entry(prev_kline, prev_bb_band, entry_size) {
                    metric.position = order.position;
                    metric.entry_price = curr_price;
                    metric.entry_side = order.side;
                    let fee =
                        calculate_fee(fee_rate, metric.entry_price, metric.position, leverage);
                    metric.take_profit_price = metric.entry_price
                        * (1. + take_profit_percentage * metric.entry_side.value());
                    metric.stop_loss_price = metric.entry_price
                        * (1. - stop_loss_percentage * metric.entry_side.value());
                    metric.total_fee += fee;
                    metric.usd_balance -= fee;
                }
            } else if metric.entry_side == TradeSide::Buy {
                // Assume we will keep tracking the price, not just tracking 15m kline
                let exit_price = if curr_kline.low <= metric.stop_loss_price {
                    Some(metric.stop_loss_price)
                } else if curr_kline.high >= metric.take_profit_price {
                    Some(metric.take_profit_price)
                } else {
                    None
                };
                if exit_price.is_some() {
                    // Calculate profit
                    let fee =
                        calculate_fee(fee_rate, exit_price.unwrap(), metric.position, leverage);
                    let profit = (exit_price.unwrap() - metric.entry_price)
                        * metric.position
                        * leverage as f64;
                    metric.usd_balance -= fee;
                    metric.usd_balance += profit;
                    metric.total_fee += fee;
                    metric.total_profit += profit;
                    if profit - fee >= 0. {
                        metric.win += 1;
                    } else {
                        metric.lose += 1;
                    }
                    metric.max_usd = metric.max_usd.max(metric.usd_balance);
                    metric.min_usd = metric.min_usd.min(metric.usd_balance);
                    metric.profit = profit;
                    metric.fee = fee;
                    metric.exit_price = exit_price.unwrap();
                    trade_log(&metric, &klines[index]);
                    init_trade(&mut metric);
                }
            } else if metric.entry_side == TradeSide::Sell {
                // Assume we will keep tracking the price, not just tracking 15m kline
                let exit_price = if curr_kline.high >= metric.stop_loss_price {
                    Some(metric.stop_loss_price)
                } else if curr_kline.low <= metric.take_profit_price {
                    Some(metric.take_profit_price)
                } else {
                    None
                };
                if exit_price.is_some() {
                    // Calculate profit
                    let fee =
                        calculate_fee(fee_rate, exit_price.unwrap(), metric.position, leverage);
                    let profit = (metric.entry_price - exit_price.unwrap())
                        * metric.position
                        * leverage as f64;
                    metric.usd_balance -= fee;
                    metric.usd_balance += profit;
                    metric.total_fee += fee;
                    metric.total_profit += profit;
                    if profit - fee >= 0. {
                        metric.win += 1;
                    } else {
                        metric.lose += 1;
                    }
                    metric.max_usd = metric.max_usd.max(metric.usd_balance);
                    metric.min_usd = metric.min_usd.min(metric.usd_balance);
                    metric.profit = profit;
                    metric.fee = fee;
                    metric.exit_price = exit_price.unwrap();
                    trade_log(&metric, &klines[index]);
                    init_trade(&mut metric);
                }
            }
        }
    }
    info!(
        "total_fee: {}, total_profit: {}, usd_balance: {}, max_usd: {}",
        metric.total_fee, metric.total_profit, metric.usd_balance, metric.max_usd
    );
    info!("elapsed: {}", timer.elapsed().as_secs());
    metric
}

fn init_trade(metric: &mut BacktestMetric) {
    metric.position = 0.;
    metric.entry_price = 0.;
    metric.entry_side = TradeSide::None;
    metric.take_profit_price = 0.;
    metric.stop_loss_price = 0.;
    metric.fee = 0.;
    metric.profit = 0.;
}

fn trade_log(metric: &BacktestMetric, curr_kline: &Kline) {
    let curr_date = NaiveDateTime::from_timestamp_millis(curr_kline.close_time).unwrap();
    let mut msg = "".to_string();
    msg += &format!("date: {:?}, ", curr_date);
    msg += &format!("win: {:?}, ", metric.win);
    msg += &format!("lose: {:?}, ", metric.lose);
    msg += &format!("usd_balance: {:.4}, ", metric.usd_balance);
    msg += &format!("position: {:.4}, ", metric.position);
    msg += &format!("entry_side: {:?}, ", metric.entry_side);
    msg += &format!("entry_price: {:.4}, ", metric.entry_price);
    msg += &format!("exit_price: {:.4}, ", metric.exit_price);
    msg += &format!("profit: {:.4}, ", metric.profit);
    msg += &format!("fee: {:.4}, ", metric.fee);

    if metric.profit >= 0. {
        info!("{}", msg);
    } else {
        warn!("{}", msg);
    }
}
