mod faces;
mod convert;
mod streaming;
mod rest;
mod strategy;

use std::{cell::RefCell, rc::Rc};

use faces::Market;
use strategy::{Decision, Strategy, static_amount::StaticAmount};
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
    let stream_uri = "wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws".to_owned();
    let rest_uri = "https://api-invest.tinkoff.ru/openapi/sandbox".to_owned();
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

    let (to_streaming, receiver) = async_channel::bounded(100);
    let (sender, from_streaming) = async_channel::bounded(100);
    streaming::start_client(token.clone(), stream_uri, receiver, sender);

    let (to_rest, receiver) = async_channel::bounded(100);
    let (sender, from_rest) = async_channel::bounded(100);
    rest::start_client(token, rest_uri, receiver, sender);

    to_streaming.send(entities::Request::OrderbookSubscribe{
        figi: "BBG000BH2JM1".to_owned(),
        depth: 4,
    }).await.unwrap();
  
    market.positions.insert("BBG000BH2JM1".to_owned(), 0);
    let market = Rc::new(RefCell::new(market));
    let mut strategy = StaticAmount::new(market.clone(), "BBG000BH2JM1".to_owned()).target(100000.0);
    let mut balance = 200000.0;

    loop {
        tokio::select! {
            Ok(msg) = from_streaming.recv() => {
                update_market_from_streaming(market.clone(), msg);
                let decision = strategy.make_decision();
                match decision {
                    Decision::Relax => {}
                    Decision::Order(faces::Order{kind, price, quantity, figi}) => {
                        let mut market = market.borrow_mut();
                        let have = market.positions.get_mut(&figi).unwrap();
                        match kind {
                            faces::OrderKind::Buy => {
                                balance -= price * (quantity as f64);
                                *have += quantity;
                                println!("BUY {}", quantity)
                            }
                            faces::OrderKind::Sell => {
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
            Ok(msg) = from_rest.recv() => {

            }
        }
    }
}

fn update_market_from_streaming(market: Rc<RefCell<Market>>, msg: entities::Response) {
    match msg.kind {
        entities::ResponseType::Candle(_) => {}
        entities::ResponseType::Orderbook { figi, depth, bids, asks } => {
            let mut market = market.borrow_mut();
            market.orderbooks.insert(figi, faces::Orderbook {bids,asks});
        }
        entities::ResponseType::Info { figi, trade_status, min_price_increment, lot } => {}
        entities::ResponseType::Error { request_id, error } => {}
    }
}