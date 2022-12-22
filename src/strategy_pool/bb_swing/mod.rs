use std::collections::VecDeque;

use chrono::NaiveDateTime;
use log::{info, warn};

use crate::{
    strategy::prev_bb_band_entry,
    types::{BacktestMetric, BbBandConfig, BollingerBand, Kline, StrategyType},
    utils::{self, calculate_fee, init_trade},
    TradeSide,
};

const DAYS: usize = 20;

// bb down buy, bb up sell
pub struct BBSwing {
    klines: VecDeque<Kline>,
    bb_bands: VecDeque<Option<BollingerBand>>,
    strategy_type: StrategyType,
    initial_captial: f64,
    take_profit_percentage: f64,
    stop_loss_percentage: f64,
    fee_rate: f64,
    leverage: u64,
    entry_protion: f64,
}

impl BBSwing {
    pub fn new(config: &BbBandConfig) -> Self {
        BBSwing {
            klines: VecDeque::new(),
            bb_bands: VecDeque::new(),
            strategy_type: config.strategy_type,
            initial_captial: config.initial_captial,
            take_profit_percentage: config.take_profit_percentage,
            stop_loss_percentage: config.stop_loss_percentage,
            fee_rate: config.fee_rate,
            leverage: config.leverage,
            entry_protion: config.entry_protion,
        }
    }

    pub fn strategy(&mut self, metric: &mut BacktestMetric, kline: &Kline) {
        self.klines.push_back(kline.clone());
        let bb_band = utils::bollinger_band(DAYS, 2., &self.klines);
        self.bb_bands.push_back(bb_band);
        if self.klines.len() > DAYS {
            let index = self.klines.len() - 1;
            let curr_kline = &self.klines[index];
            let curr_price = (curr_kline.high + curr_kline.low) / 2.;
            let prev_kline = &self.klines[index - 1];
            let prev_bb_band = self.bb_bands[index - 1].as_ref().unwrap();
            if metric.entry_side == TradeSide::None {
                let entry_size = if self.strategy_type == StrategyType::Single {
                    self.initial_captial.min(metric.usd_balance) * self.entry_protion / curr_price
                } else {
                    metric.usd_balance * self.entry_protion / curr_price
                };

                if let Some(order) = prev_bb_band_entry(prev_kline, prev_bb_band, entry_size) {
                    metric.position = order.position;
                    metric.entry_price = curr_price;
                    metric.entry_side = order.side;
                    let fee = calculate_fee(
                        self.fee_rate,
                        metric.entry_price,
                        metric.position,
                        self.leverage,
                    );
                    metric.take_profit_price = metric.entry_price
                        * (1. + self.take_profit_percentage * metric.entry_side.value());
                    metric.stop_loss_price = metric.entry_price
                        * (1. - self.stop_loss_percentage * metric.entry_side.value());
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
                    let fee = calculate_fee(
                        self.fee_rate,
                        exit_price.unwrap(),
                        metric.position,
                        self.leverage,
                    );
                    let profit = (exit_price.unwrap() - metric.entry_price)
                        * metric.position
                        * self.leverage as f64;
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
                    trade_log(&metric, &self.klines[index]);
                    init_trade(metric);
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
                    let fee = calculate_fee(
                        self.fee_rate,
                        exit_price.unwrap(),
                        metric.position,
                        self.leverage,
                    );
                    let profit = (metric.entry_price - exit_price.unwrap())
                        * metric.position
                        * self.leverage as f64;
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
                    trade_log(&metric, &self.klines[index]);
                    init_trade(metric);
                }
            }
            self.bb_bands.pop_front();
            self.klines.pop_front();
        }
    }
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
