use std::collections::HashMap;

use tinkoff_api::models::MarketInstrument;

pub mod static_amount;

#[derive(Debug)]
pub enum Decision {
    Relax,
    Order(Order)
}

pub trait Strategy {
    fn make_decision(&mut self) -> Decision;
}

#[derive(Default)]
pub struct Market {
    pub positions: HashMap<String, u32>,
    pub orders: Vec<Order>,
    pub stocks: HashMap<String, Stock>,
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

impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
        }
    }
}

pub struct Orderbook {
    pub bids: Vec<(f64, u32)>,
    pub asks: Vec<(f64, u32)>,
}
