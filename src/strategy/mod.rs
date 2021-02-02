use std::collections::HashMap;

pub mod static_amount;

pub enum Decision {
    Relax,
    Order(Order)
}

pub trait Strategy {
    fn make_decision(&mut self) -> Decision;
}

pub struct Market {
    pub positions: HashMap<String, u32>,
    pub orders: Vec<Order>,
    pub stocks: HashMap<String, Stock>,
}

pub struct Order {
    figi: String,
    kind: OrderKind,
    price: f64,
    quantity: u32, 
}

pub enum OrderKind {
    Buy,
    Sell
}

pub struct Stock {
    pub figi: String,
    pub ticker: String,
    pub isin: String,
    pub orderbook: Orderbook,
}

pub struct Orderbook {
    pub bids: Vec<(f64, u32)>,
    pub asks: Vec<(f64, u32)>,
}
