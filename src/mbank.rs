use crate::activity::{self, Activity, Money};
use crate::currency;
use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use derive_more::Display;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::Into;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Display)]
pub enum Currency {
    PLN,
    USD,
    GBP,
    EUR,
}

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(rename(deserialize = "Walor"), deserialize_with = "from_symbol")]
    symbol: String,
    #[serde(rename(deserialize = "K/S"))]
    operation: Operation,
    #[serde(rename(deserialize = "Liczba"))]
    quantity: Decimal,
    #[serde(rename(deserialize = "Kurs"), deserialize_with = "from_float")]
    price: Decimal,
    #[serde(rename(deserialize = "Waluta"))]
    currency: currency::Code,
    #[serde(
        rename(deserialize = "Czas transakcji"),
        deserialize_with = "from_timestamp",
        serialize_with = "to_timestamp"
    )]
    timestamp: NaiveDateTime,
    #[serde(rename(deserialize = "Prowizja"), deserialize_with = "from_float")]
    commission: Decimal,
    #[serde(rename(deserialize = "Waluta rozliczenia"))]
    commission_currency: currency::Code,
}

#[derive(Debug, Deserialize, Serialize)]
enum Operation {
    #[serde(rename(deserialize = "K"))]
    Buy,
    #[serde(rename(deserialize = "S"))]
    Sell,
}

fn from_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let symbol: &str = Deserialize::deserialize(deserializer)?;
    match symbol.split(" ").next() {
        Some(symbol) => Ok(symbol.to_string()),
        _ => Err(de::Error::custom(format!("Failed to parse \"{}\"", symbol))),
    }
}

fn from_float<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let price: &str = Deserialize::deserialize(deserializer)?;
    let price = price.trim().replace(",", ".");

    match Decimal::from_str_exact(&price) {
        Ok(price) => Ok(price),
        _ => Err(de::Error::custom(format!("Failed to parse \"{}\"", price))),
    }
}

fn from_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: &str = Deserialize::deserialize(deserializer)?;
    let format = "%d.%m.%Y %H:%M:%S";
    match NaiveDateTime::parse_from_str(timestamp, format) {
        Ok(timestamp) => Ok(timestamp),
        _ => Err(de::Error::custom(format!(
            "Failed to parse \"{}\"",
            timestamp
        ))),
    }
}

fn to_timestamp<S>(timestamp: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = timestamp.format("%d-%m-%Y").to_string();
    s.serialize_str(&date)
}

impl TryInto<Activity> for Transaction {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Activity, Self::Error> {
        Ok(Activity {
            symbol: self.symbol,
            timestamp: self.timestamp,
            operation: match self.operation {
                Operation::Buy => activity::Operation::Buy {
                    quantity: self.quantity.into(),
                    price: Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commission: Money::new(
                        currency::new(&self.commission_currency, self.commission.round_dp(2)),
                        &self.timestamp,
                    )?,
                },
                Operation::Sell => activity::Operation::Sell {
                    quantity: self.quantity.into(),
                    price: Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commission: Money::new(
                        currency::new(&self.commission_currency, self.commission.round_dp(2)),
                        &self.timestamp,
                    )?,
                },
            },
        })
    }
}

pub fn convert(path: &Path) -> Result<Vec<Activity>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let transactions = reader
        .deserialize::<Transaction>()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(transactions
        .into_iter()
        .map(|entry| entry.try_into())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Vec<_>>())
}
