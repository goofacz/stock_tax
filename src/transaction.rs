use crate::currency::Currency;
use chrono::NaiveDateTime;

pub enum Operation {
    Buy {
        quantity: f64,
        price: Box<dyn Currency>,
        commision: Box<dyn Currency>,
    },
    Sell {
        quantity: f64,
        price: Box<dyn Currency>,
        commision: Box<dyn Currency>,
    },
}

pub struct Transaction {
    pub symbol: String,
    pub timestamp: NaiveDateTime,
    pub operation: Operation,
}
