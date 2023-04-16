use crate::interactive_brokers;
use crate::mbank;

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

pub fn command(args: &CommandArgs) -> Result<(), Box<dyn error::Error>> {
    let path = Path::new(&args.path);
    let mut activities = match &args.source {
        ConvertSource::Mbank => mbank::convert(&path).unwrap(),
        ConvertSource::InteractiveBrokers => interactive_brokers::convert(&path).unwrap(),
    };

    activities.sort_by_key(|activity| activity.timestamp);

    let begin_date = match activities.first() {
        Some(date) => date,
        _ => {
            return Err(Box::new(Error {
                reason: "No activities".to_string(),
            }))
        }
    };
    let end_date = match activities.last() {
        Some(date) => date,
        _ => {
            return Err(Box::new(Error {
                reason: "No activities".to_string(),
            }))
        }
    };
    let file_name = format!(
        "{}_{}_{}.json",
        begin_date.timestamp.format("%Y-%m-%d").to_string(),
        end_date.timestamp.format("%Y-%m-%d").to_string(),
        &args.source
    );

    let handle = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)
    {
        Ok(handle) => handle,
        Err(error) => return Err(Box::new(error)),
    };

    return match serde_json::to_writer_pretty(handle, &activities) {
        Ok(_) => Ok(()),
        Err(error) => Err(Box::new(error)),
    };
}
