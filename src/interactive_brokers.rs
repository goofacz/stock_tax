use crate::activity::{Activity, Money, Operation};
use crate::currency;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(rename(deserialize = "Symbol"))]
    symbol: String,
    #[serde(rename(deserialize = "Quantity"))]
    quantity: Decimal,
    #[serde(rename(deserialize = "T. Price"))]
    price: Decimal,
    #[serde(rename(deserialize = "Currency"))]
    currency: currency::Code,
    #[serde(
        rename(deserialize = "Date/Time"),
        deserialize_with = "from_timestamp",
        serialize_with = "to_timestamp"
    )]
    timestamp: NaiveDateTime,
    #[serde(rename(deserialize = "Comm/Fee"))]
    commision: Decimal,
}

#[derive(Debug, Deserialize)]
struct Dividend {
    #[serde(rename(deserialize = "Description"), deserialize_with = "from_symbol")]
    symbol: String,
    #[serde(rename(deserialize = "Amount"))]
    value: Decimal,
    #[serde(rename(deserialize = "Currency"))]
    currency: currency::Code,
    #[serde(rename(deserialize = "Date"), deserialize_with = "from_date")]
    timestamp: NaiveDate,
}

fn from_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let symbol: &str = Deserialize::deserialize(deserializer)?;
    match symbol.split("(").next() {
        Some(symbol) => Ok(symbol.to_string()),
        _ => Err(de::Error::custom("")),
    }
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: &str = Deserialize::deserialize(deserializer)?;
    let format = "%Y-%m-%d";
    match NaiveDate::parse_from_str(timestamp, format) {
        Ok(timestamp) => Ok(timestamp),
        _ => Err(de::Error::custom("")),
    }
}

fn from_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: &str = Deserialize::deserialize(deserializer)?;
    let format = "%Y-%m-%d, %H:%M:%S";
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

impl TryInto<Activity> for Transaction {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Activity, Self::Error> {
        Ok(Activity {
            symbol: self.symbol,
            timestamp: self.timestamp,
            operation: match self.quantity.is_sign_positive() {
                true => Operation::Buy {
                    quantity: self.quantity,
                    price: Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commision: Money::new(
                        currency::new(&self.currency, self.commision.abs().round_dp(2)),
                        &self.timestamp,
                    )?,
                },
                false => Operation::Sell {
                    quantity: self.quantity.abs(),
                    price: Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commision: Money::new(
                        currency::new(&self.currency, self.commision.abs().round_dp(2)),
                        &self.timestamp,
                    )?,
                },
            },
        })
    }
}

impl TryInto<Activity> for Dividend {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Activity, Self::Error> {
        let timestamp =
            NaiveDateTime::new(self.timestamp, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        Ok(Activity {
            symbol: self.symbol,
            timestamp: timestamp,
            operation: Operation::Dividend {
                value: Money::new(currency::new(&self.currency, self.value), &timestamp)?,
            },
        })
    }
}

fn extract<T>(lines: String) -> Result<impl Iterator<Item = T>, Box<dyn Error>>
where
    T: de::DeserializeOwned + TryInto<Activity>,
{
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .flexible(true)
        .from_reader(lines.as_bytes());

    let values: Vec<T> = reader.deserialize::<T>().collect::<Result<Vec<_>, _>>()?;
    Ok(values.into_iter())
}

fn filter_lines<F>(lines: &[String], function: F) -> String
where
    F: FnMut(&&std::string::String) -> bool,
{
    lines
        .iter()
        .filter(function)
        .collect::<Vec<_>>()
        .iter()
        .fold(String::new(), |buf, &line| buf + line + "\n")
}

pub fn convert(path: &Path) -> Result<Vec<Activity>, Box<dyn Error>> {
    let handle = File::open(path)?;
    let reader = BufReader::new(handle);
    let lines: Vec<_> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    let transactions = filter_lines(&lines, |line| {
        let header = "Trades,Header,DataDiscriminator,Asset Category,Currency,Symbol,Date/Time,Quantity,T. Price,C. Price,Proceeds,Comm/Fee,Basis,Realized P/L,MTM P/L,Code";
        let prefix = "Trades,Data,Order,Stocks";
        line.starts_with(header) || line.starts_with(prefix)
    });

    let transactions = extract::<Transaction>(transactions)?
        .map(|entry| entry.try_into())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let dividends = filter_lines(&lines, |line| {
        let header = "Dividends,Header,Currency,Date,Description,Amount";
        let prefix = "Dividends,Data,";
        let summary_prefix = "Dividends,Data,Total";
        (line.starts_with(header) || line.starts_with(prefix)) && !line.starts_with(summary_prefix)
    });

    let dividends = extract::<Dividend>(dividends)?
        .map(|entry| entry.try_into())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    Ok(vec![]
        .into_iter()
        .chain(transactions)
        .chain(dividends)
        .collect::<Vec<_>>())
}
