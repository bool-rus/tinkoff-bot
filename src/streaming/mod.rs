
pub mod entities;
use std::{collections::HashSet, str::FromStr};
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

pub fn start_client(token: String, uri: String, receiver: Receiver<Request>, sender: Sender<Response>) {
    tokio::spawn ( async move {
        let mut websocket = connect(&uri, &token).await.unwrap();
        let mut state = HashSet::<Request>::new();
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
                        Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {},
                        Ok(Message::Binary(_)) => {},
                        Ok(Message::Close(_msg)) => println!("closing need to be processed"),
                        Err(e) => {
                            println!("error: {:?}\n reconnecting...",e);
                            //TODO: maybe websocket.send(Message::Close(None)).await;
                            websocket = connect(&uri, &token).await.unwrap();
                            for r in state.iter() {
                                websocket.send(r.into()).await.unwrap();
                            }
                        },
                    }
                },
                Ok(req) = receiver.recv() => {
                    state.insert(req.clone());
                    websocket.send((&req).into()).await.unwrap();
                }
            }
        }
    });
}