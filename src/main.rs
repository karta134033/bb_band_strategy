use anyhow::Result;
use bb_band::{
    backtest::backtest,
    hypertune::hypertune,
    types::{BbBandConfig, Cli, HypertuneConfig, Mode},
    utils::get_klines_from_db,
};
use clap::Parser;
use log::{info, LevelFilter};
use simplelog::*;
use std::fs::File;

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // WriteLogger::new(
        //     LevelFilter::Info,
        //     Config::default(),
        //     File::create("log_file.log").unwrap(),
        // ),
    ])
    .unwrap();

    let args = Cli::parse();
    let config_file = File::open(args.config_path)?;
    let config: BbBandConfig = serde_json::from_reader(config_file)?;
    info!("config: {:#?}", config);
    let klines = get_klines_from_db(&config);
    match args.mode {
        Mode::Backtest => {
            backtest(&config, &klines);
        }
        Mode::Hypertune => {
            let hypertune_config_file = File::open(args.hypertune_config.unwrap())?;
            let hypertune_config: HypertuneConfig = serde_json::from_reader(hypertune_config_file)?;
            info!("hypertune_config: {:#?}", hypertune_config);
            let _ = hypertune(&config, &hypertune_config, &klines);
        }
    }
    Ok(())
}
