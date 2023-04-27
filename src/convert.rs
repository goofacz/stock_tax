use crate::activity::{Activity, Operation};
use crate::interactive_brokers;
use crate::mbank;
use crate::nbp;
use clap::{Args, ValueEnum};
use derive_more::{Display, Error};

use serde_json;
use std::error;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Display, Error, Debug)]
pub struct Error {
    reason: String,
}

#[derive(Args)]
pub struct CommandArgs {
    source: ConvertSource,
    path: String,
}

#[derive(Display, Clone, ValueEnum)]
enum ConvertSource {
    Mbank,
    InteractiveBrokers,
}

impl Error {
    fn new(reason: &str) -> Error {
        Error {
            reason: reason.to_string(),
        }
    }
}

fn format_date(date: Option<&Activity>) -> Result<String, Box<dyn error::Error>> {
    Ok(date
        .ok_or(Error::new("No activities"))?
        .timestamp
        .format("%Y-%m-%d")
        .to_string())
}

pub fn command(args: &CommandArgs) -> Result<(), Box<dyn error::Error>> {
    let path = Path::new(&args.path);
    let mut activities = match &args.source {
        ConvertSource::Mbank => mbank::convert(&path).unwrap(),
        ConvertSource::InteractiveBrokers => interactive_brokers::convert(&path).unwrap(),
    };

    activities.sort_by_key(|activity| activity.timestamp);

    for activity in &mut activities {
        let transaction_date = activity.timestamp.date();
        match &mut activity.operation {
            Operation::Dividend {
                value,
                withholding_tax,
            } => {
                (value.pln, value.rate) = nbp::convert(&value.original, &transaction_date)?;
                (withholding_tax.pln, withholding_tax.rate) =
                    nbp::convert(&withholding_tax.original, &transaction_date)?;
            }
            Operation::Buy {
                price, commission, ..
            } => {
                (price.pln, price.rate) = nbp::convert(&price.original, &transaction_date)?;
                (commission.pln, commission.rate) =
                    nbp::convert(&commission.original, &transaction_date)?;
            }
            Operation::Sell {
                price, commission, ..
            } => {
                (price.pln, price.rate) = nbp::convert(&price.original, &transaction_date)?;
                (commission.pln, commission.rate) =
                    nbp::convert(&commission.original, &transaction_date)?;
            }
        }
    }

    let begin_date = format_date(activities.first())?;
    let end_date = format_date(activities.last())?;
    let file_name = format!("{}_{}_{}.json", begin_date, end_date, &args.source);
    let handle = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)?;

    serde_json::to_writer_pretty(handle, &activities).map(|_| Ok(()))?
}
