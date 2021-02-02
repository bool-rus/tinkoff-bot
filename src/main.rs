mod streaming;
mod strategy;

use std::collections::HashMap;

use streaming::entities;

use tinkoff_api::apis::configuration::Configuration;
use tinkoff_api::apis::market_api;
use tokio_compat_02::FutureExt;

fn retrieve_token() -> String {
    use std::io::{BufRead, stdin};
    println!("insert token: ");
    stdin().lock().lines().into_iter().nth(0).unwrap().unwrap()
}
#[tokio::main]
async fn main() {
    let token = retrieve_token();
    let conf = Configuration {
        base_path: "https://api-invest.tinkoff.ru/openapi/sandbox".to_owned(),
        bearer_access_token: Some(token.clone()),
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

    let (to_service, from_client) = async_channel::bounded(100);
    let (to_client, from_service) = async_channel::bounded(100);

    
    streaming::start_client(token, from_client, to_client);

    let requests: Vec<_> = stocks.keys().map(|k|{
        entities::Request::OrderbookSubscribe {
            figi: k.clone(),
            depth: 4,
        }
    }).collect();
    tokio::spawn(async move {
        for r in requests {
            to_service.send(r).await;
        }
    });
    while let Ok(msg) = from_service.recv().await {
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
