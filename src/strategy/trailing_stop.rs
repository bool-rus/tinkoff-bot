
use serde::{Serialize, Deserialize};

use crate::model::{Order, OrderKind};
use super::{ConfigError, Decision, Strategy};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrailingStop {
    figi: String, 
    stop_treshold: f64,
    best_price: f64,
    quantity: usize,
    finished: bool,
}

impl TrailingStop {
    fn make_order(&self, price: f64) -> Order {
        Order {
            figi: self.figi.clone(),
            kind: OrderKind::Sell,
            price,
            quantity: self.quantity as u32,
        }
    }
}

impl Default for TrailingStop {
    fn default() -> Self {
        Self {
            figi: String::new(),
            stop_treshold: 0.05,
            best_price: 0.0,
            quantity: 0,
            finished: false,
        }
    }
}

impl Strategy for TrailingStop {
    fn name(&self) -> &'static str {
        "Скользящий стоп"
    }

    fn description(&self) -> &'static str {
        "Аналогично обычному стоп-лосс, но двигается при изменении цены в лучшую сторону"
    }

    fn params(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("figi", "FIGI инструмента"),
            ("stop_treshold", "(0.05 - 5%) при каком относительном падении продавать"),
            ("quantity", "сколько продать при достижени порога"),
        ]
    }

    fn configure(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        match key {
            "figi" => self.figi = value,
            "stop_treshold" => self.stop_treshold = value.parse()?,
            "quantity" => self.quantity = value.parse()?,
            _ => return Err(ConfigError::INVALID_PARAM)
        }
        Ok(())
    }

    fn make_decision(&mut self, market: &crate::model::Market) -> Vec<Decision> {
        if self.finished { return Vec::new() }
        if let Some(state) = market.state(&self.figi) {
            match state.orderbook.bids.get(0).map(|(p, _)|*p).unwrap_or(self.best_price) {
                price if price > self.best_price => self.best_price = price,
                price if (self.best_price - price) / self.best_price > self.stop_treshold => {
                    self.finished = true;
                    return vec![Decision::Order(self.make_order(price))]
                }
                _ => {}
            }
        }
        Vec::new()
    }

    fn balance(&self) -> f64 {
        0.0
    }
}