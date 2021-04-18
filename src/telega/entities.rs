use std::collections::HashMap;

use async_channel::Receiver;
use log::info;

use crate::{model::Stock, strategy::{ConfigError, Strategy, StrategyKind}}; 
use telegram_bot::*;
use super::fsm::State;
use super::persistent::SavedState;

type Response = crate::trader::entities::Response<StrategyKind>;

pub enum Event {
    Start,
    Portfolio,
    Strategies,
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
                            "/strategies" => return Self::Strategies,
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
    Strategies,
    StrategyInfo(String, StrategyKind),
    Err(String),
}

pub struct Storage {
    pub context: Context,
    state: State,
}

impl Storage {
    pub fn new(api: Api, chat_id: ChatId) -> Self {
        let context = Context::new(api, chat_id);
        let state = State::New;
        Self {context, state}
    }
    pub fn as_saved_state(&self) -> Option<SavedState<StrategyKind>> {
        Some(SavedState::new( 
            self.state.token()?.to_owned(),
            self.context.strategies.clone()
        ))
    }
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub async fn on_event(&mut self, event: Event) -> Option<Receiver<Response>> {
        let mut state = State::New;
        std::mem::swap(&mut self.state, &mut state);
        let is_waiting_token = matches!(&state, &State::WaitingToken);
        let mut result = None;
        self.state = match state.on_event(&mut self.context, event).await {
            Ok(State::Connected(handle)) => {
                if is_waiting_token {
                    result = Some(handle.receiver());
                }
                State::Connected(handle)
            }
            Ok(state) => state,
            Err(_e) => {
                self.context.send(ResponseMessage::TraderStopped).await;
                State::New
            }
        };
        result
    }
}

pub struct Context {
    api: Api,
    chat_id: ChatId,
    stocks: HashMap<String, Stock>,
    strategy_types: HashMap<String, StrategyKind>,
    strategies: HashMap<String, StrategyKind>,
}

impl Context {
    pub fn new(api: Api, chat_id: ChatId) -> Self {
        let strategy_types = StrategyKind::variants();
        let strategies = HashMap::new();
        let stocks = Default::default();
        Self {api, chat_id, strategy_types, strategies, stocks}
    }
    fn strategies_markup(&self) -> ReplyMarkup {
        let buttons: Vec<_> = self.strategy_types.keys().map(|s|{
            vec![InlineKeyboardButton::callback(s.to_owned(), s.to_owned())]
        }).collect();
        buttons.into()
    }
    pub async fn set_parameter<S: Strategy>(&self, strategy: &mut S, key: &str, value: String) -> Result<(),ConfigError> {
        if key == "ticker" {
            if let Some(value) = self.stocks.get(&value) {
                strategy.configure("figi", value.figi.clone())?;
                self.api.send(self.chat_id.text(format!("Бумага найдена: {}", value.name))).await;
            } else {
                return Err(ConfigError::TICKER_NOT_FOUND);
            }
        } else {
            strategy.configure(key, value)?;
        }
        Ok(())
    }
    pub fn strategy(&self, key: &str) -> Option<&StrategyKind> {
        self.strategies.get(key)
    }
    pub async fn send(&self, msg: ResponseMessage) {
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
                let buttons: Vec<_> = params.into_iter().map(|(mut name, mut desc)|{
                    if name == "figi" {
                        name = "ticker";
                        desc = "Тикер бумаги";
                    }
                    vec![InlineKeyboardButton::callback(desc, name)]
                }).collect();
                let mut msg = chat_id.text("Выбирай параметр для настройки");
                msg.reply_markup(buttons);
                self.api.send(msg).await;
            }
            ResponseMessage::RequestParamValue => { self.api.send(chat_id.text("Ок, пиши значение")).await; }
            ResponseMessage::StrategyAdded => { self.api.send(chat_id.text("Ок, стратегия добавлена")).await; }
            ResponseMessage::Strategies => {
                let mut msg = chat_id.text("Стратегии".to_owned());
                let buttons: Vec<_> = self.strategies.keys().map(|k|vec![InlineKeyboardButton::callback(k.clone(), k.clone())]).collect();
                msg.reply_markup(buttons);
                self.api.send(msg).await;
            }
            ResponseMessage::Err(s) => { self.api.send(chat_id.text(s)).await; }
            ResponseMessage::StrategyInfo(key, s) => {
                let msg = format!("Инфо по стратегии {}\n{}, \n\t{}\nБаланс: {}",key, s.name(), s.description(), s.balance());
                self.api.send(chat_id.text(msg)).await;
            }
        }
    }
    pub fn update_strategies(&mut self, strategies: HashMap<String, StrategyKind>) {
        self.strategies = strategies;
    }
    pub fn strategy_by_type(&self, type_name: &str) -> Option<StrategyKind> {
        self.strategy_types.get(type_name).map(Clone::clone)
    }
    pub fn set_stocks(&mut self, stocks: HashMap<String, Stock>) {
        self.stocks = stocks;
    }
}
