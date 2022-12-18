use crate::backtest::backtest;
use anyhow::Result;
use bb_band::{
    backtest,
    types::{BbBandConfig, Cli},
    utils::get_klines_from_db,
};
use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;
use std::fs::File;
fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let args = Cli::parse();
    let file = File::open(args.file_path)?;
    let config: BbBandConfig = serde_json::from_reader(file)?;
    info!("config: {:#?}", config);

    let klines = get_klines_from_db(&config);
    backtest(&config, &klines);
    Ok(())
}
