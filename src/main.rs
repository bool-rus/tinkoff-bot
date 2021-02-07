use bot::start_bot;

mod faces;
mod convert;
mod streaming;
mod rest;
mod strategy;
mod bot;

#[tokio::main]
async fn main() { 
    let bot = tokio::spawn(async {start_bot().await});
    bot.await.unwrap();
}
