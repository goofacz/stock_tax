use chrono::NaiveDate;
use derive_more::{Add, Sub};
use macros;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Div, Mul};

pub trait Currency {
    fn get_value(&self) -> f64;
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
pub struct Usd(pub f64);

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
pub struct Pln(pub f64);

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
pub struct Eur(pub f64);

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
pub struct Gbp(pub f64);

#[derive(Debug)]
pub struct Rate<T> {
    rates: HashMap<NaiveDate, f64>,
    phantom_data: PhantomData<T>,
}

impl<T> Rate<T>
where
    T: Currency,
{
    pub fn new(rates: HashMap<NaiveDate, f64>) -> Rate<T> {
        Rate {
            rates: rates,
            phantom_data: PhantomData,
        }
    }

    pub fn convert(&self, amount: T, date: &NaiveDate) -> Result<Pln, ()> {
        let rate = self.rates.get(date).ok_or(())?;
        Ok(Pln(amount.get_value() * rate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency() {
        let a = Usd(3.45);
        assert_eq!(a.get_value(), 3.45);
        assert_eq!(a.get_name(), "USD");
    }

    #[test]
    fn test_currency_math() {
        let a = Usd(3.45);
        let b = Usd(7.);
        assert_eq!(a + b, Usd(10.45));
        assert_eq!(b + a, Usd(10.45));
        assert_eq!(a - b, Usd(-3.55));
        assert_eq!(b - a, Usd(3.55));
        assert_eq!(b * 2, Usd(14.));
        assert_eq!(b * 2.5, Usd(17.5));
        assert_eq!(b / 2, Usd(3.5));
        assert_eq!(b / 2.5, Usd(2.8));
    }

    #[test]
    fn test_rate() {
        let date1 = NaiveDate::from_ymd_opt(2022, 8, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2022, 8, 2).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2022, 8, 3).unwrap();
        let rates = HashMap::from([(date1, 4.3), (date2, 4.7)]);

        let rates: Rate<Usd> = Rate::new(rates);
        let a = Usd(20.);
        assert_eq!(rates.convert(a, &date1), Ok(Pln(86.)));
        assert_eq!(rates.convert(a, &date2), Ok(Pln(94.)));
        assert_eq!(rates.convert(a, &date3), Err(()));
    }
}
