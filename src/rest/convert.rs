use tinkoff_api::models::*;
use crate::model::Stock;
use super::entities::Response;

impl From<MarketInstrumentListResponse> for Response {
    fn from(r: MarketInstrumentListResponse) -> Self {
        Response::Stocks(r.payload.instruments.iter().map(Into::into).collect())
    }
}

impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
            position: 0,
            orderbook: Default::default(),
            candles: Vec::new(),
            inwork_orders: Default::default(),
            new_orders: Default::default(),
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
            (p.figi, p.lots as u32)
        }).collect())
    }
}