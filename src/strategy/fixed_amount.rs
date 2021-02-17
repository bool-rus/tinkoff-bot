use super::*;
use crate::{model::StockState, streaming};
use crate::model::OrderKind;

pub struct FixedAmount {
    figi: String,
    target: f64,
    balance: f64,
    buy_threshold: f64,
    sell_treshold: f64,
    corrected_buy: f64,
    corrected_sell: f64,
    factor: f64,
    first_buy: bool,
}

impl FixedAmount {
    pub fn new(figi: String) -> Self {
        Self {
            figi,
            target: 10000.0,
            balance: 0.0,
            buy_threshold: 0.01,
            sell_treshold: 0.01,
            corrected_buy: 0.01,
            corrected_sell: 0.01,
            factor: 1.0,
            first_buy: true,
        }
    }
    pub fn target(self, target: f64) -> Self {
        Self {target, ..self}
    }
    pub fn factor(self, factor: f64) -> Self {
        Self {factor, ..self}
    }
    pub fn thresholds(self, buy_threshold: f64, sell_treshold: f64) -> Self {
        Self {buy_threshold, sell_treshold, corrected_buy: buy_threshold, corrected_sell: sell_treshold, ..self}
    }

    fn _make_decision(&mut self, figi: String, bid_price: f64, ask_price: f64, balance: f64) -> Decision {
        let target = self.target;
        let factor = self.factor;

        let over = balance * bid_price - target;
        if over/target > self.corrected_sell { //TODO: использовать threshold
            let quantity = (over/bid_price)as u32;
            if quantity == 0 {
                return Decision::Relax;
            }
            log::info!("over: {:.2}, sell", over);
            self.balance += (quantity as f64) * bid_price;
            self.corrected_buy /= factor;
            if self.corrected_buy < self.buy_threshold {
                self.corrected_buy = self.buy_threshold;
            }
            self.corrected_sell *= factor;
            return Decision::Order(Order {
                kind: OrderKind::Sell,
                figi, 
                price: bid_price, 
                quantity,
            });
        }
        let under = target - balance * ask_price;
        if under/target > self.corrected_buy {
            let quantity = (under/bid_price) as u32;
            if quantity == 0 {
                return Decision::Relax;
            }
            log::info!("under: {:.2}, buy", under);
            self.balance -= (quantity as f64) * ask_price;
            self.corrected_sell /= factor;
            if self.corrected_sell < self.sell_treshold {
                self.corrected_sell = self.sell_treshold
            }
            self.corrected_buy *= factor;
            if self.first_buy {
                self.balance = 0.0;
                self.first_buy = false;
            }
            return Decision::Order(Order {
                kind: OrderKind::Buy,
                figi, 
                price: ask_price, 
                quantity,
            });
        }
        Decision::Relax
    }
}

fn have_orders(stock: &StockState)  -> bool {
    !stock.new_orders.is_empty() || !stock.inwork_orders.is_empty()
}

impl Strategy for FixedAmount {
    fn make_decision(&mut self, market: &Market) -> Decision {
        if let Some(stock) = market.state(&self.figi) {
            if have_orders(stock) {
                return Decision::Relax;
            }
            let vol =  stock.position.balance;
            let orderbook = &stock.orderbook;
            if let (Some(&bid), Some(&ask)) = (orderbook.bids.get(0), orderbook.asks.get(0)) {
                return self._make_decision(self.figi.clone(), bid.0, ask.0, vol)
            }
        }
        Decision::Relax
    }
    fn balance(&self) -> f64 {
        self.balance/self.target * 100.0
    }
}

