use crate::currency::{Currency, Pln};
use chrono::naive::{Days, NaiveDateTime};
use chrono::{NaiveDate, NaiveTime};
use derive_more::{Display, Error};
use reqwest::{blocking::Client, StatusCode};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer};
use std::error;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Rate {
    #[serde(rename(deserialize = "mid"))]
    value: Decimal,
    #[serde(rename(deserialize = "effectiveDate"), deserialize_with = "from_date")]
    date: NaiveDateTime,
    #[serde(rename(deserialize = "no"))]
    id: String,
}

#[derive(Debug, Deserialize)]
struct Rates {
    #[serde(rename(deserialize = "rates"))]
    values: Vec<Rate>,
}

#[derive(Display, Error, Debug)]
pub struct Error {
    reason: String,
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let date = match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        Ok(date) => date,
        _ => {
            return Err(de::Error::custom(format!(
                "Failed to parse date: {}",
                value
            )))
        }
    };
    let time = match NaiveTime::from_hms_opt(0, 0, 0) {
        Some(time) => time,
        _ => return Err(de::Error::custom(format!("Failed to create empty time"))),
    };
    Ok(NaiveDateTime::new(date, time))
}

pub fn convert(
    client: &Client,
    amount: &dyn Currency,
    trade_date: &NaiveDateTime,
) -> Result<(Pln, Option<Rate>), Box<dyn error::Error>> {
    let currency_name = amount.get_name();
    if currency_name == Pln::default().get_name() {
        return Ok((Pln(*amount.get_value()), None));
    }

    for days in 1..=10 {
        let date = match trade_date.checked_sub_days(Days::new(days)) {
            Some(date) => date,
            _ => break,
        };

        let request = format!(
            "http://api.nbp.pl/api/exchangerates/rates/a/{currency_name}/{date}/?format=json",
            currency_name = currency_name,
            date = date.format("%Y-%m-%d").to_string()
        );

        let reply = client.get(request).send()?;
        if reply.status() != StatusCode::OK {
            continue;
        }

        let rates: Rates = reply.json()?;
        let rate = match rates.values.first() {
            Some(rate) => rate,
            _ => {
                return Err(Box::new(Error {
                    reason: "No rate".to_string(),
                }))
            }
        };
        let value = (amount.get_value() * rate.value).round_dp(2);
        return Ok((Pln(value), Some(rate.clone())));
    }

    Err(Box::new(Error {
        reason: "Failed to find rate for trade date".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::{Usd, Eur};
    use rust_decimal_macros::dec;

    #[test]
    fn test_usd_day_off() {
        let client = Client::new();
        let date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let rate_date = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();
        let rate_time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let rate_timestamp = NaiveDateTime::new(rate_date, rate_time);
        let amount = Usd(dec!(20));
        let (pln, rate) = convert(&client, &amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(75.95)));
        assert_eq!(
            rate,
            Some(Rate {
                value: dec!(3.7977),
                date: rate_timestamp,
                id: "251/A/NBP/2019".to_string()
            })
        );
    }

    #[test]
    fn test_eur_business_day() {
        let client = Client::new();
        let date = NaiveDate::from_ymd_opt(2021, 1, 6).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let rate_date = NaiveDate::from_ymd_opt(2021, 1, 5).unwrap();
        let rate_time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let rate_timestamp = NaiveDateTime::new(rate_date, rate_time);
        let amount = Eur(dec!(20));
        let (pln, rate) = convert(&client, &amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(90.89)));
        assert_eq!(
            rate,
            Some(Rate {
                value: dec!(4.5446),
                date: rate_timestamp,
                id: "002/A/NBP/2021".to_string()
            })
        );
    }

    #[test]
    fn test_pln() {
        let client = Client::new();
        let date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let time = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
        let trade_timestamp = NaiveDateTime::new(date, time);
        let amount = Pln(dec!(20));
        let (pln, rate) = convert(&client, &amount, &trade_timestamp).unwrap();
        assert_eq!(pln, Pln(dec!(20)));
        assert_eq!(rate, None);
    }
}
