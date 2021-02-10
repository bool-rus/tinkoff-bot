use std::{collections::HashMap, time::SystemTime};

pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
use chrono::{FixedOffset, Local, TimeZone};
use serde::Serialize;

pub use crate::streaming::entities::Interval;

#[derive(Default, Clone)]
pub struct Market {
    pub stocks: HashMap<String, Stock>,
}

impl Market {
    pub fn update_orders(&mut self, orders: Vec<OrderState>) {
        for stock in self.stocks.values_mut() {
            stock.inwork_orders = HashMap::new();
        }
        let orders: HashMap<String, HashMap<_, _>> = orders.into_iter().fold(HashMap::new(), |mut map, state|{
            let figi = state.order.figi.clone();
            match map.get_mut(&figi) {
                Some(v) => {v.insert(state.order_id.clone(), state);},
                None => {
                    let mut v = HashMap::new();
                    v.insert(state.order_id.clone(), state);
                    map.insert(figi, v);
                }
            }
            map
        });
        orders.into_iter().for_each(|(figi, orders)|{
            self.stocks.get_mut(&figi).and_then(|stock| {
                stock.inwork_orders = orders;
                Some(())
            });
        });
    }
    pub fn update_positons(&mut self, positions: Vec<(String, u32)>) {
        for (figi, position) in positions {
            if let Some(stock) = self.stocks.get_mut(&figi) {
                stock.position = position;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Order {
    pub figi: String,
    pub kind: OrderKind,
    pub price: f64, //надо бы сюда Decimal завезти
    pub quantity: u32, 
}

impl Eq for Order {}

impl std::hash::Hash for Order {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&serde_json::to_vec(self).unwrap());
    }
}

pub type OrderKind = tinkoff_api::models::OperationType;

#[derive(Debug, Clone)]
pub struct Stock {
    pub figi: String,
    pub ticker: String,
    pub isin: Option<String>,
    pub position: u32,
    pub orderbook: Orderbook,
    pub candles: Vec<Candle>,
    pub inwork_orders: HashMap<String, OrderState>,
    pub new_orders: HashMap<SystemTime, Order>,
}

#[derive(Debug, Clone)]
pub struct OrderState {
    pub order_id: String,
    pub order: Order,
    pub executed: u32,
}

#[derive(Debug, Clone)]
pub struct Orderbook {
    pub time: DateTime,
    pub bids: Vec<(f64, u32)>,
    pub asks: Vec<(f64, u32)>,
}

impl Default for Orderbook {
    fn default() -> Self {
        Self {
            time: chrono::FixedOffset::east(0).ymd(2000, 1, 1).and_hms(0,0,0),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub close: f64,
    pub low: f64,
    pub hight: f64,
    pub volume: i32,
    pub time: DateTime,
}
