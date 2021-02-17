use tinkoff_api::models::*;
use crate::model::{Position, StockState, Stock};
use super::entities::Response;

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

impl From<OrdersResponse> for Response {
    fn from(response: OrdersResponse) -> Self {
        Response::Orders(response.payload.into_iter().map(
            | Order { order_id, figi, operation, price, requested_lots, executed_lots, ..}| {
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
        }).collect())
    }
}

impl From<PortfolioResponse> for Response {
    fn from(res: PortfolioResponse) -> Self {
        Response::Positions(res.payload.positions.into_iter().map(|p|{
            (p.figi, Position {lots: p.lots, balance: p.balance})
        }).collect())
    }
}