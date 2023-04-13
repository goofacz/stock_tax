use chrono::NaiveDate;
use derive_more::{Add, Sub};
use macros;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Div, Mul};

pub trait Currency: Debug {
    fn get_value(&self) -> &Decimal;
    fn get_name(&self) -> &str;
}

impl fmt::Display for dyn Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.get_value(), self.get_name())
    }
}

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    Copy,
    Clone,
    Default,
    macros::Currency,
    macros::Mul,
    macros::Div,
)]
pub struct Usd(pub Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    Copy,
    Clone,
    Default,
    macros::Currency,
    macros::Mul,
    macros::Div,
)]
pub struct Pln(pub Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    Copy,
    Clone,
    Default,
    macros::Currency,
    macros::Mul,
    macros::Div,
)]
pub struct Eur(pub Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    Copy,
    Clone,
    Default,
    macros::Currency,
    macros::Mul,
    macros::Div,
)]
pub struct Gbp(pub Decimal);

#[derive(Debug)]
pub struct Rate<T> {
    rates: HashMap<NaiveDate, Decimal>,
    phantom_data: PhantomData<T>,
}

impl<T> Rate<T>
where
    T: Currency,
{
    pub fn new(rates: HashMap<NaiveDate, Decimal>) -> Rate<T> {
        Rate {
            rates: rates,
            phantom_data: PhantomData,
        }
    }

    pub fn convert(&self, amount: &T, date: &NaiveDate) -> Result<Pln, ()> {
        let rate = self.rates.get(date).ok_or(())?;
        let result = amount.get_value().checked_mul(*rate).unwrap().round_dp(2);
        Ok(Pln(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_currency() {
        let a = Usd(dec!(3.45));
        assert_eq!(a.get_value().clone(), dec!(3.45));
        assert_eq!(a.get_name(), "USD");
    }

    #[test]
    fn test_currency_math() {
        let a = Usd(dec!(3.45));
        let b = Usd(dec!(7.));
        assert_eq!(a + b, Usd(dec!(10.45)));
        assert_eq!(b + a, Usd(dec!(10.45)));
        assert_eq!(a - b, Usd(dec!(-3.55)));
        assert_eq!(b - a, Usd(dec!(3.55)));
        assert_eq!(b * dec!(2), Usd(dec!(14.)));
        assert_eq!(b * dec!(2.5), Usd(dec!(17.5)));
        assert_eq!(b / dec!(2), Usd(dec!(3.5)));
        assert_eq!(b / dec!(2.5), Usd(dec!(2.8)));
    }

    #[test]
    fn test_rate() {
        let date1 = NaiveDate::from_ymd_opt(2022, 8, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2022, 8, 2).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2022, 8, 3).unwrap();
        let rates = HashMap::from([(date1, dec!(4.3)), (date2, dec!(4.7))]);

        let rates: Rate<Usd> = Rate::new(rates);
        let a = Usd(dec!(20.));
        assert_eq!(rates.convert(&a, &date1), Ok(Pln(dec!(86.))));
        assert_eq!(rates.convert(&a, &date2), Ok(Pln(dec!(94.))));
        assert_eq!(rates.convert(&a, &date3), Err(()));
    }
}
