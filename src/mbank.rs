use crate::activity;
use crate::currency;
use chrono::NaiveDateTime;
use csv::{ReaderBuilder, WriterBuilder};
use derive_more::Display;
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
    quantity: u32,
    #[serde(rename(deserialize = "Kurs"), deserialize_with = "from_float")]
    price: f64,
    #[serde(rename(deserialize = "Waluta"))]
    currency: Currency,
    #[serde(
        rename(deserialize = "Czas transakcji"),
        deserialize_with = "from_timestamp",
        serialize_with = "to_timestamp"
    )]
    timestamp: NaiveDateTime,
    #[serde(rename(deserialize = "Prowizja"), deserialize_with = "from_float")]
    commision: f64,
    #[serde(rename(deserialize = "Waluta rozliczenia"))]
    commision_currency: Currency,
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
        _ => Err(de::Error::custom("")),
    }
}

fn from_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let price: &str = Deserialize::deserialize(deserializer)?;
    match price.trim().replace(",", ".").parse() {
        Ok(price) => Ok(price),
        _ => Err(de::Error::custom("")),
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
        _ => Err(de::Error::custom("")),
    }
}

fn to_timestamp<S>(timestamp: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = timestamp.format("%d-%m-%Y").to_string();
    s.serialize_str(&date)
}

fn into_currency(currency: &Currency, value: f64) -> Box<dyn currency::Currency> {
    match currency {
        Currency::PLN => Box::new(currency::Pln(value)),
        Currency::USD => Box::new(currency::Usd(value)),
        Currency::GBP => Box::new(currency::Gbp(value)),
        Currency::EUR => Box::new(currency::Eur(value)),
    }
}

impl Into<activity::Activity> for Transaction {
    fn into(self) -> activity::Activity {
        activity::Activity {
            symbol: self.symbol,
            timestamp: self.timestamp,
            operation: match self.operation {
                Operation::Buy => activity::Operation::Buy {
                    quantity: self.quantity.into(),
                    price: into_currency(&self.currency, self.price),
                    commision: into_currency(&self.commision_currency, self.commision),
                },
                Operation::Sell => activity::Operation::Sell {
                    quantity: self.quantity.into(),
                    price: into_currency(&self.currency, self.price),
                    commision: into_currency(&self.commision_currency, self.commision),
                },
            },
        }
    }
}

pub fn convert(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let mut writer = WriterBuilder::new().has_headers(true).from_writer(vec![]);

    for record in reader.deserialize() {
        let record: Transaction = record?;
        writer.serialize(record)?;
    }
    Ok(String::from_utf8(writer.into_inner()?)?)
}
