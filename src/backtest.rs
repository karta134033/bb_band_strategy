use chrono::NaiveDateTime;
use log::{info, warn};
use std::{collections::VecDeque, time::Instant};

use crate::{
    strategy,
    types::{self, BbBandConfig},
    utils, TradeSide,
};
use strategy::prev_bb_band_entry;
use types::{Kline, StrategyType};
use utils::calculate_fee;

pub fn backtest(config: &BbBandConfig, klines: &Vec<Kline>) {
    // Config
    let initial_captial = config.initial_captial;
    let take_profit_percentage = config.take_profit_percentage;
    let stop_loss_percentage = config.stop_loss_percentage;
    let fee_rate = config.fee_rate;
    let leverage = config.leverage;
    let strategy_type = config.strategy_type;
    let entry_protion = config.entry_protion;

    // Variables
    let mut usd_balance = initial_captial;
    let mut position = 0.;
    let mut entry_price = 0.;
    let mut entry_side = TradeSide::None;
    let mut take_profit_price = 0.;
    let mut stop_loss_price = 0.;
    let mut win = 0;
    let mut lose = 0;
    let mut total_fee = 0.;
    let mut total_profit = 0.;
    let mut max_usd = usd_balance;

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
            if entry_side == TradeSide::None {
                let entry_size = if strategy_type == StrategyType::Single {
                    initial_captial.min(usd_balance) * entry_protion / curr_price
                } else {
                    usd_balance * entry_protion / curr_price
                };

                if let Some(order) = prev_bb_band_entry(prev_kline, prev_bb_band, entry_size) {
                    position = order.position;
                    entry_price = curr_price;
                    entry_side = order.side;
                    let fee = calculate_fee(fee_rate, entry_price, position, leverage);
                    take_profit_price =
                        entry_price * (1. + take_profit_percentage * entry_side.value());
                    stop_loss_price =
                        entry_price * (1. - stop_loss_percentage * entry_side.value());
                    total_fee += fee;
                    usd_balance -= fee;
                }
            } else if entry_side == TradeSide::Buy {
                let mid_price = (curr_kline.high + curr_kline.low) / 2.;
                // Assume we will keep tracking the price, not just tracking 15m kline
                let exit_price = if mid_price >= take_profit_price {
                    Some(take_profit_price)
                } else if mid_price <= stop_loss_price {
                    Some(stop_loss_price)
                } else {
                    None
                };
                if exit_price.is_some() {
                    // Calculate profit
                    let fee = calculate_fee(fee_rate, exit_price.unwrap(), position, leverage);
                    let profit = (exit_price.unwrap() - entry_price) * position * leverage as f64;
                    usd_balance -= fee;
                    usd_balance += profit;
                    total_fee += fee;
                    total_profit += profit;
                    if profit - fee >= 0. {
                        win += 1;
                    } else {
                        lose += 1;
                    }
                    max_usd = max_usd.max(usd_balance);
                    trade_log(
                        win,
                        lose,
                        usd_balance,
                        position,
                        entry_side,
                        entry_price,
                        exit_price,
                        profit,
                        &klines[index],
                    );

                    // Init trade
                    position = 0.;
                    entry_price = 0.;
                    entry_side = TradeSide::None;
                    take_profit_price = 0.;
                    stop_loss_price = 0.;
                }
            } else if entry_side == TradeSide::Sell {
                let mid_price = (curr_kline.high + curr_kline.low) / 2.;
                // Assume we will keep tracking the price, not just tracking 15m kline
                let exit_price = if mid_price <= take_profit_price {
                    Some(take_profit_price)
                } else if mid_price >= stop_loss_price {
                    Some(stop_loss_price)
                } else {
                    None
                };
                if exit_price.is_some() {
                    // Calculate profit
                    let fee = calculate_fee(fee_rate, exit_price.unwrap(), position, leverage);
                    let profit = (entry_price - exit_price.unwrap()) * position * leverage as f64;
                    usd_balance -= fee;
                    usd_balance += profit;
                    total_fee += fee;
                    total_profit += profit;
                    if profit - fee >= 0. {
                        win += 1;
                    } else {
                        lose += 1;
                    }
                    max_usd = max_usd.max(usd_balance);
                    trade_log(
                        win,
                        lose,
                        usd_balance,
                        position,
                        entry_side,
                        entry_price,
                        exit_price,
                        profit,
                        &klines[index],
                    );

                    // Init trade
                    position = 0.;
                    entry_price = 0.;
                    entry_side = TradeSide::None;
                    take_profit_price = 0.;
                    stop_loss_price = 0.;
                }
            }
        }
    }
    info!(
        "total_fee: {}, total_profit: {}, usd_balance: {}, max_usd: {}",
        total_fee, total_profit, usd_balance, max_usd
    );
    info!("elapsed: {}", timer.elapsed().as_secs());
}

fn trade_log(
    win: i64,
    lose: i64,
    usd_balance: f64,
    position: f64,
    entry_side: TradeSide,
    entry_price: f64,
    exit_price: Option<f64>,
    profit: f64,
    curr_kline: &Kline,
) {
    let curr_date = NaiveDateTime::from_timestamp_millis(curr_kline.close_time).unwrap();
    let msg = format!("win: {}, lose: {}, usd_balance: {}, position: {}, entry_price: {}, exit_price: {:?}, side: {:?}, profit: {}, date: {:?}",
    win, lose, usd_balance, position, entry_price, exit_price, entry_side, profit, curr_date);
    if profit >= 0. {
        info!("{}", msg);
    } else {
        warn!("{}", msg);
    }
}
