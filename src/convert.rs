use tinkoff_api::models::{CandleResolution, MarketInstrument};
use tokio_tungstenite::tungstenite::Message;
use crate::faces::*;

use crate:: streaming::entities::Request;


impl From<&MarketInstrument> for Stock {
    fn from(i: &MarketInstrument) -> Self {
        Stock {
            figi: i.figi.to_owned(),
            ticker: i.ticker.to_owned(),
            isin: i.isin.to_owned(),
            orderbook: Default::default(),
            candles: Vec::new(),
        }
    }
}

impl Into<Message> for &Request {
    fn into(self) -> Message {
        Message::Text(self.to_string())
    }
}

impl Into<CandleResolution> for Interval {
    fn into(self) -> CandleResolution {
        match self {
            Interval::MIN1 => CandleResolution::_1min,
            Interval::MIN2 => CandleResolution::_2min,
            Interval::MIN3 => CandleResolution::_3min,
            Interval::MIN5 => CandleResolution::_5min,
            Interval::MIN10 => CandleResolution::_10min,
            Interval::MIN15 => CandleResolution::_15min,
            Interval::MIN30 => CandleResolution::_30min,
            Interval::HOUR => CandleResolution::Hour,
            Interval::HOUR2 => CandleResolution::Hour,
            Interval::HOUR4 => CandleResolution::Hour,
            Interval::DAY => CandleResolution::Day,
            Interval::WEEK => CandleResolution::Week,
            Interval::MOUNTH => CandleResolution::Month
        }
    }
}

impl From<tinkoff_api::models::Candle> for Candle {
    fn from(candle: tinkoff_api::models::Candle) -> Self {
        Self {
            start: candle.o,
            end: candle.c,
            low: candle.l,
            hight: candle.h,
            volume: candle.v,
            time: DateTime::parse_from_rfc3339(&candle.time).unwrap(),
        }
    }
}
