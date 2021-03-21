use std::collections::HashMap;

use crate::model::{Position, Stock};

pub type Key = String;

#[derive(Debug, Clone)]
pub enum Request<S> {
    Portfolio,
    AddStrategy(Key, S),
    RemoveStrategy(Key),
    Strategies,
}

#[derive(Debug, Clone)]
pub enum Response<S> {
    Portfolio(Vec<(Stock, Position)>),
    Stocks(Vec<Stock>),
    Strategies(HashMap<Key, S>),
}
