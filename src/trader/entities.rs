#[derive(Clone, Copy)]
pub enum Request {
    Portfolio,
}

#[derive(Clone)]
pub enum Response {
    Portfolio(Vec<(String, f64)>),
}
