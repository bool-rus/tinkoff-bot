use crate::faces::Order;
pub mod static_amount;

#[derive(Debug)]
pub enum Decision {
    Relax,
    Order(Order)
}

pub trait Strategy {
    fn make_decision(&mut self) -> Decision;
}
