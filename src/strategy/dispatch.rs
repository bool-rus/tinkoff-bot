use enum_dispatch::enum_dispatch;
use super::fixed_amount::FixedAmount;
use strum::IntoEnumIterator;
use strum::EnumIter;


#[enum_dispatch]
#[derive(EnumIter)]
pub enum StrategyKind {
    FixedAmount
}

impl StrategyKind {
    pub fn variants() -> Vec<StrategyKind> {
        Self::iter().collect()
    }
}