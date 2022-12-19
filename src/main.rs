use anyhow::Result;
use bb_band::{
    backtest::backtest,
    hypertune::hypertune,
    types::{BbBandConfig, Cli, HypertuneConfig, Mode},
    utils::get_klines_from_db,
};
use clap::Parser;
use log::info;
use slog::o;
use slog::Drain;
use std::fs::File;
use std::fs::OpenOptions;

fn main() -> Result<()> {
    // Log level
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Output logs
    let log_path = "log_file.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let _log = slog::Logger::root(drain, o!());

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
