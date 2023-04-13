use crate::currency;
use chrono::Datelike;
use chrono::NaiveDate;
use reqwest;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Rate {
    #[serde(rename(deserialize = "mid"))]
    value: Decimal,
    #[serde(rename(deserialize = "effectiveDate"), deserialize_with = "from_date")]
    date: NaiveDate,
    #[serde(rename(deserialize = "no"))]
    id: String,
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

pub fn lookup_yearly_rates(
    currency: &String,
    year: i32,
) -> Result<HashMap<NaiveDate, Decimal>, Box<dyn Error>> {
    let request = format!("http://api.nbp.pl/api/exchangerates/rates/a/{currency}/{begin_date}/{end_date}/?format=json",
        currency=currency,
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

pub fn lookup_rates<T: currency::Currency + Default>(
    begin_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<currency::Rate<T>, Box<dyn Error>> {
    let currency_name = T::default().get_name().to_string();
    let mut rates = HashMap::new();
    for year in begin_date.year()..=end_date.year() {
        let yearly_rates = lookup_yearly_rates(&currency_name, year)?;
        rates.extend(yearly_rates);
    }
    Ok(currency::Rate::new(rates.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_lookup_rates() {
        let begin_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 10, 1).unwrap();
        let rates = lookup_rates::<currency::Usd>(begin_date, end_date).unwrap();
        let date = NaiveDate::from_ymd_opt(2022, 8, 2).unwrap();
        assert_eq!(
            rates.convert(&currency::Usd(dec!(1.)), &date).unwrap(),
            currency::Pln(dec!(4.6))
        );
    }
}
