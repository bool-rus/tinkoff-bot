use std::time::SystemTime;

use crate::model::{Position, Stock};
use crate::strategy::StrategyKind;
pub type Key = String;

#[derive(Clone)]
pub enum Request<S> {
    Portfolio,
    AddStrategy(Key, S),
    RemoveStrategy(Key)
}

#[derive(Clone)]
pub enum Response {
    Portfolio(Vec<(Stock, Position)>),
    Stocks(Vec<Stock>),
}
