use std::{collections::HashMap, time::SystemTime};

use tinkoff_api::apis::Error;
use crate::model::{Candle, DateTime, Interval, Order, OrderKind, OrderState, Stock};

#[derive(Clone, Debug)]
pub enum Request {
    GetStocks,
    GetETFs,
    GetBonds,
    GetCandles { figi: String, from: DateTime, to: DateTime, interval: Interval},
    LimitOrder(SystemTime, Order),
    GetOrders,
    GetPositions,
}

#[derive(Debug)]
pub enum Response {
    Err(Request, ErrX),
    Stocks(Vec<Stock>),
    Candles { figi: String, candles: Vec<Candle>},
    Order(SystemTime, OrderState),
    Orders(Vec<OrderState>),
    Positions(Vec<(String, u32)>),
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