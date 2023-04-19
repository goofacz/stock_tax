use crate::activity::{Activity, Operation};
use crate::currency::Pln;
use chrono::{Datelike, NaiveDateTime};
use clap::Args;
use derive_more::{Display, Error};
use glob::glob;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fs::OpenOptions;
use std::io::BufReader;

#[derive(Display, Error, Debug)]
pub struct Error {
    reason: String,
}

#[derive(Args)]
pub struct CommandArgs {
    path: String,
}

#[derive(Debug)]
struct Dividend {
    timestamp: NaiveDateTime,
    value: Pln,
}

#[derive(Debug)]
struct Share {
    timestamp: NaiveDateTime,
    quantity: Decimal,
    value: Pln,
    commision: Pln,
}

#[derive(Debug, Default)]
struct Stock {
    shares: Vec<Share>,
    dividends: Vec<Dividend>,
}

#[derive(Debug, Default)]
struct Account {
    stocks: HashMap<String, Stock>,
}

impl Account {
    fn get_stock(&mut self, symbol: &str) -> &mut Stock {
        self.stocks.entry(symbol.to_string()).or_default()
    }

    fn dividend(&mut self, symbol: &str, timestamp: &NaiveDateTime, value: Pln) {
        let stock = self.get_stock(symbol);
        stock.dividends.push(Dividend {
            timestamp: *timestamp,
            value: value,
        });
    }

    fn buy(
        &mut self,
        symbol: &str,
        timestamp: &NaiveDateTime,
        quantity: Decimal,
        value: Pln,
        commision: Pln,
    ) {
        let stock = self.get_stock(symbol);
        stock.shares.push(Share {
            timestamp: *timestamp,
            quantity: quantity,
            value: value,
            commision: commision,
        });
    }

    fn calculate_taxes(mut self) {
        let mut years: Vec<_> = self
            .stocks
            .values()
            .map(|stock| {
                let mut dividends_years = stock
                    .dividends
                    .iter()
                    .map(|dividend| dividend.timestamp.year())
                    .collect::<Vec<_>>();
                dividends_years.dedup();
                dividends_years
            })
            .flatten()
            .collect::<Vec<_>>();
        years.dedup();
        println!(">>>>>\n{:#?}", years);
    }
}

fn load_activities(path: &String) -> Result<Vec<Activity>, Box<dyn error::Error>> {
    let mut activities: Vec<Activity> = vec![];
    for file_path in glob(path)? {
        let file_path = file_path?;
        let file = OpenOptions::new().read(true).open(file_path)?;
        let reader = BufReader::new(file);
        activities.append(&mut serde_json::from_reader(reader)?);
    }
    activities.sort_by_key(|activity| activity.timestamp);
    Ok(activities)
}

pub fn command(args: &CommandArgs) -> Result<(), Box<dyn error::Error>> {
    let mut account = Account::default();
    let activities = load_activities(&args.path)?;

    for activity in activities {
        match &activity.operation {
            Operation::Dividend { value } => {
                account.dividend(&activity.symbol, &activity.timestamp, value.pln);
            }
            Operation::Buy {
                quantity,
                price,
                commision,
            } => {
                account.buy(
                    &activity.symbol,
                    &activity.timestamp,
                    *quantity,
                    price.pln,
                    commision.pln,
                );
            }
            _ => {}
        }
    }
    println!("{:#?}", account);
    account.calculate_taxes();
    Ok(())
}
