use std::error::Error;
use chrono::NaiveDateTime;
use reqwest;
use serde::{de, Deserialize, Deserializer};
use crate::transaction::Currency;

#[derive(Deserialize)]
pub struct CurrencyRate {
    no: String,
    mid: f32
}

pub fn lookup_currency_rate(currecny: Currency, timestamp: NaiveDateTime) -> Result<CurrencyRate, Box<dyn Error>> {
    let currecny: String = currecny.to_string();
    let date = timestamp.format("%Y-%m-%d").to_string();
    let request = format!("http://api.nbp.pl/api/exchangerates/rates/a/{currecny}/{date}/?format=json", currecny=currecny, date=date);
    let reply = reqwest::blocking::get(request)?;
    Ok(reply.json()?)
}
