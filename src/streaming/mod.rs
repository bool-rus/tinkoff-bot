
pub mod entities;
use std::{collections::HashSet, str::FromStr, unimplemented};
use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};
use tungstenite::{Message, http};
use async_channel::{Sender, Receiver};
use entities::{Request, Response};

async fn connect(uri: &str, token: &str) ->  Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {

    let req = http::Request::builder()
    .uri(uri)
    .header("Authorization", format!("Bearer {}", token))
    .body(())
    .unwrap();

    let (websocket, _response) = tokio_tungstenite::connect_async(req).await?;
    println!("[streaming] Connected");
    Ok(websocket)
}

pub struct Service {
    token: String,
    uri: String,
    need_pong: bool,
    state: HashSet<Request>,
    sender: Sender<Response>,
    receiver: Receiver<Request>,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Service {
    pub fn start(token: String, uri: String) -> (Sender<Request>, Receiver<Response>) {
        let (sender,r) = async_channel::bounded(100);
        let (s, receiver) = async_channel::bounded(100);
        tokio::spawn (async move {
            let websocket = connect(&uri, &token).await.unwrap();
            Self {token, uri, need_pong: false, state: HashSet::new(), sender, receiver, websocket, }.run().await;
        });
        (s,r)
    }
    async fn run(&mut self) {
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
                Ok(msg) => self.sender.send(msg).await.unwrap(),
                Err(e) => println!("error on parsing text: {} \n {:?}", text, e),
            },
            Ok(Message::Ping(data)) => self.websocket.send(Message::Pong(data)).await.unwrap(),
            Ok(Message::Pong(_)) => self.need_pong = false,
            Ok(Message::Close(_)) => self.reconnect().await,
            Ok(Message::Binary(_)) => {},
            Err(e) => {
                println!("error read from websocket: {:?}", e);
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
            println!("pong not received. need reconnect");
            self.reconnect().await;
            return;
        }
        match self.websocket.send(Message::Ping(vec![])).await {
            Ok(_) => self.need_pong = true,
            Err(e) => {
                println!("cannot send Ping: {:?}", e);
                self.reconnect().await;
            }
        }
    }
    async fn reconnect(&mut self) {
        self.websocket.send(Message::Close(None)).await.unwrap_or(());
        loop {
            match connect(&self.uri, &self.token).await {
                Ok(ws) => {
                    self.websocket = ws;
                    self.need_pong = false;
                    break;
                }
                Err(e) => println!("cannot reconnect: {:?}", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(61)).await;
        }

        for r in self.state.iter() {
            self.websocket.send(r.into()).await;
        }
    }
}