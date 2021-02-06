use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::faces::*;

pub struct FixedAmount {
    figi: String,
    target: f64,
    buy_threshold: f64,
    sell_treshold: f64,
}

impl FixedAmount {
    pub fn new(figi: String) -> Self {
        Self {
            figi,
            target: 1000.0,
            buy_threshold: 0.02,
            sell_treshold: 0.02,
        }
    }
    pub fn target(self, target: f64) -> Self {
        Self {target, ..self}
    }
    pub fn thresholds(self, buy_threshold: f64, sell_treshold: f64) -> Self {
        Self {buy_threshold, sell_treshold, ..self}
    }

    fn _make_decision(&self, figi: String, bid_price: f64, ask_price: f64, position: u32) -> Decision {
        use OrderKind::*;
        let target = self.target;
        let over = (position as f64) * bid_price - target;
        if over/target > 0.001 { //TODO: использовать threshold
            let quantity = (over/bid_price).round() as u32;
            return Decision::Order(Order {
                kind: Sell,
                figi, 
                price: bid_price, 
                quantity,
            });
        }
        let under = target - (position as f64) * ask_price;
        if under/target > 0.001 {
            let quantity = (under/bid_price).round() as u32;
            return Decision::Order(Order {
                kind: Buy,
                figi, 
                price: ask_price, 
                quantity,
            });
        }
        Decision::Relax
    }
}
fn have_order(figi: &str, orders: &Vec<Order>) -> bool {
    orders.iter().find(|&order|order.figi.eq(figi)).is_some()
}

impl Strategy for FixedAmount {
    fn make_decision(&mut self, market: &Market) -> Decision {
        if have_order(&self.figi, &market.orders) {
            return Decision::Relax;
        }
        let vol = *market.positions.get(&self.figi).unwrap();
        if let Some(orderbook) = market.orderbooks.get(&self.figi) {
            if let (Some(&bid), Some(&ask)) = (orderbook.bids.get(0), orderbook.asks.get(0)) {
                return self._make_decision(self.figi.clone(), bid.0, ask.0, vol)
            }
        }
        Decision::Relax
    }
}

