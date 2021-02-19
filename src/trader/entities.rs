use std::time::SystemTime;

use crate::{model::{Position, Stock}, strategy::{ConfigurableStrategy, Strategy}};
pub type Key = SystemTime;

pub enum Request {
    Portfolio,
    AddStrategy(Key, Box<dyn ConfigurableStrategy>),
    RemoveStrategy(Key)
}

#[derive(Clone)]
pub enum Response {
    Portfolio(Vec<(Stock, Position)>),
    Stocks(Vec<Stock>),
}
