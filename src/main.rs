use async_std::task;
use bb_band::{
    mongo_client::MongoClient, utils, BTCUSDT_15M, KLINE_DB, LOCAL_MONGO_CONNECTION_STRING,
};

fn main() {
    let mongo_clinet = task::block_on(MongoClient::new(LOCAL_MONGO_CONNECTION_STRING));
    let klines = task::block_on(mongo_clinet.get_klines(
        KLINE_DB,
        BTCUSDT_15M,
        1604160899999,
        Some(1604160899999),
    ));
    utils::sma(20, klines);
}
