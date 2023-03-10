use anyhow::Result;
use log::info;
use std::{fs::File, path::Path};

use crate::{
    backtest::backtest,
    types::{BbBandConfig, HypertuneConfig, Kline},
};

pub fn hypertune(
    config: &BbBandConfig,
    hypertune_config: &HypertuneConfig,
    klines: &Vec<Kline>,
) -> Result<()> {
    info!("hypertune_config: {:?}", hypertune_config);
    let take_profit_percentage_max = hypertune_config.take_profit_percentage_max;
    let stop_loss_percentage_max = hypertune_config.stop_loss_percentage_max;
    let bb_width_max = hypertune_config.bb_width_max;

    let output_path = Path::new("output.csv");
    let file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(file);
    writer.write_record(&[
        "initial_captial",
        "usd_balance",
        "max_usd",
        "min_usd",
        "win",
        "lose",
        "win_rate",
        "total_fee",
        "total_profit",
        "take_profit_percentage",
        "stop_loss_percentage",
        "bb_width",
    ])?;
    let mut trial_config = config.clone();
    trial_config.take_profit_percentage = hypertune_config.take_profit_percentage_min;
    trial_config.bb_width = hypertune_config.bb_width_min;

    while trial_config.take_profit_percentage <= take_profit_percentage_max {
        trial_config.stop_loss_percentage = hypertune_config.stop_loss_percentage_min;

        while trial_config.stop_loss_percentage <= stop_loss_percentage_max {
            trial_config.bb_width = hypertune_config.bb_width_min;

            while trial_config.bb_width <= bb_width_max {
                let metric = backtest(&trial_config, klines);
                let mut record = Vec::new();
                record.push(metric.initial_captial.to_string());
                record.push(metric.usd_balance.to_string());
                record.push(metric.max_usd.to_string());
                record.push(metric.min_usd.to_string());
                record.push(metric.win.to_string());
                record.push(metric.lose.to_string());
                record.push((metric.win as f64 / (metric.win + metric.lose) as f64).to_string());
                record.push(metric.total_fee.to_string());
                record.push(metric.total_profit.to_string());
                record.push(trial_config.take_profit_percentage.to_string());
                record.push(trial_config.stop_loss_percentage.to_string());
                record.push(metric.bb_width.to_string());
                writer.write_record(&record)?;
                writer.flush()?;
                trial_config.bb_width += hypertune_config.bb_width_step;
            }
            trial_config.stop_loss_percentage += hypertune_config.stop_loss_percentage_step;
        }
        trial_config.take_profit_percentage += hypertune_config.take_profit_percentage_step;
    }

    Ok(())
}
