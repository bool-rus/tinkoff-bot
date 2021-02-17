use crate::model::{Position, Stock};

#[derive(Clone, Copy)]
pub enum Request {
    Portfolio,
}

#[derive(Clone)]
pub enum Response {
    Portfolio(Vec<(Stock, Position)>),
}
