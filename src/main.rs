mod streaming;

use std::collections::HashMap;

use futures_util::SinkExt;
use streaming::entities;
use tokio::io::{AsyncReadExt, stdin};

use tinkoff_api::apis::configuration::Configuration;
use tinkoff_api::apis::market_api;
use tokio_compat_02::FutureExt;

#[tokio::main]
async fn main() {
    let conf = Configuration {
        base_path: "https://api-invest.tinkoff.ru/openapi/sandbox".to_owned(),
        bearer_access_token: Some("t.xwGtvjeVXUHM0JwVh9IYDGB5JsISXS51m63-PKNfQT4zz2Xkl4KHW-OvpoYgBHYuN9JfV5DcNB2WJjfpoKv5Kg".to_owned()),
        ..Default::default()
    };
    let stocks = market_api::market_stocks_get(&conf).compat().await.unwrap();

    let instruments = stocks.payload.instruments;
    
    let stocks = instruments.iter().fold(
        HashMap::with_capacity(instruments.len()/4), 
        |mut map, i| {
            map.insert(i.figi.clone(), i);
            map
        }
    );

    let (sender, receiver) = async_channel::bounded(100);
    let mut sink = streaming::start_client(sender).await;

    let requests: Vec<_> = stocks.keys().map(|k|{
        entities::Request::OrderbookSubscribe {
            figi: k.clone(),
            depth: 4,
        }.to_string()
    }).collect();
    tokio::spawn(async move {
        for r in requests {
            sink.send(tungstenite::Message::Text(r)).await;
        }
    });
    while let Ok(msg) = receiver.recv().await {
        match msg.kind {
            entities::ResponseType::Candle(_) => {}
            entities::ResponseType::Orderbook { figi, depth, bids, asks } => {
                if asks.len() > 0 && bids.len() > 0 {
                    let spread = (asks[0].0 - bids[0].0)/asks[0].0;
                    if spread > 0.05 {
                        println!("ticker {}, good spread: {:.0}%", stocks.get(&figi).unwrap().ticker, spread*100.0)
                    }
                }
            }
            entities::ResponseType::Info { figi, trade_status, min_price_increment, lot } => {}
            entities::ResponseType::Error { request_id, error } => {}
        }
    }
}
