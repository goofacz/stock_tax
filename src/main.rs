use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::Path;
use std::vec::Vec;

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
    #[arg(short, long)]
    symbols: Option<Vec<String>>,
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
            println!("{:?}", args.symbols);
            let activities = match &args.source {
                ConvertSource::Mbank => mbank::convert(&path).unwrap(),
                ConvertSource::InteractiveBrokers => interactive_brokers::convert(&path).unwrap(),
            };
            println!("{:?}", activities);
        }
    }
}
