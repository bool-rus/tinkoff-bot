use std::str::FromStr;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Interval {
    #[serde(rename="1min")]
    MIN1, 
    #[serde(rename="2min")]
    MIN2, 
    #[serde(rename="3min")]
    MIN3,
    #[serde(rename="5min")]
    MIN5,
    #[serde(rename="10min")]
    MIN10,
    #[serde(rename="15min")]
    MIN15,
    #[serde(rename="30min")]
    MIN30,
    #[serde(rename="hour")]
    HOUR,
    #[serde(rename="2hour")]
    HOUR2,
    #[serde(rename="4hour")]
    HOUR4,
    #[serde(rename="day")]
    DAY,
    #[serde(rename="week")]
    WEEK,
    #[serde(rename="month")]
    MOUNTH
}
#[derive(Serialize, Clone, Hash, Eq, PartialEq, Debug)]
#[serde(tag = "event")]
pub enum Request {
    #[serde(rename="candle:subscribe")]
    CandleSubscribe {figi: String, interval: Interval},
    #[serde(rename="candle:unsubscribe")]
    CandleUnsubscribe {figi: String, interval: Interval},
    #[serde(rename="orderbook:subscribe")]
    OrderbookSubscribe {figi: String, depth: u32},
    #[serde(rename="orderbook:unsubscribe")]
    OrderbookUnsubscribe {figi: String, depth: u32},
    #[serde(rename="instrument_info:subscribe")]
    InfoSubscribe {figi: String},
    #[serde(rename="instrument_info:unsubscribe")]
    InfoUnsubsribe {figi: String},
}

impl ToString for Request {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
#[derive(Deserialize, Debug)]
pub struct Candle {
    o: f64, c: f64, h: f64, l: f64, v: i32, 
    #[serde(with = "rfc3339")]
    time: DateTime<FixedOffset>, 
    interval: Interval, figi: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TradeStatus {
    BreakInTrading,
    NormalTrading,
    NotAvailableForTrading,
    ClosingAuction,
    ClosingPeriod,
    DiscreteAuction,
    OpeningPeriod,
    TradingAtClosingAuctionPrice,
}
#[derive(Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Candle (Candle),
    Orderbook {
        figi: String,
        depth: u32,
        #[serde(with="u32as_floating_point")]
        bids: Vec<(f64, u32)>,
        #[serde(with="u32as_floating_point")]
        asks: Vec<(f64, u32)>,
    },
    #[serde(rename = "instrument_info")]
    Info {
        figi: String,
        trade_status:  TradeStatus,
        min_price_increment: f64,
        lot: u32,
    },
    Error {
        request_id: Option<String>,
        error: String,
    }
}
#[derive(Deserialize, Debug)]
pub struct Response {
    #[serde(with = "rfc3339")]
    pub time: DateTime<FixedOffset>,
    #[serde(flatten)]
    pub kind: ResponseType,
}

impl FromStr for Response {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

mod u32as_floating_point {
    use serde::{self, Deserialize, Deserializer};
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<(f64, u32)>, D::Error> where D: Deserializer<'de> {
        let fp = Vec::<(f64, f64)>::deserialize(deserializer)?;
        Ok(fp.into_iter().map(|(f,u)|(f, u as u32)).collect())
    }
}

mod rfc3339 {
    use chrono::{DateTime, FixedOffset};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
    }

}

#[test]
pub fn test_response() {
    let data = r#"{
        "event": "candle",
        "time": "2019-08-07T15:35:00.029721253Z",
        "payload": {
            "o": 64.0575,
            "c": 64.0575,
            "h": 64.0575,
            "l": 64.0575,
            "v": 156,
            "time": "2019-08-07T15:35:00Z",
            "interval": "5min",
            "figi": "BBG0013HGFT4"
        }
    }"#;
    let v: Response = serde_json::from_str(data).unwrap();
    println!("response candle: {:?}", v);

    let data = r#"{
        "event": "orderbook",
        "time": "2019-08-07T15:35:00.029721253Z",
        "payload": {
            "figi": "BBG0013HGFT4",
            "depth": 2,
            "bids": [
                [64.3525, 204],
                [64.1975, 276]
            ],
            "asks": [
                [64.38, 227],
                [64.5225, 120]
            ]
        }
    }"#;

    let v: Response = serde_json::from_str(data).unwrap();
    println!("response orderbook: {:?}", v);

    let data = r#"{
        "event": "instrument_info",
        "time": "2019-08-07T15:35:00.029721253Z",
        "payload": {
            "figi": "BBG0013HGFT4",
            "trade_status": "normal_trading",
            "min_price_increment": 0.0025,
            "lot": 1000
        }
    }"#;

    let v: Response = serde_json::from_str(data).unwrap();
    println!("response info: {:?}", v);

    let data = r#"{
        "event": "error",
        "time": "2019-08-07T15:35:00.029721253Z",
        "payload": {
            "request_id": "123ASD1123",
            "error": "Subscription instrument_info:subscribe. FIGI NOOOOOOO not found"
        }
    }"#;

    let v: Response = serde_json::from_str(data).unwrap();
    println!("response error: {:?}", v);

}