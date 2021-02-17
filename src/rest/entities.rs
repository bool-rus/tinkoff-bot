use std::time::SystemTime;

use tinkoff_api::apis::Error;
use crate::model::{Candle, DateTime, Interval, Order, OrderState, Position, Stock};

#[derive(Clone, Debug)]
pub enum Request {
    Instruments,
    Candles { figi: String, from: DateTime, to: DateTime, interval: Interval},
    LimitOrder(SystemTime, Order),
    Portfolio,
}

#[derive(Debug)]
pub enum Response {
    Err(Request, ErrX),
    Stocks(Vec<Stock>),
    Candles { figi: String, candles: Vec<Candle>},
    Order(SystemTime, OrderState),
    Portfolio { positions: Vec<(String, Position)>, orders: Vec<OrderState> },
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