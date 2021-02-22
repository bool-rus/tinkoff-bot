use std::time::SystemTime;

use crate::model::{Position, Stock};
use crate::strategy::StrategyKind;
pub type Key = SystemTime;

pub enum Request {
    Portfolio,
    AddStrategy(Key, StrategyKind),
    RemoveStrategy(Key)
}

#[derive(Clone)]
pub enum Response {
    Portfolio(Vec<(Stock, Position)>),
    Stocks(Vec<Stock>),
}
