use clap::{Args, Parser, Subcommand, ValueEnum};
use derive_more::Display;
use serde_json;
use std::fs::OpenOptions;
use std::path::Path;

mod activity;
mod currency;
mod interactive_brokers;
mod mbank;
mod nbp;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Convert(ConvertArgs),
}

#[derive(Args)]
struct ConvertArgs {
    source: ConvertSource,
    path: String,
}

#[derive(Display, Clone, ValueEnum)]
enum ConvertSource {
    Mbank,
    InteractiveBrokers,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Convert(args) => {
            let path = Path::new(&args.path);
            let mut activities = match &args.source {
                ConvertSource::Mbank => mbank::convert(&path).unwrap(),
                ConvertSource::InteractiveBrokers => interactive_brokers::convert(&path).unwrap(),
            };

            activities.sort_by_key(|x| x.timestamp);

            let begin_date = activities
                .first()
                .unwrap()
                .timestamp
                .format("%Y-%m-%d")
                .to_string();
            let end_date = activities
                .last()
                .unwrap()
                .timestamp
                .format("%Y-%m-%d")
                .to_string();
            let file_name = format!("{}_{}_{}.json", begin_date, end_date, &args.source);
            let mut handle = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .unwrap();
            serde_json::to_writer_pretty(handle, &activities).unwrap();
        }
    }
}
