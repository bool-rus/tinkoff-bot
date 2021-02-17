use tinkoff_api::models::*;
use crate::model::{OrderState, Stock};

impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            name: i.name.to_owned(),
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
            min_increment: i.min_price_increment.unwrap_or(0.01),
            lot: i.lot as u32,
        }
    }
}

impl From<Order> for OrderState {
    fn from(o: Order) -> Self {
        let Order { order_id, figi, operation, price, requested_lots, executed_lots, ..} = o;
        let order = crate::model::Order {
            figi,
            kind: operation.into(),
            price,
            quantity: requested_lots as u32,
        };
        crate::model::OrderState {
            order_id,
            order,
            executed: executed_lots as u32,
        }
    }
}
