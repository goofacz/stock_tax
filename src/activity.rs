use crate::currency::{Currency, Eur, Gbp, Pln, Usd};
use crate::nbp;
use chrono::naive::serde::ts_milliseconds;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Money {
    #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
    pub original: Box<dyn Currency>,
    pub pln: Pln,
    pub rate: Option<nbp::Rate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    Buy {
        quantity: Decimal,
        price: Money,
        commision: Money,
    },
    Sell {
        quantity: Decimal,
        price: Money,
        commision: Money,
    },
    Dividend {
        value: Money,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub symbol: String,
    #[serde(with = "ts_milliseconds")]
    pub timestamp: NaiveDateTime,
    pub operation: Operation,
}

fn from_currency<'de, D>(deserializer: D) -> Result<Box<dyn Currency>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let (amount, currency) = value
        .split_once(' ')
        .ok_or(de::Error::custom(format!("Failed to split \"{}\"", value)))?;
    let amount = Decimal::from_str_exact(amount)
        .map_err(|_| de::Error::custom(format!("Failed to parse \"{}\"", amount)))?;

    match currency {
        "PLN" => Ok(Box::new(Pln(amount))),
        "USD" => Ok(Box::new(Usd(amount))),
        "GBP" => Ok(Box::new(Gbp(amount))),
        "EUR" => Ok(Box::new(Eur(amount))),
        _ => Err(de::Error::custom(format!(
            "Unknown currency \"{}\"",
            currency
        ))),
    }
}

fn to_currency<S>(currency: &Box<dyn Currency>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&currency.to_string())
}

impl Money {
    pub fn new(
        value: Box<dyn Currency>,
        timestamp: &NaiveDateTime,
    ) -> Result<Money, Box<dyn Error>> {
        let (pln, rate) = nbp::convert(&value, timestamp)?;
        Ok(Money {
            original: value,
            pln: pln,
            rate: rate,
        })
    }
}
