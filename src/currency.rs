use derive_more::Display;
use derive_more::{Add, AddAssign, Sub};
use macros;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;
use std::ops::{Div, Mul};

#[derive(Debug, Deserialize, Serialize, Display, PartialEq)]
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
    macros::Display,
)]
pub struct Usd(pub Decimal);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Add,
    AddAssign,
    Sub,
    PartialEq,
    Copy,
    Clone,
    Default,
    macros::Currency,
    macros::Mul,
    macros::Div,
    macros::Display,
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
    macros::Display,
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
    macros::Display,
)]
pub struct Gbp(pub Decimal);

impl fmt::Display for dyn Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.get_value(), self.get_code())
    }
}

pub fn new(name: &Code, value: Decimal) -> Box<dyn Currency> {
    match name {
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
