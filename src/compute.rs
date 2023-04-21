use crate::activity::{Activity, Money, Operation};
use crate::currency::Pln;
use chrono::{Datelike, NaiveDateTime};
use clap::Args;
use derive_more::{Display, Error};
use glob::glob;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json;
use std::cmp::min;
use std::collections::HashMap;
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
struct Share {
    quantity: Decimal,
    price: Pln,
    commission: Pln,
}

#[derive(Debug, Default)]
struct Stock {
    shares: HashMap<String, Vec<Share>>,
}

impl Share {
    fn new(quantity: &Decimal, price: &Money, commission: &Money) -> Share {
        Share {
            quantity: *quantity,
            price: price.pln,
            commission: Pln((commission.pln.0 / quantity).round_dp(2)),
        }
    }
}

impl Stock {
    fn get_shares(&mut self, symbol: String) -> &mut Vec<Share> {
        self.shares.entry(symbol).or_default()
    }

    fn buy(
        &mut self,
        timestamp: &NaiveDateTime,
        symbol: &str,
        quantity: &Decimal,
        price: &Money,
        commission: &Money,
    ) {
        println!("{date}: {symbol}: Buy quantity: {quantity} price: {price_org} / {price_pln} commission: {commission_org} / {commission_pln}",
            date=timestamp.date(),
            symbol=symbol,
            quantity=quantity,
            price_org=price.original,
            price_pln=price.pln,
            commission_org=commission.original,
            commission_pln=commission.pln);

        let shares = self.get_shares(symbol.to_string());
        let share = Share::new(quantity, price, commission);
        shares.push(share);
    }

    fn sell(
        &mut self,
        timestamp: &NaiveDateTime,
        symbol: &str,
        quantity: &Decimal,
        price: &Money,
        commission: &Money,
    ) -> Pln {
        let shares = self.get_shares(symbol.to_string());
        let mut remaining_quantity = quantity.clone();
        let sell_cost = (price.pln * *quantity) + commission.pln;
        let mut buy_cost = Pln::default();
        while remaining_quantity > dec!(0) {
            let share = shares.first_mut().unwrap();
            let quantity = min(share.quantity, remaining_quantity);

            share.quantity -= quantity;
            remaining_quantity -= quantity;

            buy_cost += share.price * quantity;
            buy_cost += share.commission * quantity;

            if share.quantity == dec!(0) {
                shares.pop();
            }
        }

        let tax = buy_cost - sell_cost;

        println!("{date}: {symbol}: Sell quantity: {quantity} price: {price_org} / {price_pln} commission: {commission_org} / {commission_pln} tax: {tax}",
            date=timestamp.date(),
            symbol=symbol,
            quantity=quantity,
            price_org=price.original,
            price_pln=price.pln,
            commission_org=commission.original,
            commission_pln=commission.pln,
            tax=tax);

        tax
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
    let mut stock = Stock::default();
    let all_activities = load_activities(&args.path)?;

    for annual_activities in
        all_activities.group_by(|a, b| a.timestamp.year() == b.timestamp.year())
    {
        let mut total_tax = Pln::default();
        let mut total_loss = Pln::default();

        let year = match annual_activities.first() {
            Some(activity) => activity.timestamp.year(),
            None => continue,
        };
        for activity in annual_activities.into_iter() {
            let timestamp = &activity.timestamp;
            let symbol = &activity.symbol;
            match &activity.operation {
                Operation::Dividend { value } => {
                    let tax = Pln((value.pln.0 * dec!(0.04)).round_dp(0));
                    total_tax += tax;
                    println!(
                        "{date}: {symbol}: Dividend value: {original} / {pln}: tax: {tax}",
                        date = timestamp.date(),
                        symbol = symbol,
                        original = value.original,
                        pln = value.pln,
                        tax = tax
                    );
                }
                Operation::Buy {
                    quantity,
                    price,
                    commission,
                } => {
                    stock.buy(timestamp, symbol, quantity, price, commission);
                }
                Operation::Sell {
                    quantity,
                    price,
                    commission,
                } => {
                    let value = stock.sell(timestamp, symbol, quantity, price, commission);
                    if value.0.is_sign_positive() {
                        total_tax += value;
                    } else {
                        total_loss += value;
                    }
                }
            }
        }

        println!("{}: Total tax: {} loss: {}", year, total_tax, total_loss);
    }

    Ok(())
}
