use crate::currency::Currency;
use chrono::NaiveDate;

pub struct Dividend {
    pub symbol: String,
    pub timestamp: NaiveDate,
    pub amount: Box<dyn Currency>,
}
