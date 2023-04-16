use derive_more::{Add, Sub};
use macros;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::fmt;
use std::fmt::Debug;

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
}
