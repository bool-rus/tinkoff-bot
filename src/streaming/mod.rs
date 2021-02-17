
pub mod entities;
use std::{collections::HashSet, str::FromStr, unimplemented};
use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};
use tungstenite::{Message, http};
use async_channel::{Sender, Receiver};
use entities::{Request, Response};

use crate::model::ServiceHandle;

pub use entities::{Request as StreamingRequest, Response as StreamingResponse};

async fn connect(uri: &str, token: &str) ->  Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {

    let req = http::Request::builder()
    .uri(uri)
    .header("Authorization", format!("Bearer {}", token))
    .body(())
    .unwrap();

    let (websocket, _response) = tokio_tungstenite::connect_async(req).await?;
    log::info!("websocket connected");
    Ok(websocket)
}

pub struct Streaming {
    token: String,
    uri: String,
    need_pong: bool,
    state: HashSet<Request>,
    sender: Sender<Response>,
    receiver: Receiver<Request>,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Streaming {
    pub fn start(token: String, uri: String) -> ServiceHandle<Request, Response> {
        let (sender,r) = async_channel::bounded(100);
        let (s, receiver) = async_channel::bounded(100);
        tokio::spawn (async move {
            let websocket = connect(&uri, &token).await.unwrap();
            Self {token, uri, need_pong: false, state: HashSet::new(), sender, receiver, websocket, }.run().await;
        });
        ServiceHandle::new( s,  r)
    }
    async fn run(mut self) {
        log::info!("Streaming service started");
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(17));
        loop {tokio::select! {
            Some(msg) = self.websocket.next() => self.on_response(msg).await,
            req = self.receiver.recv() => match req {
                Ok(req) => {self.on_command(req).await;}
                Err(_) => break,
            },
            _ = timer.tick() => self.on_timer().await,
        }}
    }
    async fn on_response(&mut self, msg: Result<Message, tungstenite::error::Error>) {
         match msg {
            Ok(Message::Text(text)) => match Response::from_str(&text) {
                Ok(msg) => {self.sender.send(msg).await;},
                Err(e) => log::error!("error on parsing text: {} \n {:?}", text, e),
            },
            Ok(Message::Ping(data)) => {self.websocket.send(Message::Pong(data)).await;},
            Ok(Message::Pong(_)) => self.need_pong = false,
            Ok(Message::Close(_)) => self.reconnect().await,
            Ok(Message::Binary(_)) => {},
            Err(e) => {
                log::warn!("error read from websocket: {:?}, reconnecting...", e);
                self.reconnect().await;
            }
        }
    }
    async fn on_command(&mut self, req: Request) -> Result<(), tungstenite::error::Error>{
        use Request::*;
        match req.clone() {   
            CandleSubscribe { .. } | 
            InfoSubscribe { .. } |
            OrderbookSubscribe{ .. } => { self.state.insert(req.clone()); }
            CandleUnsubscribe { figi, interval } => { self.state.remove( &CandleSubscribe { figi, interval } ); }
            OrderbookUnsubscribe { figi, depth } => { self.state.remove( &OrderbookSubscribe { figi, depth } ); }
            InfoUnsubsribe { figi } => { self.state.remove( &InfoSubscribe { figi } ); }
        }
        self.websocket.send((&req).into()).await
    }
    async fn on_timer(&mut self) {
        if self.need_pong {
            log::warn!("pong not received, reconnecting...");
            self.reconnect().await;
            return;
        }
        match self.websocket.send(Message::Ping(vec![])).await {
            Ok(_) => self.need_pong = true,
            Err(e) => {
                log::warn!("cannot send Ping: {:?}, reconnecting...", e);
                self.reconnect().await;
            }
        }
    }
    async fn resubscribe(&mut self) -> Result<(), tungstenite::error::Error> {
        for r in &self.state {
            self.websocket.send(r.into()).await?;
        }
        Ok(())
    }
    async fn reconnect(&mut self) {
        self.websocket.send(Message::Close(None)).await.unwrap_or(());
        loop {
            match connect(&self.uri, &self.token).await {
                Ok(ws) => {
                    self.websocket = ws;
                    self.need_pong = false;
                    match self.resubscribe().await {
                        Ok(_) => break,
                        Err(e) => log::error!("cannoct resubscribe: {:?}", e),
                    }
                }
                Err(e) => log::error!("cannot reconnect: {:?}", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(61)).await;
        }
    }
}