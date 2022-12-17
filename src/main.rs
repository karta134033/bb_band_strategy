use chrono::NaiveDateTime;
use log::{info, warn};
use simple_logger::SimpleLogger;
use std::{collections::VecDeque, time::Instant};

use async_std::task;
use bb_band::{
    mongo_client::MongoClient,
    strategy::prev_bb_band_entry,
    types::{Kline, StrategyType},
    utils::{self, calculate_fee, datetime_to_ts_ms},
    TradeSide, BTCUSDT_15M, KLINE_DB, LOCAL_MONGO_CONNECTION_STRING,
};

fn main() {
    SimpleLogger::new().init().unwrap();

    // Config
    let from = (2019, 10, 30); // y, m , d
    let to = (2022, 11, 30); // y, m , d
    let initial_captial: f64 = 10000.;
    let take_profit_percentage = 0.006;
    let stop_loss_percentage = 0.005;
    let fee_rate = 0.04 / 100.;
    let leverage = 2.;
    let strategy_type = StrategyType::Compound;
    let entry_protion = 0.3;

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

    let timer = Instant::now();
    let from_ts_ms = datetime_to_ts_ms(from.0, from.1, from.2);
    let to_ts_ms = datetime_to_ts_ms(to.0, to.1, to.2);

    let mongo_clinet = task::block_on(MongoClient::new(LOCAL_MONGO_CONNECTION_STRING));
    let klines =
        task::block_on(mongo_clinet.get_klines(KLINE_DB, BTCUSDT_15M, from_ts_ms, Some(to_ts_ms)));

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
                    initial_captial.max(usd_balance) * entry_protion / curr_price
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
                    let profit = (exit_price.unwrap() - entry_price) * position * leverage;
                    usd_balance -= fee;
                    usd_balance += profit;
                    total_fee += fee;
                    total_profit += profit;
                    if profit - fee >= 0. {
                        win += 1;
                    } else {
                        lose += 1;
                    }
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
                    let profit = (entry_price - exit_price.unwrap()) * position * leverage;
                    usd_balance -= fee;
                    usd_balance += profit;
                    total_fee += fee;
                    total_profit += profit;
                    if profit - fee >= 0. {
                        win += 1;
                    } else {
                        lose += 1;
                    }
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
        "total_fee: {}, total_profit: {}, usd_balance: {}",
        total_fee, total_profit, usd_balance
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
