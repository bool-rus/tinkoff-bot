pub mod entities;

use async_channel::{Receiver, Sender};
use entities::*;
use tinkoff_api::apis::configuration::Configuration;
use tinkoff_api::apis::market_api::*;
use tokio_compat_02::FutureExt;

pub fn start_client(token: String, root_url: String, receiver: Receiver<Request>, sender: Sender<Response>) {
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
        Request::GetStocks => Response::Stocks(market_stocks_get(conf).compat().await?.payload.instruments.iter().map(Into::into).collect()),
        Request::GetETFs => Response::ETFs(market_stocks_get(conf).compat().await?.payload.instruments.iter().map(Into::into).collect()),
        Request::GetBonds => Response::Bonds(market_stocks_get(conf).compat().await?.payload.instruments.iter().map(Into::into).collect()),
        Request::GetCandles { figi, from, to, interval } => {
            let response = market_candles_get(&conf, &figi, from.to_string(), to.to_string(), interval.into()).compat().await?;
            Response::Candles {
                figi: figi,
                candles: response.payload.candles.into_iter().map(Into::into).collect(),
            }
        }
    })
}
