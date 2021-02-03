mod streaming;
mod strategy;

use std::{cell::{Cell, RefCell}, collections::HashMap, rc::Rc};

use entities::Request;
use strategy::{Market, Orderbook, Strategy, static_amount::StaticAmount, Decision, Order};
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
async fn main() { //"BBG000BH2JM1" - NLOK
    let token = retrieve_token();
    let conf = Configuration {
        base_path: "https://api-invest.tinkoff.ru/openapi/sandbox".to_owned(),
        bearer_access_token: Some(token.clone()),
        ..Default::default()
    };
    let stocks = market_api::market_stocks_get(&conf).compat().await.unwrap();

    let instruments = stocks.payload.instruments;
    
    let mut market = instruments.iter().fold(
        Market::default(), 
        |mut m, i| {
            let stock = i.into();
            m.stocks.insert(i.figi.to_owned(), stock);
            m
        }
    );

    let (to_service, from_client) = async_channel::bounded(100);
    let (to_client, from_service) = async_channel::bounded(100);

    streaming::start_client(token, from_client, to_client);

    to_service.send(entities::Request::OrderbookSubscribe{
        figi: "BBG000BH2JM1".to_owned(),
        depth: 4,
    }).await.unwrap();
  
    market.positions.insert("BBG000BH2JM1".to_owned(), 0);
    let market = Rc::new(RefCell::new(market));
    let mut strategy = StaticAmount::new(market.clone(), "BBG000BH2JM1".to_owned()).target(100000.0);
    let mut balance = 200000.0;
    while let Ok(msg) = from_service.recv().await {
        match msg.kind {
            entities::ResponseType::Candle(_) => {}
            entities::ResponseType::Orderbook { figi, depth, bids, asks } => {
                let mut market = market.borrow_mut();
                market.orderbooks.insert(figi, Orderbook {bids,asks});
            }
            entities::ResponseType::Info { figi, trade_status, min_price_increment, lot } => {}
            entities::ResponseType::Error { request_id, error } => {}
        }
        let decision = strategy.make_decision();
        match decision {
            Decision::Relax => {}
            Decision::Order(Order{kind, price, quantity, figi}) => {
                let mut market = market.borrow_mut();
                let have = market.positions.get_mut(&figi).unwrap();
                match kind {
                    strategy::OrderKind::Buy => {
                        balance -= price * (quantity as f64);
                        *have += quantity;
                        println!("BUY {}", quantity)
                    }
                    strategy::OrderKind::Sell => {
                        balance += price * (quantity as f64);
                        *have -= quantity;
                        println!("SELL {}", quantity)
                    }
                }
                let expected_balance = balance + price * (*have as f64);
                println!("portfolio: {} in curr, {} expected full", balance, expected_balance);
            }
        }
    }
}
