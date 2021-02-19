use async_channel::Receiver;

use crate::model::ServiceHandle; 
use crate::trader::entities::{Request,Response};



pub enum Command {
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


pub enum ResponseMessage {
    Dummy,
    RequestToken,
    TraderStarted(Receiver<Response>),
    InProgress,
    TraderStopped,
}

pub enum State {
    New,
    WaitingToken,
    Connected(ServiceHandle<Request, Response>),
}
