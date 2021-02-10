use std::{cmp::min, fmt::DebugSet};

use crate::{model::{DateTime, Interval, Market, Order, Orderbook}, rest};
pub mod fixed_amount;
use chrono::{Date, Duration, FixedOffset, Local};
pub use fixed_amount::FixedAmount;
use crate::streaming::entities::Request as StreamingRequest;
use crate::rest::entities::Request as RestRequest;

#[derive(Debug)]
pub enum Decision {
    Relax,
    Order(Order),
    CallStreaming(StreamingRequest),
    CallRest(RestRequest),
}

pub trait Strategy {
    fn make_decision(&mut self, market: &Market) -> Decision;
    fn balance(&self) -> f64;
}
/*
pub struct StrategyProfiler<T: Strategy> {
    strategy: T,
    figi: String,
    date_start: Date<FixedOffset>,
    date_end: Date<FixedOffset>,
}

impl <T: Strategy> StrategyProfiler<T> {
    pub fn new(strategy: T, figi: String) -> Self {
        let today = chrono::Utc::now().with_timezone(&FixedOffset::east(3*3600)).date();
        Self {
            strategy, 
            figi,
            date_start: today - Duration::days(200),
            date_end: today-Duration::days(1),
        }
    }
}

impl <T: Strategy> Strategy for StrategyProfiler<T> {
    fn make_decision(&mut self, market: &Market) -> Decision {
        self.date_start = self.date_start + Duration::days(1);
        if self.date_start < self.date_end  {
            println!("retrieve balance for {:?}", self.date_start);
            return Decision::CallRest(rest::entities::Request::GetCandles { 
                figi: self.figi.clone(), 
                from: self.date_start.and_hms(0, 0, 0), 
                to: self.date_start.and_hms(23, 59, 59), 
                interval: Interval::MIN1,
            })
        } else {
            let mut fake = market.clone();
            let offset = FixedOffset::east(0);
            let mut counter = 0;
            println!("candles: {}", market.stocks.get(&self.figi).unwrap().candles.len());
            for candle in &market.stocks.get(&self.figi).unwrap().candles {
                let bid = f64::min(candle.open, candle.close);
                let ask = f64::max(candle.open, candle.close);
                fake.stocks.get_mut(&self.figi).unwrap().orderbook = Orderbook {
                    time: Local::now().with_timezone(&offset),
                    bids: vec![(bid, 100)],
                    asks: vec![(ask, 100)],
                };
                if let Decision::Order(order) = self.strategy.make_decision(&fake) {
                    let Order {kind, quantity, ..} = order;
                    if quantity == 0 { continue; }
                    counter += 1;
                    if counter % 1 == 0 {
                        println!("{:?} {}, balance: {}", kind, quantity, self.strategy.balance());
                    }
                    let papers = fake.positions.get_mut(&self.figi).unwrap();
                    match kind {
                        crate::model::OrderKind::Buy => *papers += quantity,
                        crate::model::OrderKind::Sell => *papers -= quantity,
                    }
                }
                
            }
            println!("result balance: {}, papers: {}", self.strategy.balance(), fake.positions.get(&self.figi).unwrap());
            Decision::Relax
        }
    }

    fn balance(&self) -> f64 {
        self.strategy.balance()
    }
}
*/