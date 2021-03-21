pub mod entities;

use std::{collections::HashMap, time::SystemTime};
use async_channel::{Receiver, Sender};
use entities::*;
use crate::rest::*;
use crate::streaming::*;
use crate::model::*;
use crate::strategy::{Strategy, Decision};

pub struct TraderConf {
    pub rest_uri: String,
    pub streaming_uri: String,
    pub token: String,
}

pub struct Trader<S> {
    sender: Sender<Response<S>>,
    receiver: Receiver<Request<S>>,
    streaming: ServiceHandle<StreamingRequest, StreamingResponse>,
    rest: ServiceHandle<RestRequest, RestResponse>,
    market: Market,
    strategies: HashMap<Key, S>,
}

impl<S: Strategy + Send + Clone + 'static> Trader<S> {
    pub fn start(conf: TraderConf) -> ServiceHandle<Request<S>, Response<S>> {
        let (sender, r) = async_channel::bounded(1000);
        let (s, receiver) = async_channel::bounded(1000);
        let TraderConf{rest_uri, streaming_uri, token} = conf;
        let trader = Self {
            sender, 
            receiver, 
            streaming: Streaming::start(token.clone(), streaming_uri), 
            rest: Rest::start(token, rest_uri), 
            market: Default::default(),
            strategies: Default::default(),
        };
        tokio::spawn(async move {
            match trader.run().await {
                Ok(()) => log::error!("Trade-bot finished... WTF?"),
                Err(_) => log::info!("Trade-bot stopped because channel in closed"),
            };
        });

        ServiceHandle::new(s, r)
    }

    async fn run(mut self) -> Result<(), ChannelStopped> {
        log::info!("Trader started");
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(7));
        self.rest.send(RestRequest::Instruments).await?;
        loop {
            tokio::select! {
                msg = self.streaming.recv() => {
                    self.update_market_from_streaming(msg?);
                }
                msg = self.rest.recv() => {
                    self.update_market_from_rest(msg?).await?;
                }
                msg = self.receiver.recv() => {
                    let msg = msg.map_err(|_|ChannelStopped)?;
                    self.process_request(msg).await?;
                }
                _ = timer.tick() => {
                    use crate::rest::entities::Request;
                    self.rest.send(Request::Portfolio).await?;
                }
            }
            let market = &self.market;
            let decisions: Vec<_> = self.strategies.values_mut().map(|s|s.make_decision(market).into_iter()).flatten().collect();
            for decision in decisions {
                self.process_decision(decision).await?;
            }
        }
    }

    async fn process_request(&mut self, request: entities::Request<S>) -> Result<(), ChannelStopped> {
        use entities::*;
        match request {
            Request::Portfolio => self.sender.send(Response::Portfolio(self.market.portfolio())).await?,
            Request::AddStrategy(k, s) => { 
                self.strategies.insert(k, s); 
                let strategies = self.strategies.clone();
                self.sender.send(Response::Strategies(strategies)).await?;
            }
            Request::RemoveStrategy(k) => { self.strategies.remove(&k); }
            Request::Strategies => unimplemented!()
        };
        Ok(())
    }


    async fn process_decision(&mut self, decision: Decision) -> Result<(), ChannelStopped> {
        match decision {
            Decision::Order(order) => {
                let stock = self.market.state_mut(&order.figi);
                let key = SystemTime::now();
                stock.new_orders.insert(key, order.clone());
                self.rest.send(crate::rest::entities::Request::LimitOrder(key, order)).await?;
            }
        }
        Ok(())
    }
    
    fn update_market_from_streaming(&mut self, msg: StreamingResponse) {
        let StreamingResponse { time, kind } = msg;
        use crate::streaming::entities::ResponseType;
        match kind {
            ResponseType::Candle(_) => {}
            ResponseType::Orderbook {figi, depth: _, bids, asks,} => {
                self.market.state_mut(&figi).orderbook = Orderbook { time, bids, asks };
            }
            ResponseType::Info {..} => {}
            ResponseType::Error { .. } => {}
        }
    }

    async fn update_market_from_rest(&mut self, msg: RestResponse) -> Result<(), ChannelStopped> {
        match msg {
            RestResponse::Err(request, e) => {
                if let crate::rest::entities::Request::LimitOrder(key, order, ..) = request {
                    self.market.state_mut(&order.figi).new_orders.remove(&key);
                }
                log::error!("ERR from rest!!! {:?}", e);
            }
            RestResponse::Stocks(stocks) => {
                self.sender.send(Response::Stocks(stocks.clone())).await?;
                self.market.update_stocks(stocks);
            },
            RestResponse::Candles { figi, candles } => {
                self.market.state_mut(&figi).candles.extend(candles.into_iter());
            }
            RestResponse::Order(key, state) => {
                let stock = self.market.state_mut(&state.order.figi);
                stock.new_orders.remove(&key);
                stock.inwork_orders.insert(state.order_id.clone(),state);
            }
            RestResponse::Portfolio{positions, orders} => {
                for (figi, _) in &positions {
                    let figi = figi.clone();
                    let depth = 10; //TODO: надо бы параметризировать
                    self.streaming.send(StreamingRequest::OrderbookSubscribe {figi, depth}).await?;
                }
                self.market.update_portfolio(positions, orders)
            }
        }
        Ok(())
    }
}
