
pub mod entities;
use std::{str::FromStr, unimplemented};

use futures_util::{SinkExt, StreamExt, select};
use tungstenite::Message;
use async_channel::{Sender, Receiver};
use entities::{Request, Response};

pub fn start_client(token: String, receiver: Receiver<Request>, sender: Sender<Response>) {
    tokio::spawn ( async move {
        let req = http::Request::builder()
            .uri("wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws")
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();
        
        let (mut websocket, _response) = tokio_tungstenite::connect_async(req).await.unwrap();
        println!("Connected");
        loop {
            tokio::select! {
                Some(msg) = websocket.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match Response::from_str(&text) {
                                Ok(msg) => sender.send(msg).await.unwrap(),
                                Err(e) => println!("error on parsing text: {} \n {:?}", text, e),
                            };
                        },
                        Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {println!("ping")},
                        Ok(Message::Binary(_)) => {},
                        Ok(Message::Close(_msg)) => println!("closing need to be processed"),
                        Err(e) => println!("error: {:?}", e),
                    }
                },
                Ok(req) = receiver.recv() => {
                    websocket.send(Message::Text(req.to_string())).await.unwrap();
                }
            }
        }
    });
}