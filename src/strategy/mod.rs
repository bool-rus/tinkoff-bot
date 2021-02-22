mod dispatch;
mod fixed_amount;
use crate::model::{Market, Order};
use enum_dispatch::enum_dispatch;
pub use dispatch::StrategyKind;
use fixed_amount::FixedAmount;

#[derive(Debug)]
pub enum Decision {
    Relax,
    Order(Order),
}

pub trait ConfigurableStrategy: Strategy + Send {
    fn name(&self) -> &'static str {
        "UNDEFINED"
    }
    fn description(&self) -> &'static str {
        "Empty description"
    }
    fn params(&self) -> Vec<(&'static str, &'static str)> {
        Vec::new()
    }
    fn configure(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        Ok(())
    }
}

#[enum_dispatch(StrategyKind)]
pub trait Strategy {
    fn make_decision(&mut self, market: &Market) -> Decision;
    fn balance(&self) -> f64;
}

#[derive(Default, Clone)]
pub struct Dummy;

impl Strategy for Dummy {
    fn make_decision(&mut self, market: &Market) -> Decision {
        Decision::Relax
    }

    fn balance(&self) -> f64 {
        0.0
    }
}

pub use error::ConfigError;
mod error {
    use std::{error::Error, fmt::Display, num::ParseFloatError};

    #[derive(Debug)]
    pub struct  ConfigError(&'static str);
    impl Display for ConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.0)
        }
    }
    impl Error for ConfigError {}
    impl ConfigError {
        pub const INVALID_PARAM: ConfigError= ConfigError("Нет такого параметра");
    }

    impl From<ParseFloatError> for ConfigError {
        fn from(_: ParseFloatError) -> Self {
            Self("Это не дробное число")
        }
    }
}








/*
pub struct StrategyProfiler<T: Strategy> {
    strategy: T,
    figi: String,
    date_start: Date<FixedOffset>,
    date_end: Date<FixedOffset>,
}

impl <T: Strategy> StrategyProfiler<T> {
    pub fn new(strategy: T, figi: String) -> Self {
        let today = chrono::Utc::now().with_timezone(&FixedOffset::east(3*3600)).date();
        Self {
            strategy, 
            figi,
            date_start: today - Duration::days(200),
            date_end: today-Duration::days(1),
        }
    }
}

impl <T: Strategy> Strategy for StrategyProfiler<T> {
    fn make_decision(&mut self, market: &Market) -> Decision {
        self.date_start = self.date_start + Duration::days(1);
        if self.date_start < self.date_end  {
            println!("retrieve balance for {:?}", self.date_start);
            return Decision::CallRest(rest::entities::Request::GetCandles { 
                figi: self.figi.clone(), 
                from: self.date_start.and_hms(0, 0, 0), 
                to: self.date_start.and_hms(23, 59, 59), 
                interval: Interval::MIN1,
            })
        } else {
            let mut fake = market.clone();
            let offset = FixedOffset::east(0);
            let mut counter = 0;
            println!("candles: {}", market.stocks.get(&self.figi).unwrap().candles.len());
            for candle in &market.stocks.get(&self.figi).unwrap().candles {
                let bid = f64::min(candle.open, candle.close);
                let ask = f64::max(candle.open, candle.close);
                fake.stocks.get_mut(&self.figi).unwrap().orderbook = Orderbook {
                    time: Local::now().with_timezone(&offset),
                    bids: vec![(bid, 100)],
                    asks: vec![(ask, 100)],
                };
                if let Decision::Order(order) = self.strategy.make_decision(&fake) {
                    let Order {kind, quantity, ..} = order;
                    if quantity == 0 { continue; }
                    counter += 1;
                    if counter % 1 == 0 {
                        println!("{:?} {}, balance: {}", kind, quantity, self.strategy.balance());
                    }
                    let papers = fake.positions.get_mut(&self.figi).unwrap();
                    match kind {
                        crate::model::OrderKind::Buy => *papers += quantity,
                        crate::model::OrderKind::Sell => *papers -= quantity,
                    }
                }
                
            }
            println!("result balance: {}, papers: {}", self.strategy.balance(), fake.positions.get(&self.figi).unwrap());
            Decision::Relax
        }
    }

    fn balance(&self) -> f64 {
        self.strategy.balance()
    }
}
*/