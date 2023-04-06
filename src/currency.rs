use chrono::NaiveDate;
use derive_more::{Add, Sub};
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

#[derive(Debug, Deserialize, Serialize, Add, Sub, PartialEq, Copy, Clone)]
pub struct Usd(f64);

#[derive(Debug, Deserialize, Serialize, Add, Sub, PartialEq, Copy, Clone)]
pub struct Pln(f64);

pub struct Rate<T> {
    rates: HashMap<NaiveDate, f64>,
    phantom_data: PhantomData<T>,
}

impl Currency for Usd {
    fn get_value(&self) -> f64 {
        self.0
    }

    fn get_name(&self) -> &str {
        return "USD";
    }
}

impl<T> Mul<T> for Usd
where
    f64: From<T>,
{
    type Output = Usd;
    fn mul(self, rhs: T) -> Self {
        Usd(self.0 * f64::from(rhs))
    }
}

impl<T> Div<T> for Usd
where
    f64: From<T>,
{
    type Output = Usd;
    fn div(self, rhs: T) -> Self {
        Usd(self.0 / f64::from(rhs))
    }
}


impl Currency for Pln {
    fn get_value(&self) -> f64 {
        self.0
    }

    fn get_name(&self) -> &str {
        return "PLN";
    }
}

impl<T> Rate<T>
where
    T: Currency,
{
    pub fn convert(&self, amount: &T, date: &NaiveDate) -> Result<Pln, ()> {
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
}
