mod convert;
pub mod entities;

use async_channel::{Receiver, Sender};
use entities::*;
use tinkoff_api::apis::configuration::Configuration;
use tinkoff_api::apis::market_api::*;
use tinkoff_api::apis::orders_api::*;
use tinkoff_api::apis::portfolio_api::*;
use tinkoff_api::models::LimitOrderRequest;
use tokio_compat_02::FutureExt;

use crate::model::{Order, OrderState};

pub fn start_client(
    token: String,
    root_url: String,
    receiver: Receiver<Request>,
    sender: Sender<Response>,
) {
    let conf = Configuration {
        base_path: root_url,
        bearer_access_token: Some(token),
        ..Default::default()
    };
    tokio::spawn(async move {
        while let Ok(req) = receiver.recv().await {
            match send(&conf, req.clone()).await {
                Ok(res) => sender.send(res).await.unwrap(),
                Err(e) => sender.send(Response::Err(req, e)).await.unwrap(),
            }
        }
    });
}

async fn send(conf: &Configuration, request: Request) -> Result<Response, ErrX> {
    Ok(match request {
        Request::GetStocks => market_stocks_get(conf).compat().await?.into(),
        Request::GetETFs => market_etfs_get(conf).compat().await?.into(),
        Request::GetBonds => market_bonds_get(conf).compat().await?.into(),
        Request::GetCandles {figi,from,to,interval,} => {
            let response =
                market_candles_get_own(&conf, &figi, from.to_rfc3339(), to.to_rfc3339(), "1min")
                    .compat()
                    .await?;
            Response::Candles {
                figi,
                candles: response
                    .payload
                    .candles
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }
        }
        Request::LimitOrder(key, order) => {
            let tinkoff_api::models::PlacedLimitOrder { executed_lots, order_id, .. } = orders_limit_order_post(
                &conf,
                    &order.figi,
                    LimitOrderRequest {
                        lots: order.quantity as i32,
                        operation: order.kind,
                        price: order.price,
                    },
                    None,
                ).compat().await?.payload;
            Response::Order(key, OrderState {order_id, order, executed: executed_lots as u32})
        }
        Request::GetOrders => orders_get(&conf, None).compat().await?.into(),
        Request::GetPositions => portfolio_get(&conf, None).compat().await?.into(),
    })
}

pub async fn market_candles_get_own(
    configuration: &Configuration,
    figi: &str,
    from: String,
    to: String,
    interval: &str,
) -> Result<tinkoff_api::models::CandlesResponse, tinkoff_api::apis::Error<MarketCandlesGetError>> {
    let local_var_client = &configuration.client;

    let local_var_uri_str = format!("{}/market/candles", configuration.base_path);
    let mut local_var_req_builder = local_var_client.get(local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("figi", &figi.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("from", &from.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("to", &to.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("interval", &interval.to_string())]);
    if let Some(ref local_var_user_agent) = configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if local_var_status.is_success() {
        serde_json::from_str(&local_var_content).map_err(tinkoff_api::apis::Error::from)
    } else {
        let local_var_entity: Option<MarketCandlesGetError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = tinkoff_api::apis::ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(tinkoff_api::apis::Error::ResponseError(local_var_error))
    }
}