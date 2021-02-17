pub mod entities;

use std::time::SystemTime;

use async_channel::{Receiver, Sender};
use entities::*;
use crate::{strategy::{Decision, Dummy, Strategy}, streaming::{Streaming, StreamingRequest, StreamingResponse}};
use crate::rest::{Rest, RestRequest, RestResponse};

use crate::model::*;

pub struct TraderConf {
    pub rest_uri: String,
    pub streaming_uri: String,
    pub token: String,
}

pub struct Trader {
    sender: Sender<Response>,
    receiver: Receiver<Request>,
    streaming: ServiceHandle<StreamingRequest, StreamingResponse>,
    rest: ServiceHandle<RestRequest, RestResponse>,
    market: Market,
    strategy: Dummy,
}

impl Trader {
    pub fn start(conf: TraderConf) -> ServiceHandle<Request, Response> {
        let (sender, r) = async_channel::bounded(1000);
        let (s, receiver) = async_channel::bounded(1000);
        let TraderConf{rest_uri, streaming_uri, token} = conf;
        let trader = Self {
            sender, 
            receiver, 
            streaming: Streaming::start(token.clone(), streaming_uri), 
            rest: Rest::start(token, rest_uri), 
            market: Default::default(),
            strategy: Default::default(),
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
                    self.update_market_from_rest(msg?);
                }
                msg = self.receiver.recv() => {
                    let msg = msg.map_err(|_|ChannelStopped)?;
                    self.process_request(msg).await;
                }
                _ = timer.tick() => {
                    use crate::rest::entities::Request;
                    self.rest.send(Request::Portfolio).await?;
                }
            }
            self.new_decision().await?;
        }
    }

    async fn process_request(&mut self, request: entities::Request) {
        use entities::*;
        match request {
            Request::Portfolio => self.sender.send(Response::Portfolio(self.market.portfolio())).await.is_ok(),
        };
    }


    async fn new_decision(&mut self) -> Result<(), ChannelStopped> {
        let decision = self.strategy.make_decision(&self.market);
        match decision {
            Decision::Relax => {}
            Decision::Order(order) => {
                let stock = self.market.state_mut(&order.figi);
                let key = SystemTime::now();
                stock.new_orders.insert(key, order.clone());
                self.rest.send(crate::rest::entities::Request::LimitOrder(key, order)).await?;
            }
            Decision::CallRest(req) => self.rest.send(req).await?,
            Decision::CallStreaming(req) => self.streaming.send(req).await?,
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

    fn update_market_from_rest(&mut self, msg: RestResponse) {
        match msg {
            RestResponse::Err(request, e) => {
                if let crate::rest::entities::Request::LimitOrder(key, order, ..) = request {
                    self.market.state_mut(&order.figi).new_orders.remove(&key);
                }
                log::error!("ERR from rest!!! {:?}", e);
            }
            RestResponse::Stocks(stocks) => self.market.update_stocks(stocks),
            RestResponse::Candles { figi, candles } => {
                self.market.state_mut(&figi).candles.extend(candles.into_iter());
            }
            RestResponse::Order(key, state) => {
                let stock = self.market.state_mut(&state.order.figi);
                stock.new_orders.remove(&key);
                stock.inwork_orders.insert(state.order_id.clone(),state);
            }
            RestResponse::Portfolio{positions, orders} => self.market.update_portfolio(positions, orders),
        }
    }

}