use crate::currency::{Code, Currency, Pln};
use chrono::naive::Days;
use chrono::NaiveDate;
use derive_more::{Display, Error};
use lazy_static::lazy_static;
use reqwest::{blocking::Client, StatusCode};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};

use std::iter;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Rate {
    value: Decimal,
    date: NaiveDate,
    id: String,
}
#[derive(Debug, Deserialize, Clone)]
struct Entry {
    #[serde(rename(deserialize = "mid"))]
    value: Decimal,
    #[serde(rename(deserialize = "effectiveDate"), deserialize_with = "from_date")]
    date: NaiveDate,
    #[serde(rename(deserialize = "no"))]
    id: String,
}

#[derive(Debug, Deserialize)]
struct Entries {
    #[serde(rename(deserialize = "rates"))]
    values: Vec<Entry>,
}

#[derive(Display, Error, Debug, PartialEq)]
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

pub fn convert(
    amount: &Box<dyn Currency>,
    transaction_date: &NaiveDate,
) -> Result<(Pln, Option<Rate>), Error> {
    let code = amount.get_code();

    lazy_static! {
        static ref CLIENT: Client = Client::new();
    }

    match code {
        Code::PLN => {
            return Ok((Pln(*amount.get_value()), None));
        }
        _ => {
            for date in generate_previous_days(transaction_date) {
                let request = format!(
                    "http://api.nbp.pl/api/exchangerates/rates/a/{name}/{date}/?format=json",
                    name = code.to_string().to_lowercase(),
                    date = date.format("%Y-%m-%d").to_string()
                );

                let reply = CLIENT
                    .get(request)
                    .send()
                    .or(Err(Error::new("Failed to send GET request")))?;
                match reply.status() {
                    StatusCode::OK => {
                        let mut entries: Entries =
                            reply.json().or(Err(Error::new("Failed to parse reply")))?;
                        let entry = entries.values.pop().ok_or(Error::new("No entries"))?;
                        let rate: Rate = entry.into();
                        let value = (amount.get_value() * rate.value).round_dp(2);
                        return Ok((Pln(value), Some(rate)));
                    }
                    StatusCode::NOT_FOUND => {
                        continue;
                    }
                    _ => {
                        return Err(Error::new("GET request failed"));
                    }
                }
            }
            Err(Error::new(&format!(
                "Failed to rate for any date prior to {}",
                transaction_date.format("%d-%m-%Y")
            )))
        }
    }
}

fn generate_previous_days(date: &NaiveDate) -> impl Iterator<Item = NaiveDate> {
    iter::successors(Some(*date), |date| date.checked_sub_days(Days::new(1)))
        .skip(1)
        .take(10)
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| de::Error::custom(format!("Failed to parse date: {}", value)))
}

impl Into<Rate> for Entry {
    fn into(self) -> Rate {
        Rate {
            value: self.value,
            date: self.date,
            id: self.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::{Eur, Usd};
    use rust_decimal_macros::dec;

    #[test]
    fn test_usd_day_off() {
        let trade_date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let amount: Box<dyn Currency> = Box::new(Usd(dec!(20)));

        assert_eq!(
            convert(&amount, &trade_date),
            Ok((
                Pln(dec!(75.95)),
                Some(Rate {
                    value: dec!(3.7977),
                    date: NaiveDate::from_ymd_opt(2019, 12, 31).unwrap(),
                    id: "251/A/NBP/2019".to_string()
                })
            ))
        );
    }

    #[test]
    fn test_eur_business_day() {
        let trade_date = NaiveDate::from_ymd_opt(2021, 1, 6).unwrap();
        let amount: Box<dyn Currency> = Box::new(Eur(dec!(20)));

        assert_eq!(
            convert(&amount, &trade_date),
            Ok((
                Pln(dec!(90.89)),
                Some(Rate {
                    value: dec!(4.5446),
                    date: NaiveDate::from_ymd_opt(2021, 1, 5).unwrap(),
                    id: "002/A/NBP/2021".to_string()
                })
            ))
        );
    }

    #[test]
    fn test_pln() {
        let trade_date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let amount: Box<dyn Currency> = Box::new(Pln(dec!(23)));

        assert_eq!(convert(&amount, &trade_date), Ok((Pln(dec!(23)), None)));
    }
}
