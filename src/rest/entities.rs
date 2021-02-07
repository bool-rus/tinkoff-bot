use tinkoff_api::apis::Error;
use crate::faces::{Candle, DateTime, Interval, Stock};

#[derive(Clone)]
pub enum Request {
    GetStocks,
    GetETFs,
    GetBonds,
    GetCandles { figi: String, from: DateTime, to: String, interval: Interval},
}

pub enum Response {
    Err(Request, ErrX),
    Stocks(Vec<Stock>),
    ETFs(Vec<Stock>),
    Bonds(Vec<Stock>),
    Candles { figi: String, candles: Vec<Candle>},
}

#[derive(Debug)]
pub struct ErrX{
    msg: String,
}

impl <T: std::fmt::Debug> From<Error<T>> for ErrX {
    fn from(e: Error<T>) -> Self {
        Self {
            msg: format!("{:?}",e),
        }
    }
}