use crate::{
    types::{BollingerBand, Kline, OrderSpec},
    TradeSide,
};

pub fn prev_bb_band_entry(
    prev_kline: &Kline,
    prev_bb_band: &BollingerBand,
    entry_size: f64,
) -> Option<OrderSpec> {
    let prev_price = prev_kline.close;
    if prev_price > prev_bb_band.up || prev_price < prev_bb_band.down {
        let side = if prev_price > prev_bb_band.up {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };
        let position = entry_size;
        Some(OrderSpec { position, side })
    } else {
        None
    }
}
