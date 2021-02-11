use std::time::SystemTime;


use crate::model::*;
use crate::rest;
use crate::strategy::*;
use crate::streaming;
use crate::streaming::entities;

pub async fn start_bot() {
    //"BBG000BH2JM1" - NLOK
    let token = retrieve_token();
    let stream_uri = "wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws".to_owned();
    let rest_uri = "https://api-invest.tinkoff.ru/openapi/sandbox".to_owned();

    let mut market = Market::default();

    let (to_streaming, from_streaming) = streaming::Service::start(token.clone(), stream_uri);

    let (to_rest, receiver) = async_channel::bounded(100);
    let (sender, from_rest) = async_channel::bounded(100);
    rest::start_client(token, rest_uri, receiver, sender);


    let figi = "BBG000BH2JM1".to_owned();
    let target = 10000.0;
    let mut strategy = FixedAmount::new(figi.clone())
    .target(target)
    .thresholds(0.001, 0.001)
    .factor(1.0);

    let initial_requests;
    {
        use rest::entities::Request::*;
        initial_requests = vec![GetStocks, GetETFs, GetBonds, GetPositions];
    }

    for req in initial_requests {
        to_rest.send(req).await.unwrap();
        if let Ok(msg) = from_rest.recv().await {
            update_market_from_rest(&mut market, msg);
        }
    }

    let mut timer = tokio::time::interval(std::time::Duration::from_secs(7));
    loop {
        tokio::select! {
            Ok(msg) = from_streaming.recv() => {
                update_market_from_streaming(&mut market, msg);
            }
            Ok(msg) = from_rest.recv() => {
                update_market_from_rest(&mut market, msg);
            }
            _ = timer.tick() => {
                use crate::rest::entities::Request;
                to_rest.send(Request::GetPositions).await.unwrap();
                to_rest.send(Request::GetOrders).await.unwrap();
            }
        }
        let decision = strategy.make_decision(&market);
        match decision {
            Decision::Relax => {}
            Decision::Order(order) => {
                let stock = market.stocks.get_mut(&order.figi).unwrap();
                println!("order: {:?}, balance: {:.2}%", order, strategy.balance());
                let key = SystemTime::now();
                stock.new_orders.insert(key, order.clone());
                to_rest.send(crate::rest::entities::Request::LimitOrder(key, order)).await.unwrap();
            }
            Decision::CallRest(req) => to_rest.send(req).await.unwrap(),
            Decision::CallStreaming(req) => to_streaming.send(req).await.unwrap(),
        }
    }
}

fn retrieve_token() -> String {
    return "t.xwGtvjeVXUHM0JwVh9IYDGB5JsISXS51m63-PKNfQT4zz2Xkl4KHW-OvpoYgBHYuN9JfV5DcNB2WJjfpoKv5Kg".to_owned();
    use std::io::{stdin, BufRead};
    println!("insert token: ");
    stdin().lock().lines().into_iter().nth(0).unwrap().unwrap()
}

fn update_market_from_streaming(market: &mut Market, msg: entities::Response) {
    let entities::Response { time, kind } = msg;
    use entities::ResponseType;
    match kind {
        ResponseType::Candle(_) => {}
        ResponseType::Orderbook {figi,depth: _,bids,asks,} => {
            market.stocks.get_mut(&figi).and_then(|stock| {
                stock.orderbook = Orderbook { time, bids, asks };
                Some(())
            });
        }
        ResponseType::Info {..} => {}
        ResponseType::Error { .. } => {}
    }
}

fn update_market_from_rest(market: &mut Market, msg: rest::entities::Response) {
    use rest::entities::Response;
    match msg {
        Response::Err(request, e) => {
            if let crate::rest::entities::Request::LimitOrder(key, order, ..) = request {
                market.stocks.get_mut(&order.figi).unwrap().new_orders.remove(&key);
            }
            println!("ERR from rest!!! {:?}", e);
        }
        Response::Stocks(stocks) => {
            stocks.into_iter().for_each(|s| {
                market.stocks.insert(s.figi.to_owned(), s);
            });
        }
        Response::Candles { figi, candles } => {
            if let Some(stock) = market.stocks.get_mut(&figi) {
                stock.candles.extend(candles.into_iter());
            } else  {
                println!("stocks for {} not found", figi);
            }
        }
        rest::entities::Response::Order(key, state) => {
            let stock = market.stocks.get_mut(&state.order.figi).unwrap();
            stock.new_orders.remove(&key);
            stock.inwork_orders.insert(state.order_id.clone(),state);
        }
        rest::entities::Response::Orders(orders) => market.update_orders(orders),
        rest::entities::Response::Positions(positions) => market.update_positons(positions),
    }
}
