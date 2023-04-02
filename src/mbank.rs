use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(rename(deserialize = "Walor"), deserialize_with = "from_symbol")]
    symbol: String,
    #[serde(rename(deserialize = "K/S"))]
    operation: Operation,
    #[serde(rename(deserialize = "Liczba"))]
    quantity: u32,
    #[serde(rename(deserialize = "Kurs"), deserialize_with = "from_float")]
    stock_rate: f32,
    #[serde(rename(deserialize = "Waluta"))]
    currency: Currency,
    #[serde(rename(deserialize = "Czas transakcji"), deserialize_with = "from_timestamp", serialize_with = "to_timestamp")]
    timestamp: NaiveDateTime,
    #[serde(rename(deserialize = "Prowizja"), deserialize_with = "from_float")]
    commision: f32,
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

#[derive(Debug, Deserialize, Serialize)]
enum Currency {
    PLN,
    USD,
    GBP,
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

fn from_float<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let stock_rate: &str = Deserialize::deserialize(deserializer)?;
    match stock_rate.trim().replace(",", ".").parse() {
        Ok(stock_rate) => Ok(stock_rate),
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
        err => Err(de::Error::custom("")),
    }
}

fn to_timestamp<S>(timestamp: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = timestamp.format("%d-%m-%Y").to_string();
    s.serialize_str(&date)
}

pub fn convert (path: &Path) -> Result<String, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(vec![]);

    for record in reader.deserialize() {
        let record: Transaction = record?;
        writer.serialize(record)?;
    }
    Ok(String::from_utf8(writer.into_inner()?)?)
}
