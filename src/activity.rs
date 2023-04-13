use crate::currency::{Currency, Eur, Gbp, Pln, Usd};
use chrono::naive::serde::ts_nanoseconds;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    Buy {
        quantity: Decimal,
        #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
        price: Box<dyn Currency>,
        #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
        commision: Box<dyn Currency>,
    },
    Sell {
        quantity: Decimal,
        #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
        price: Box<dyn Currency>,
        #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
        commision: Box<dyn Currency>,
    },
    Dividend {
        #[serde(deserialize_with = "from_currency", serialize_with = "to_currency")]
        value: Box<dyn Currency>,
    },
}

fn from_currency<'de, D>(deserializer: D) -> Result<Box<dyn Currency>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let (amount, currency) = match value.split_once(' ') {
        Some((amount, currency)) => (amount, currency),
        _ => return Err(de::Error::custom("")),
    };
    let amount = match Decimal::from_str_exact(amount) {
        Ok(amount) => amount,
        _ => return Err(de::Error::custom("")),
    };

    match currency {
        "PLN" => Ok(Box::new(Pln(amount))),
        "USD" => Ok(Box::new(Usd(amount))),
        "GBP" => Ok(Box::new(Gbp(amount))),
        "EUR" => Ok(Box::new(Eur(amount))),
        _ => Err(de::Error::custom("")),
    }
}

fn to_currency<S>(currency: &Box<dyn Currency>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&currency.to_string())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub symbol: String,
    #[serde(with = "ts_nanoseconds")]
    pub timestamp: NaiveDateTime,
    pub operation: Operation,
}
