use crate::model::{Position, Stock};

pub type Key = String;

#[derive(Clone)]
pub enum Request<S> {
    Portfolio,
    AddStrategy(Key, S),
    RemoveStrategy(Key),
    Strategies,
}

#[derive(Clone)]
pub enum Response<S> {
    Portfolio(Vec<(Stock, Position)>),
    Stocks(Vec<Stock>),
    Strategies(Vec<S>),
}
