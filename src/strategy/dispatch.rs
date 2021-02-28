use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use super::fixed_amount::FixedAmount;
use super::trailing_stop::TrailingStop;
use super::Strategy;
use strum::IntoEnumIterator;
use strum::EnumIter;


#[enum_dispatch(Strategy)]
#[derive(EnumIter, Clone, PartialEq)]
pub enum StrategyKind {
    FixedAmount,
    TrailingStop,
}

impl StrategyKind {
    pub fn variants() -> HashMap<String, StrategyKind> {
        Self::iter().fold(HashMap::new(), |mut map, s| {
            map.insert(s.name().to_owned(), s);
            map
        })
    }
}