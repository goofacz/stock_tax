use crate::currency::Currency;
use chrono::NaiveDateTime;

#[derive(Debug)]
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
    Dividend {
        value: Box<dyn Currency>,
    },
}

#[derive(Debug)]
pub struct Activity {
    pub symbol: String,
    pub timestamp: NaiveDateTime,
    pub operation: Operation,
}
