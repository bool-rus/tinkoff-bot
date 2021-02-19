use simplelog::*;

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
    let token = std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let handle = telega::Bot::start(token);
    handle.await.unwrap();
}
