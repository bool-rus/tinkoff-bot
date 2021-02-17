use std::{collections::HashMap, time::SystemTime};

pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
use async_channel::{Receiver, Sender};
use chrono::TimeZone;
use serde::Serialize;

pub use crate::streaming::entities::Interval;

#[derive(Default, Clone)]
pub struct Market {
    stocks: HashMap<String, Stock>,
    state: HashMap<String, StockState>,
}

impl Market {
    pub fn update_stocks(&mut self, stocks: Vec<Stock>) {
        stocks.into_iter().for_each(|s| {
            self.stocks.insert(s.figi.to_owned(), s);
        });
    }
    pub fn update_orders(&mut self, orders: Vec<OrderState>) {
        for state in self.state.values_mut() {
            state.inwork_orders = HashMap::new();
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
            self.state.get_mut(&figi).and_then(|state| {
                state.inwork_orders = orders;
                Some(())
            });
        });
    }
    pub fn update_positons(&mut self, positions: Vec<(String, Position)>) {
        for (figi, position) in positions {
            self.state_mut(&figi).position = position;
        }
    }
    pub fn portfolio(&self) -> Vec<(Stock, Position)> {
        log::info!("all stocks: {}", self.state.len());
        self.state.iter().filter_map(|(figi, state)| {
            let position = state.position;
            if position.balance != 0.0 {
                Some((self.stock(figi).clone(), position))
            } else {
                None
            }
        }).collect()
    }
    pub fn stock(&self, figi: &str) -> Stock {
        self.stocks.get(figi).map(Clone::clone).unwrap_or(Stock {
            name: figi.to_owned(),
            figi: figi.to_owned(),
            ticker: figi.to_owned(),
            isin: None,
            min_increment: 0.01,
            lot: 1,
        })
    }
    pub fn state_mut(&mut self, figi: &str) -> &mut StockState {
        self.state.entry(figi.to_owned()).or_insert(Default::default())
    }
    pub fn state(&self, figi: &str) -> Option<&StockState> {
        self.state.get(figi)
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

#[derive(Debug, Clone, Default)]
pub struct StockState {
    pub position: Position,
    pub orderbook: Orderbook,
    pub candles: Vec<Candle>,
    pub inwork_orders: HashMap<String, OrderState>,
    pub new_orders: HashMap<SystemTime, Order>,
}

#[derive(Debug, Clone)]
pub struct Stock {
    pub name: String,
    pub figi: String,
    pub ticker: String,
    pub isin: Option<String>,
    pub min_increment: f64,
    pub lot: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub lots: i32,
    pub balance: f64,
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
    pub high: f64,
    pub volume: i32,
    pub time: DateTime,
}


pub struct ServiceHandle<Req, Res> {
    sender: Sender<Req>,
    receiver: Receiver<Res>,
}

impl <Req, Res> ServiceHandle<Req, Res> {
    pub fn new(sender: Sender<Req>, receiver: Receiver<Res>) -> Self {
        Self {sender, receiver}
    }
    pub async fn send(&self, msg: Req) -> Result<(), ChannelStopped> {
        self.sender.send(msg).await.map_err(|_|ChannelStopped)
    }
    pub async fn recv(&self) -> Result<Res, ChannelStopped> {
        self.receiver.recv().await.map_err(|_|ChannelStopped)
    }
    pub fn receiver(&self) -> Receiver<Res> {
        self.receiver.clone()
    }
    pub fn stop(&mut self) {
        self.sender.close();
        self.receiver.close();
    }
}

pub struct ChannelStopped;
