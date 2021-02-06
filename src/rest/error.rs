use tinkoff_api::{apis::market_api::MarketStocksGetError, models::Error as ServerError};

#[derive(Debug)]
pub enum Error {
    Server(ServerError),
    UnknownValue(serde_json::Value)
}

impl From<MarketStocksGetError> for Error {
    fn from(e: MarketStocksGetError) -> Self {
        match e {
            MarketStocksGetError::Status500(e) => Error::Server(e),
            MarketStocksGetError::UnknownValue(v) => Error::UnknownValue(v)
        }
    }
}
