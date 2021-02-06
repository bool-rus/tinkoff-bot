use std::collections::HashMap;


#[derive(Default)]
pub struct Market {
    pub positions: HashMap<String, u32>,
    pub orders: Vec<Order>,
    pub stocks: HashMap<String, Stock>,
    pub bonds: HashMap<String, Stock>,
    pub etfs: HashMap<String, Stock>,
    pub orderbooks: HashMap<String, Orderbook>,
}

#[derive(Debug)]
pub struct Order {
    pub figi: String,
    pub kind: OrderKind,
    pub price: f64,
    pub quantity: u32, 
}
#[derive(Debug)]
pub enum OrderKind {
    Buy,
    Sell
}

pub struct Stock {
    pub figi: String,
    pub ticker: String,
    pub isin: Option<String>,
}


pub struct Orderbook {
    pub bids: Vec<(f64, u32)>,
    pub asks: Vec<(f64, u32)>,
}
