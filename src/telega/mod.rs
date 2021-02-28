mod entities;
mod fsm;

use entities::*;
use tokio::task::JoinHandle;
use tokio_compat_02::FutureExt;

use std::collections::HashMap;
use std::time::Duration;

use futures_util::StreamExt;
use telegram_bot::*;
use traders::Traders;

use crate::model::Stock;
use crate::trader::entities::Response;

use self::fsm::State;


pub struct Bot {
    api: Api,
    storage: HashMap<ChatId, Storage>,
    traders: Traders,
    stocks: HashMap<String, Stock>,
}

impl Bot {
    async fn on_trader(&mut self, chat: ChatId, response: Response) -> Result<(), Error> {
        match response {
            Response::Portfolio(positions) => {
                let text = positions.into_iter().fold("Твой портфель:".to_owned(), |prev, (stock, position)| {
                    format!("{}\n\t{}: {}\n", prev, stock.name, position.balance)
                });
                self.api.send(chat.text(text)).await?;
            }
            Response::Stocks(v) => {
                self.stocks = v.into_iter().fold(HashMap::new(), |mut map, stock| {
                    map.insert(stock.ticker.clone(), stock);
                    map
                });
            }
        }
        Ok(())
    }
    async fn on_chat(&mut self, message: UpdateKind) -> Result<(), Error> {
        let api = &mut self.api;
        let chat_id = match &message {
            UpdateKind::Message(msg) => msg.to_source_chat(),
            UpdateKind::CallbackQuery(msg) => {
                let answer = msg.answer("Принято");    
                api.send(answer).await;        
                msg.from.to_user_id().into()
            },
            _ => return Ok(())
        };
        let storage = match self.storage.get_mut(&chat_id) {
            Some(v) => v,
            None => {
                self.storage.insert(chat_id, Storage::new(api.clone(), chat_id));
                self.storage.get_mut(&chat_id).unwrap()
            }
        };
        if let Some(r) = storage.on_event(message.into()).await {
            self.traders.insert(chat_id, r);
        }
        Ok(())
    }

    async fn run(&mut self) -> ! {
        let mut stream = self.api.stream();
        loop {
            tokio::select! {
                (chat, response) = &mut self.traders => {
                    let result = self.on_trader(chat, response).await;
                }
                Some(update_result) = stream.next() => {
                    let update;
                    match update_result {
                        Ok(v) => update = v,
                        Err(e) => {
                            log::error!("err: {:?}", e);
                            tokio::time::sleep(Duration::from_secs(7)).await;
                            stream = self.api.stream();
                            continue;
                        }
                    }
                    let result = self.on_chat(update.kind).await;
                }
            }
        }
    }

    pub fn start(token: String) -> JoinHandle<()> {
        let api = Api::new(token);
        let storage = HashMap::new();
        let traders = Traders::new();
        let stocks = HashMap::new();
        tokio::spawn( async move {
            Self {api, storage, traders, stocks}.run().compat().await
        })
    }
}

mod traders {
    use std::collections::HashMap;
    use std::task::Poll;

    use async_channel::Receiver;
    use telegram_bot::ChatId;

    use crate::trader::entities::Response;

    pub struct Traders(HashMap<ChatId, Receiver<Response>>);

    impl Traders {
        pub fn new() -> Self {
            Self(HashMap::new())
        }
        pub fn insert(&mut self, chat: ChatId, receiver: Receiver<Response>) {
            self.0.insert(chat, receiver);
        }
        pub fn remove(&mut self, chat: ChatId) {
            self.0.remove(&chat);
        }
    }

    impl Unpin for Traders {}

    impl std::future::Future for Traders {
        type Output = (ChatId, Response);

        fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            for (&chat, receiver) in &self.0 {
                if let Ok(msg) = receiver.try_recv() {
                    return Poll::Ready((chat, msg))
                }
            }
            Poll::Pending
        }
    }

}