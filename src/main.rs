use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json;
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

#[derive(Clone, ValueEnum)]
enum ConvertSource {
    Mbank,
    InteractiveBrokers,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Convert(args) => {
            let path = Path::new(&args.path);
            let activities = match &args.source {
                ConvertSource::Mbank => mbank::convert(&path).unwrap(),
                ConvertSource::InteractiveBrokers => interactive_brokers::convert(&path).unwrap(),
            };
            println!("{}", serde_json::to_string(&activities).unwrap());
        }
    }
}
