use async_channel::Receiver;

use crate::{model::ServiceHandle, strategy::{self, Strategy as _}, trader::entities::{Request, Response}};
use crate::strategy::StrategyKind as Strategy;

use super::entities::*;

type Handle = ServiceHandle<Request, Response>;

#[derive(Clone, PartialEq)]
pub struct NamedStrategy {
    strategy: Strategy,
    name: String
}

#[derive(Clone, PartialEq)]
pub struct StrategyParam {
    strategy: NamedStrategy,
    name: String,
}

pub enum State {
    New, 
    WaitingToken,
    Connected(Handle),
    ChoosingStrategy(Handle),
    WaitingStrategyName(Handle, Strategy),
    ChoosingStrategyParam(Handle, NamedStrategy),
    WaitingStrategyParam(Handle, StrategyParam),
}

impl State {
    pub async fn on_event(self, ctx: &mut Context, event: Event) -> State {
        use State as S;
        use Event as E;
        use ResponseMessage as RM;
        match (self, event) {
            (_, E::Start) => {
                ctx.send(RM::RequestToken).await;
                S::WaitingToken
            },
            (_, E::TraderFail) => {
                ctx.send(RM::TraderStopped).await;
                S::New
            }
            (S::WaitingToken, E::Text(token)) => connect(ctx, token).await,
            (S::Connected(handle), E::Portfolio) => {
                handle.send(Request::Portfolio).await;
                ctx.send(RM::InProgress).await;
                S::Connected(handle)
            }
            (S::Connected(handle), E::Strategy) => {
                ctx.send(RM::SelectStrategy).await;
                S::ChoosingStrategy(handle)
            }
            (S::ChoosingStrategy(handle), E::Select(strategy_type)) => {
                if let Some(strategy) = ctx.strategy_by_type(&strategy_type) {
                    ctx.send(RM::RequestStrategyName).await;
                    S::WaitingStrategyName(handle, strategy)
                } else  {
                    ctx.send(RM::Dummy).await;
                    S::ChoosingStrategy(handle)
                }
            }
            (S::WaitingStrategyName(handle, strategy), E::Text(name)) => 
                to_choosing_strategy_param(ctx, handle, NamedStrategy {strategy, name}).await,
            (S::ChoosingStrategyParam(handle, NamedStrategy { strategy, name }), E::Finish) => {
                ctx.add_strategy(name.clone(), strategy.description().to_owned());
                handle.send(Request::AddStrategy(name, strategy)).await;
                ctx.send(RM::StrategyAdded).await;
                S::Connected(handle)
            }
            (S::ChoosingStrategyParam(handle, strategy), E::Select(name)) => {
                ctx.send(RM::RequestParamValue).await;
                S::WaitingStrategyParam(handle, StrategyParam { strategy, name })
            }
            (S::WaitingStrategyParam(handle, StrategyParam { mut strategy, name }), E::Text(value)) => {
                match strategy.strategy.configure(&name, value) {
                    Ok(()) => to_choosing_strategy_param(ctx, handle, strategy).await,
                    Err(e) => with_err(ctx, S::WaitingStrategyParam(handle, StrategyParam {strategy, name}), e).await
                    
                }
            }
            (state, _) => {
                ctx.send(RM::Dummy).await;
                state
            },
        }
    }
}

async fn with_err<E: std::fmt::Display>(ctx: &mut Context, state: State, err: E) -> State {
    let msg = format!("Упс... {}", err);
    ctx.send(ResponseMessage::Err(msg)).await;
    state
}

async fn to_choosing_strategy_param(ctx: &mut Context, handle: Handle, strategy: NamedStrategy) -> State {
    let params = strategy.strategy.params();
    ctx.send(ResponseMessage::SelectStrategyParam(params)).await;
    State::ChoosingStrategyParam(handle, strategy)
}

async fn connect(ctx: &mut Context, token: String) -> State {
    use crate::trader::{Trader, TraderConf};
    let conf = TraderConf {
        rest_uri: "https://api-invest.tinkoff.ru/openapi/sandbox/".to_owned(),
        streaming_uri: "wss://api-invest.tinkoff.ru/openapi/md/v1/md-openapi/ws".to_owned(),
        token,
    };
    let handle = Trader::start(conf);
    ctx.set_receiver(handle.receiver());
    ctx.send(ResponseMessage::TraderStarted).await;
    State::Connected(handle)
}
