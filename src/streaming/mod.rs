
pub mod entities;
use std::str::FromStr;

use futures_util::{SinkExt, StreamExt};
use tungstenite::Message;
use async_channel::Sender;
use entities::Response;

pub async fn start_client(sender: Sender<Response>) -> impl SinkExt<Message> {

    let req = http::Request::builder()
        .uri("wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws")
        .header("Authorization", "Bearer t.xwGtvjeVXUHM0JwVh9IYDGB5JsISXS51m63-PKNfQT4zz2Xkl4KHW-OvpoYgBHYuN9JfV5DcNB2WJjfpoKv5Kg")
        .body(())
        .unwrap();
    
    let (websocket, _response) = tokio_tungstenite::connect_async(req).await.unwrap();
    let (sink, mut stream) = websocket.split();
    tokio::spawn ( async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match Response::from_str(&text) {
                        Ok(msg) => {sender.send(msg).await;},
                        Err(e) => println!("error on parsing text: {} \n {:?}", text, e),
                    };
                },
                Err(e) => println!("error: {:?}", e),
                Ok(msg) => println!("unknown response: {:?}", msg),
            }
        }
    });
    sink
    
}