use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub enum Currency {
    PLN,
    USD,
    GBP,
    EUR,
}

#[derive(Debug)]
pub struct CurrencyRates {
    pub currency: Currency,
    pub rates: HashMap<NaiveDate, f32>,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
