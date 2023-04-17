use crate::currency::{Currency, Pln};
use chrono::naive::{Days, NaiveDateTime};
use chrono::{NaiveDate, NaiveTime};
use derive_more::{Display, Error};
use lazy_static::lazy_static;
use reqwest::{blocking::Client, StatusCode};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Rate {
    value: Decimal,
    timestamp: NaiveDateTime,
    id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Entry {
    #[serde(rename(deserialize = "mid"))]
    value: Decimal,
    #[serde(rename(deserialize = "effectiveDate"), deserialize_with = "from_date")]
    timestamp: NaiveDateTime,
    #[serde(rename(deserialize = "no"))]
    id: String,
}

#[derive(Debug, Deserialize)]
struct Entries {
    #[serde(rename(deserialize = "rates"))]
    values: Vec<Entry>,
}

#[derive(Display, Error, Debug)]
pub struct Error {
    reason: String,
}

impl Error {
    fn new(reason: &str) -> Error {
        Error {
            reason: reason.to_string(),
        }
    }
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| de::Error::custom(format!("Failed to parse date: {}", value)))?;
    let time = NaiveTime::from_hms_opt(0, 0, 0)
        .ok_or(de::Error::custom(format!("Failed to create empty time")))?;
    Ok(NaiveDateTime::new(date, time))
}

impl Into<Rate> for Entry {
    fn into(self) -> Rate {
        Rate {
            value: self.value,
            timestamp: self.timestamp,
            id: self.id,
        }
    }
}

pub fn convert(
    amount: &Box<dyn Currency>,
    trade_date: &NaiveDateTime,
) -> Result<(Pln, Option<Rate>), Box<dyn error::Error>> {
    lazy_static! {
        static ref CLIENT: Client = Client::new();
    }

    let currency_name = amount.get_name();
    if currency_name == Pln::default().get_name() {
        return Ok((Pln(*amount.get_value()), None));
    }

    for days in 1..=10 {
        let date = trade_date
            .checked_sub_days(Days::new(days))
            .ok_or(Error::new("Failed to decrement date"))?;

        let request = format!(
            "http://api.nbp.pl/api/exchangerates/rates/a/{currency_name}/{date}/?format=json",
            currency_name = currency_name,
            date = date.format("%Y-%m-%d").to_string()
        );

        let reply = CLIENT.get(request).send()?;
        match reply.status() {
            StatusCode::NOT_FOUND => {
                continue;
            }
            StatusCode::OK => {
                let mut entries: Entries = reply.json()?;
                let entry = entries.values.pop().ok_or(Error::new("No entries"))?;
                let value = (amount.get_value() * entry.value).round_dp(2);
                return Ok((Pln(value), Some(entry.into())));
            }
            _ => {
                return Err(Box::new(Error::new("GET request failed")));
            }
        }
    }

    Err(Box::new(Error::new("Failed to find rate for trade date")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::{Eur, Usd};
    use rust_decimal_macros::dec;

    #[test]
    fn test_usd_day_off() {
        let date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let rate_date = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();
        let rate_time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let rate_timestamp = NaiveDateTime::new(rate_date, rate_time);
        let amount: Box<dyn Currency> = Box::new(Usd(dec!(20)));
        let (pln, rate) = convert(&amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(75.95)));
        assert_eq!(
            rate,
            Some(Rate {
                value: dec!(3.7977),
                timestamp: rate_timestamp,
                id: "251/A/NBP/2019".to_string()
            })
        );
    }

    #[test]
    fn test_eur_business_day() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 6).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let rate_date = NaiveDate::from_ymd_opt(2021, 1, 5).unwrap();
        let rate_time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let rate_timestamp = NaiveDateTime::new(rate_date, rate_time);
        let amount: Box<dyn Currency> = Box::new(Eur(dec!(20)));
        let (pln, rate) = convert(&amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(90.89)));
        assert_eq!(
            rate,
            Some(Rate {
                value: dec!(4.5446),
                timestamp: rate_timestamp,
                id: "002/A/NBP/2021".to_string()
            })
        );
    }

    #[test]
    fn test_pln() {
        let date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let amount: Box<dyn Currency> = Box::new(Pln(dec!(20)));
        let (pln, rate) = convert(&amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(20)));
        assert_eq!(rate, None);
    }
}
