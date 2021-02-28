use std::collections::HashMap;

use async_channel::Receiver;

use crate::strategy::StrategyKind; 
use crate::trader::entities::Response;
use telegram_bot::*;

use super::fsm::State;

pub enum Event {
    Start,
    Portfolio,
    Strategy,
    Finish,
    Text(String),
    Select(String),
    Unknown,
}

impl From<UpdateKind> for Event {
    fn from(u: UpdateKind) -> Self {
        match u {
            UpdateKind::Message(Message {kind: MessageKind::Text {data, entities}, ..}) => {
                if entities.is_empty() {
                    return Self::Text(data)
                }
                for entity in entities {
                    if matches!(entity.kind, MessageEntityKind::BotCommand) {
                        match invoke(&entity, &data).as_ref() {
                            "/start" => return Self::Start,
                            "/portfolio" => return Self::Portfolio,
                            "/strategy" => return Self::Strategy,
                            "/finish" => return Self::Finish,
                            _ => {},
                        }
                    }
                }
                Self::Unknown
            }
            UpdateKind::CallbackQuery(query) => query.data.map(|s|Self::Select(s)).unwrap_or(Self::Unknown),
            _ => Self::Unknown
        }
    }
}

fn invoke(entity: &MessageEntity, data: &str) -> String {
    let MessageEntity {offset, length, ..} = entity;
    let chars: Vec<_> = data.encode_utf16().skip(*offset as usize).take(*length as usize).collect();
    String::from_utf16(&chars).unwrap()
} 

pub enum ResponseMessage {
    Dummy,
    RequestToken,
    RequestStrategyName,
    TraderStarted,
    InProgress,
    TraderStopped,
    SelectStrategy,
    SelectStrategyParam(Vec<(&'static str, &'static str)>),
    RequestParamValue,
    StrategyAdded,
    Err(String),
}
pub enum Command {
    Start,
    Portfolio,
    Strategy, 
    Unknown,
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "/start" => Self::Start,
            "/portfolio" => Self::Portfolio,
            "/strategy" => Self::Strategy,
            _ => Self::Unknown,
        }
    }
}

pub struct Storage {
    context: Context,
    state: State,
}

impl Storage {
    pub fn new(api: Api, chat_id: ChatId) -> Self {
        let context = Context::new(api, chat_id);
        let state = State::New;
        Self {context, state}
    }
    pub async fn on_event(&mut self, event: Event) {
        let mut state = State::New;
        std::mem::swap(&mut self.state, &mut state);
        self.state = match state.on_event(&mut self.context, event).await {
            Ok(state) => state,
            Err(_e) => {
                self.context.send(ResponseMessage::TraderStopped).await;
                State::New
            }
        };
    }
    pub fn invoke_receiver(&mut self) -> Option<Receiver<Response>> {
        self.context.invoke_receiver()
    }
}

pub struct Context {
    api: Api,
    chat_id: ChatId,
    strategy_types: HashMap<String, StrategyKind>,
    strategies: HashMap<String, String>,
    receiver: Option<Receiver<Response>>,
}

impl Context {
    pub fn new(api: Api, chat_id: ChatId) -> Self {
        let strategy_types = StrategyKind::variants();
        let strategies = HashMap::new();
        Self {api, chat_id, strategy_types, strategies, receiver: None}
    }
    fn strategies_markup(&self) -> ReplyMarkup {
        let buttons: Vec<_> = self.strategy_types.keys().map(|s|{
            vec![InlineKeyboardButton::callback(s.to_owned(), s.to_owned())]
        }).collect();
        buttons.into()
    }
    pub async fn send(&mut self, msg: ResponseMessage) {
        let chat_id = self.chat_id;
        match msg {
            ResponseMessage::Dummy => { self.api.send(chat_id.text("Сорян, мне нечего ответить...")).await; }
            ResponseMessage::RequestToken => { self.api.send(chat_id.text("Принял, засылай токен")).await; }
            ResponseMessage::TraderStarted => { self.api.send(chat_id.text("Красава, подключаюсь...")).await; }
            ResponseMessage::InProgress => { self.api.send(SendChatAction::new(chat_id, ChatAction::Typing)).await; }
            ResponseMessage::TraderStopped => { self.api.send(chat_id.text("Упс, я обосрался... Давай сначала")).await; }
            ResponseMessage::RequestStrategyName => { self.api.send(chat_id.text("Придумай имя для своей стратегии")).await; }
            ResponseMessage::SelectStrategy => {
                let mut msg = chat_id.text("Выбирай стратегию");
                msg.reply_markup(self.strategies_markup());
                self.api.send(msg).await;
             }
            ResponseMessage::SelectStrategyParam(params) => {
                let buttons: Vec<_> = params.into_iter().map(|(name, desc)|{
                    vec![InlineKeyboardButton::callback(desc, name)]
                }).collect();
                let mut msg = chat_id.text("Выбирай параметр для настройки");
                msg.reply_markup(buttons);
                self.api.send(msg).await;
            }
            ResponseMessage::RequestParamValue => { self.api.send(chat_id.text("Ок, пиши значение")).await; }
            ResponseMessage::StrategyAdded => { self.api.send(chat_id.text("Ок, стратегия добавлена")).await; }
            ResponseMessage::Err(s) => { self.api.send(chat_id.text(s)).await; }
        }
    }
    pub fn strategy_by_type(&self, type_name: &str) -> Option<StrategyKind> {
        self.strategy_types.get(type_name).map(Clone::clone)
    }
    pub fn add_strategy(&mut self, name: String, strategy: String) {
        self.strategies.insert(name, strategy);
    }
    pub fn set_receiver(&mut self, r: Receiver<Response>) {
        self.receiver = Some(r)
    }
    pub fn invoke_receiver(&mut self) -> Option<Receiver<Response>> {
        let mut result = None;
        std::mem::swap(&mut result, &mut self.receiver);
        result
    }
}
