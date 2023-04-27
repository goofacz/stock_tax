use crate::currency::{Currency, Eur, Gbp, Pln, Rates, Usd};
use crate::nbp::Rate;
use chrono::naive::serde::ts_seconds;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
pub struct RateDate {
    #[serde(with = "ts_seconds")]
    pub value: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Money {
    #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
    pub original: Box<dyn Currency>,
    pub pln: Pln,
    pub rate: Option<Rate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    Buy {
        quantity: Decimal,
        price: Money,
        commission: Money,
    },
    Sell {
        quantity: Decimal,
        price: Money,
        commission: Money,
    },
    Dividend {
        value: Money,
        withholding_tax: Money,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub symbol: String,
    #[serde(with = "ts_seconds")]
    pub timestamp: NaiveDateTime,
    pub operation: Operation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Document {
    pub activities: Vec<Activity>,
    pub rates: Rates,
}

fn from_currency<'de, D>(deserializer: D) -> Result<Box<dyn Currency>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
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
