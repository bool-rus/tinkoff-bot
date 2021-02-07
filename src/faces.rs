use std::collections::HashMap;

pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
pub use crate::streaming::entities::Interval;

#[derive(Default)]
pub struct Market {
    pub positions: HashMap<String, u32>,
    pub orders: Vec<Order>,
    pub stocks: HashMap<String, Stock>,
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
    pub orderbook: Orderbook,
    pub candles: Vec<Candle>,
}

#[derive(Default)]
pub struct Orderbook {
    pub bids: Vec<(f64, u32)>,
    pub asks: Vec<(f64, u32)>,
}

pub struct Candle {
    pub start: f64,
    pub end: f64,
    pub low: f64,
    pub hight: f64,
    pub volume: i32,
    pub time: DateTime,
}
