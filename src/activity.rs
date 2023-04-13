use crate::currency::Currency;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum Operation {
    Buy {
        quantity: Decimal,
        price: Box<dyn Currency>,
        commision: Box<dyn Currency>,
    },
    Sell {
        quantity: Decimal,
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
