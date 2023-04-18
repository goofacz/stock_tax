use crate::activity;
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

fn extract<'de, T>(lines: String) -> Result<Vec<T>, Box<dyn Error>>
where
    T: de::DeserializeOwned,
{
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .flexible(true)
        .from_reader(lines.as_bytes());

    Ok(reader.deserialize::<T>().collect::<Result<Vec<_>, _>>()?)
}

impl TryInto<activity::Activity> for Transaction {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<activity::Activity, Self::Error> {
        Ok(activity::Activity {
            symbol: self.symbol,
            timestamp: self.timestamp,
            operation: match self.quantity.is_sign_positive() {
                true => activity::Operation::Buy {
                    quantity: self.quantity,
                    price: activity::Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commision: activity::Money::new(
                        currency::new(&self.currency, self.commision.abs().round_dp(2)),
                        &self.timestamp,
                    )?,
                },
                false => activity::Operation::Sell {
                    quantity: self.quantity.abs(),
                    price: activity::Money::new(
                        currency::new(&self.currency, self.price.round_dp(2)),
                        &self.timestamp,
                    )?,
                    commision: activity::Money::new(
                        currency::new(&self.currency, self.commision.abs().round_dp(2)),
                        &self.timestamp,
                    )?,
                },
            },
        })
    }
}

impl TryInto<activity::Activity> for Dividend {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<activity::Activity, Self::Error> {
        let timestamp =
            NaiveDateTime::new(self.timestamp, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        Ok(activity::Activity {
            symbol: self.symbol,
            timestamp: timestamp,
            operation: activity::Operation::Dividend {
                value: activity::Money::new(currency::new(&self.currency, self.value), &timestamp)?,
            },
        })
    }
}

pub fn convert(path: &Path) -> Result<Vec<activity::Activity>, Box<dyn Error>> {
    let handle = File::open(path)?;
    let reader = BufReader::new(handle);
    let lines: Vec<_> = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let mut activities = vec![];

    let transactions = lines
        .iter()
        .filter(|line| {
            let header = "Trades,Header,DataDiscriminator,Asset Category,Currency,Symbol,Date/Time,Quantity,T. Price,C. Price,Proceeds,Comm/Fee,Basis,Realized P/L,MTM P/L,Code";
            let prefix = "Trades,Data,Order,Stocks";
            line.starts_with(header) || line.starts_with(prefix)
        })
        .collect::<Vec<_>>()
        .iter()
        .fold(String::new(),|buf, &line|{ buf + line + "\n"});

    activities.append(
        &mut extract::<Transaction>(transactions)?
            .into_iter()
            .map(|entry| entry.try_into())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Vec<activity::Activity>>(),
    );

    let dividends = lines
        .iter()
        .filter(|line| {
            let header = "Dividends,Header,Currency,Date,Description,Amount";
            let prefix = "Dividends,Data,";
            let summary_prefix = "Dividends,Data,Total";
            (line.starts_with(header) || line.starts_with(prefix))
                && !line.starts_with(summary_prefix)
        })
        .collect::<Vec<_>>()
        .iter()
        .fold(String::new(), |buf, &line| buf + line + "\n");

    activities.append(
        &mut extract::<Dividend>(dividends)?
            .into_iter()
            .map(|entry| entry.try_into())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect::<Vec<activity::Activity>>(),
    );

    Ok(activities)
}
