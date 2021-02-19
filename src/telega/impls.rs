
use telegram_bot::{Message, MessageEntity, MessageEntityKind, MessageKind};

use crate::model::ServiceHandle;
use crate::trader::{Trader, TraderConf};
use crate::trader::entities::*;

use super::entities::*;


impl State {
    pub async fn process_chat(&mut self, msg: &Message) -> ResponseMessage {
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

fn invoke(entity: &MessageEntity, data: &str) -> String {
    let MessageEntity {offset, length, ..} = entity;
    let chars: Vec<_> = data.encode_utf16().skip(*offset as usize).take(*length as usize).collect();
    String::from_utf16(&chars).unwrap()
} 