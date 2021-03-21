use super::*;
use serde::{Serialize, Deserialize};
use crate::model::StockState;
use crate::model::OrderKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedAmount {
    figi: String,
    target: f64,
    balance: f64,
    buy_threshold: f64,
    sell_threshold: f64,
    corrected_buy: f64,
    corrected_sell: f64,
    factor: f64,
    first_buy: bool,
}

impl Default for FixedAmount {
    fn default() -> Self {
        Self::new("".to_owned())
    }
}

impl FixedAmount {
    pub fn new(figi: String) -> Self {
        Self {
            figi,
            target: 10000.0,
            balance: 0.0,
            buy_threshold: 0.01,
            sell_threshold: 0.01,
            corrected_buy: 0.01,
            corrected_sell: 0.01,
            factor: 1.0,
            first_buy: true,
        }
    }

    fn _make_decision(&mut self, figi: String, bid_price: f64, ask_price: f64, balance: f64) -> Vec<Decision> {
        let target = self.target;
        let factor = self.factor;

        let over = balance * bid_price - target;
        if over/target > self.corrected_sell { //TODO: использовать threshold
            let quantity = (over/bid_price)as u32;
            if quantity == 0 {
                return Vec::new();
            }
            log::info!("over: {:.2}, sell", over);
            self.balance += (quantity as f64) * bid_price;
            self.corrected_buy /= factor;
            if self.corrected_buy < self.buy_threshold {
                self.corrected_buy = self.buy_threshold;
            }
            self.corrected_sell *= factor;
            return vec![Decision::Order(Order {
                kind: OrderKind::Sell,
                figi, 
                price: bid_price, 
                quantity,
            })];
        }
        let under = target - balance * ask_price;
        if under/target > self.corrected_buy {
            let quantity = (under/bid_price) as u32;
            if quantity == 0 {
                return Vec::new();
            }
            log::info!("under: {:.2}, buy", under);
            self.balance -= (quantity as f64) * ask_price;
            self.corrected_sell /= factor;
            if self.corrected_sell < self.sell_threshold {
                self.corrected_sell = self.sell_threshold
            }
            self.corrected_buy *= factor;
            if self.first_buy {
                self.balance = 0.0;
                self.first_buy = false;
            }
            return vec![Decision::Order(Order {
                kind: OrderKind::Buy,
                figi, 
                price: ask_price, 
                quantity,
            })];
        }
        Vec::new()
    }
}

fn have_orders(stock: &StockState)  -> bool {
    !stock.new_orders.is_empty() || !stock.inwork_orders.is_empty()
}

impl Strategy for FixedAmount {
    fn make_decision(&mut self, market: &Market) -> Vec<Decision> {
        if let Some(stock) = market.state(&self.figi) {
            if have_orders(stock) {
                return Vec::new();
            }
            let vol =  stock.position.balance;
            let orderbook = &stock.orderbook;
            if let (Some(&bid), Some(&ask)) = (orderbook.bids.get(0), orderbook.asks.get(0)) {
                return self._make_decision(self.figi.clone(), bid.0, ask.0, vol)
            }
        }
        Vec::new()
    }
    fn balance(&self) -> f64 {
        self.balance/self.target * 100.0
    }

    fn name(&self) -> &'static str {
        "Фикс стоимость"
    }
    fn description(&self) -> &'static str {
        r#"Стратегия по сохранению фиксированной общей стоимости позиции. 
        Если общая стоимость превышает заданный порог отклонения от целевой - продаем
        Если меньше - покупаем"#
    }

    fn params(&self) -> Vec<(&'static str, &'static str)> {
        vec!{
            ("figi", "FIGI инструмента"),
            ("target", "На какую сумму должно быть куплено"),
            ("buy_threshold", "Порог снижения суммы для покупки"),
            ("sell_threshold", "Порог роста цены для продажи"),
            ("factor", ""),
        }
    }

    fn configure(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        match key {
            "figi" => self.figi = value,
            "target" => self.target = value.parse()?,
            "buy_threshold" => self.buy_threshold = value.parse()?,
            "sell_threshold" => self.sell_threshold = value.parse()?,
            "factor" => self.factor = value.parse()?,
            _ => return Err(ConfigError::INVALID_PARAM),
        }
        Ok(())
    }
}

