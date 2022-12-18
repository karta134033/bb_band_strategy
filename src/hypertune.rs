use anyhow::Result;
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
    let take_profit_percentage_max = 0.2;
    let stop_loss_percentage_max = 0.2;
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
    ])?;
    let mut trial_config = config.clone();
    while trial_config.stop_loss_percentage <= stop_loss_percentage_max {
        trial_config.take_profit_percentage += hypertune_config.take_profit_percentage_step;
        let metric = backtest(&trial_config, klines);
        writer.write_record(&metric.csv_record())?;
        writer.flush()?;
        if trial_config.take_profit_percentage > take_profit_percentage_max {
            trial_config.take_profit_percentage = config.take_profit_percentage;
            trial_config.stop_loss_percentage += config.take_profit_percentage;
        }
    }

    Ok(())
}
