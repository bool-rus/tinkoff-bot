use simplelog::*;
use tokio_compat_02::FutureExt;
use trader::{Trader, TraderConf};

mod model;
mod convert;
mod streaming;
mod rest;
mod strategy;
mod telega;
mod trader;

#[tokio::main]
async fn main() { 
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();
    let handle = tokio::spawn(async {
        telega::start().compat().await.unwrap()
    });
    handle.await.unwrap();
}
