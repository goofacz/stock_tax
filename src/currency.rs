use crate::nbp;
use crate::tax::Tax;
use chrono::NaiveDateTime;
use derive_more::{Add, AddAssign, Sub};
use derive_more::{Display, Error};
use macros;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::ops::{Div, Mul};

#[derive(Display, Error, Debug, PartialEq)]
pub struct Error {
    reason: String,
}

impl Error {
    fn new(reason: &str) -> Error {
        Error {
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Display, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Code {
    PLN,
    USD,
    GBP,
    EUR,
}

pub trait Currency: Debug {
    fn get_value(&self) -> &Decimal;
    fn get_code(&self) -> Code;
}

pub trait Builder<T> {
    fn new(amount: T) -> Self;
    fn new_box(amount: T) -> Box<Self>;
}

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    PartialOrd,
    Copy,
    Clone,
    Default,
    macros::Currency,
)]
pub struct Usd(Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    AddAssign,
    Sub,
    PartialEq,
    PartialOrd,
    Copy,
    Clone,
    Default,
    macros::Currency,
)]
pub struct Pln(Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    PartialOrd,
    Copy,
    Clone,
    Default,
    macros::Currency,
)]
pub struct Eur(Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    Sub,
    PartialEq,
    PartialOrd,
    Copy,
    Clone,
    Default,
    macros::Currency,
)]
pub struct Gbp(Decimal);

#[derive(Debug, Deserialize, Serialize)]
pub struct Rates {
    values: HashMap<(Code, NaiveDateTime), nbp::Rate>,
}

impl fmt::Display for dyn Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.get_value(), self.get_code())
    }
}

pub fn new(code: &Code, value: Decimal) -> Box<dyn Currency> {
    match code {
        Code::PLN => Box::new(Pln(value)),
        Code::USD => Box::new(Usd(value)),
        Code::GBP => Box::new(Gbp(value)),
        Code::EUR => Box::new(Eur(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_currency() {
        let usd = Usd(dec!(3.45));
        assert_eq!(usd.get_value().clone(), dec!(3.45));
        assert_eq!(usd.get_code(), Code::USD);

        let pln = Pln(dec!(1.23));
        assert_eq!(pln.get_value().clone(), dec!(1.23));
        assert_eq!(pln.get_code(), Code::PLN);
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
}
