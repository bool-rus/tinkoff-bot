use tinkoff_api::models::MarketInstrument;
use tokio_tungstenite::tungstenite::Message;

use crate::{faces::Stock, streaming::entities::Request};


impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
        }
    }
}

impl Into<Message> for &Request {
    fn into(self) -> Message {
        Message::Text(self.to_string())
    }
}