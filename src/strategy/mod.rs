use crate::faces::{Market, Order};
pub mod fixed_amount;

#[derive(Debug)]
pub enum Decision {
    Relax,
    Order(Order)
}

pub trait Strategy {
    fn make_decision(&mut self, market: &Market) -> Decision;
}
