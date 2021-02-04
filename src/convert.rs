use tinkoff_api::models::MarketInstrument;

use crate::faces::Stock;


impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
        }
    }
}