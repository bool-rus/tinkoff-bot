use std::cell::RefCell;
use super::*;
use OrderKind::*;

pub struct StaticAmount {
    market: RefCell<Market>,
    figi: String,
    target: f64,
}

fn make_decision(figi: String, bid_price: f64, ask_price: f64, position: u32, target: f64) -> Decision {
    let over = ((position as f64) * bid_price - target)/bid_price;
    if over > 1.0 {
        return Decision::Order(Order {
            kind: Sell,
            figi, 
            price: bid_price, 
            quantity: over.round() as u32,
        });
    }
    let under = (target - (position as f64) * ask_price)/ask_price;
    if under > 1.0 {
        return Decision::Order(Order {
            kind: Buy,
            figi, 
            price: ask_price, 
            quantity: under.round() as u32,
        });
    }
    Decision::Relax
}

fn have_order(figi: &str, orders: &Vec<Order>) -> bool {
    orders.iter().find(|&order|order.figi.eq(figi)).is_some()
}

impl Strategy for StaticAmount {
    fn make_decision(&mut self) -> Decision {
        let market = self.market.borrow();
        if have_order(&self.figi, &market.orders) {
            return Decision::Relax;
        }
        let vol = *market.positions.get(&self.figi).unwrap();
        if let Some(orderbook) = market.stocks.get(&self.figi).map(|s|&s.orderbook) {
            if let (Some(&bid), Some(&ask)) = (orderbook.bids.get(0), orderbook.asks.get(0)) {
                return make_decision(self.figi.clone(), bid.0, ask.0, vol, self.target)
            }
        }
        Decision::Relax
    }
}

