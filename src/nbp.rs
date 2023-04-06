use crate::transaction::Currency;
use chrono::NaiveDate;
use reqwest;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Rate {
    #[serde(rename(deserialize = "mid"))]
    value: f32,
    #[serde(rename(deserialize = "effectiveDate"), deserialize_with = "from_date")]
    date: NaiveDate,
}

#[derive(Debug, Deserialize)]
struct Rates {
    #[serde(rename(deserialize = "rates"))]
    values: Vec<Rate>,
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        Ok(date) => Ok(date),
        _ => Err(de::Error::custom("")),
    }
}

pub fn lookup_currency_rate(
    currency: Currency,
    year: i32,
) -> Result<HashMap<NaiveDate, f32>, Box<dyn Error>> {
    let request = format!("http://api.nbp.pl/api/exchangerates/rates/a/{currency}/{begin_date}/{end_date}/?format=json",
        currency=currency.to_string(),
        begin_date=NaiveDate::from_ymd_opt(year, 1, 1)
            .ok_or("")?,
        end_date=NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .ok_or("")?
            .pred_opt()
            .ok_or("")?);
    let reply = reqwest::blocking::get(request)?;
    let rates: Rates = reply.json()?;
    Ok(HashMap::from_iter(
        rates.values.iter().map(|rate| (rate.date, rate.value)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_2022() {
        let rates = lookup_currency_rate(Currency::USD, 2022);
        assert!(rates.is_ok());

        let rates = rates.unwrap();
        let date = NaiveDate::from_ymd_opt(2022, 8, 2).unwrap();
        assert_eq!(rates[&date], 4.5984);
    }
}
