use std::{collections::HashMap, future::Future, mem::swap, time::Duration};

use async_channel::Receiver;
use futures_util::{FutureExt, StreamExt, TryFutureExt, future::select_all};
use telegram_bot::*;
use traders::Traders;

use crate::{model::ServiceHandle, trader::{Trader, TraderConf}};
use crate::trader::entities::{Request, Response};

enum Command {
    Start,
    Portfolio,
    Unknown,
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "/start" => Self::Start,
            "/portfolio" => Self::Portfolio,
            _ => Self::Unknown,
        }
    }
}

enum State {
    New,
    WaitingToken,
    Connected(ServiceHandle<Request, Response>),
}

enum ResponseMessage {
    Dummy,
    RequestToken,
    TraderStarted(Receiver<Response>),
    InProgress,
    TraderStopped,
}


impl State {
    pub async fn process(&mut self, msg: &Message) -> ResponseMessage {
        let mut cache = Self::New;
        std::mem::swap(&mut cache, self);
        let (cache, res) = match cache {
            State::New => cache.at_new(msg),
            State::WaitingToken => cache.at_wait_token(msg),
            State::Connected(handle) => Self::at_connected(handle, msg).await,
        };
        *self = cache;
        res
    }
    fn at_new(self, msg: &Message) -> (Self, ResponseMessage) {
        match invoke_command(msg) {
            None |
            Some(Command::Portfolio) | 
            Some(Command::Unknown) => (self, ResponseMessage::Dummy),
            Some(Command::Start) => (Self::WaitingToken, ResponseMessage::RequestToken),
        }
    }
    fn at_wait_token(self, msg: &Message) -> (Self, ResponseMessage) {
        if let MessageKind::Text {data, entities} = &msg.kind {
            if entities.is_empty() {
                let conf = TraderConf {
                    rest_uri: "https://api-invest.tinkoff.ru/openapi/sandbox/".to_owned(),
                    streaming_uri: "wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws".to_owned(),
                    token: data.clone(),
                };
                let handle = Trader::start(conf);
                let receiver = handle.receiver();
                (Self::Connected(handle), ResponseMessage::TraderStarted(receiver))
            } else {
                (self, ResponseMessage::Dummy)
            }
        } else {
            (self, ResponseMessage::Dummy)
        }
    }
    async fn at_connected(handle: ServiceHandle<Request, Response>, msg: &Message) -> (Self, ResponseMessage) {
        match invoke_command(msg) {
            None | Some(Command::Unknown) => (Self::Connected(handle), ResponseMessage::Dummy),
            Some(Command::Start) => (Self::WaitingToken, ResponseMessage::RequestToken),
            Some(Command::Portfolio) => {
                if handle.send(Request::Portfolio).await.is_ok() {
                    (Self::Connected(handle), ResponseMessage::InProgress)
                } else {
                    (Self::New, ResponseMessage::TraderStopped)
                }
            }
        }
    }
    
}

pub async fn start() -> Result<(), Error>{

    let token = std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let mut storage = HashMap::new();
    let mut traders = Traders::new();
    let mut stocks = HashMap::new();

    let mut stream = api.stream();
   loop { tokio::select! {
        (chat, response) = &mut traders => {
            match response {
                Response::Portfolio(positions) => {
                    let text = positions.into_iter().fold("Твой портфель:".to_owned(), |prev, (stock, position)| {
                        format!("{}\n\t{}: {}\n", prev, stock.name, position.balance)
                    });
                    api.send(chat.text(text)).await;
                }
                Response::Stocks(v) => {
                    stocks = v.into_iter().fold(HashMap::new(), |mut map, stock| {
                        map.insert(stock.ticker.clone(), stock);
                        map
                    });
                }
            }
        }
        Some(update_result) = stream.next() => {
            // If the received update contains a new message...
            let update;
            match update_result {
                Ok(v) => update = v,
                Err(e) => {
                    log::error!("err: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(7)).await;
                    stream = api.stream();
                    continue;
                }
            }

            if let UpdateKind::Message(message) = update.kind {
                let chat_id = message.to_source_chat();
                let state = match storage.get_mut(&chat_id) {
                    Some(v) => v,
                    None => {
                        storage.insert(chat_id, State::New);
                        storage.get_mut(&chat_id).unwrap()
                    }
                };
                
                let is_ok = match state.process(&message).await {
                    ResponseMessage::Dummy => api.send(chat_id.text("Сорян, мне нечего ответить...")).await.is_ok(),
                    ResponseMessage::RequestToken => api.send(chat_id.text("Принял, засылай токен")).await.is_ok(),
                    ResponseMessage::TraderStarted(r) => {
                        traders.insert(chat_id, r);
                        api.send(chat_id.text("Красава, подключаюсь...")).await.is_ok()
                    }
                    ResponseMessage::InProgress => api.send(SendChatAction::new(chat_id, ChatAction::Typing)).await.is_ok(),
                    ResponseMessage::TraderStopped => api.send(chat_id.text("Упс, я обосрался... Давай сначала")).await.is_ok(),
                };
                if !is_ok {
                    log::error!("I'm in trouble! Send message failed");
                }
                
            }
        }
    }}
}

fn create_buttons() -> Vec<Vec<InlineKeyboardButton>>{
    let kb =  InlineKeyboardButton::callback;
    vec![
        vec![kb("top left", "tl"), kb("top right", "tr")],
        vec![kb("bottom left", "bl"), kb("bottom right", "br")],
    ]
}

fn invoke(entity: &MessageEntity, data: &str) -> String {
    let MessageEntity {offset, length, ..} = entity;
    let chars: Vec<_> = data.encode_utf16().skip(*offset as usize).take(*length as usize).collect();
    String::from_utf16(&chars).unwrap()
} 

fn invoke_command(msg: &Message) -> Option<Command> {
    match &msg.kind {
        MessageKind::Text { data, entities } => {
            for entity in entities {
                if matches!(entity.kind, MessageEntityKind::BotCommand) {
                    return Some(Command::from(invoke(entity, data).as_ref()))
                }
            }
            None
        }
        _ => None
    }
}

mod traders {
    use std::{collections::HashMap, future::Future, pin::Pin, task::Poll};

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

    impl Future for Traders {
        type Output = (ChatId, Response);

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            for (&chat, receiver) in &self.0 {
                if let Ok(msg) = receiver.try_recv() {
                    return Poll::Ready((chat, msg))
                }
            }
            Poll::Pending
        }
    }

}