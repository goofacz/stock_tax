use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::Path;
use std::vec::Vec;

mod currency;
mod dividend;
mod interactive_brokers;
mod mbank;
mod nbp;
mod transaction;

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
            match &args.source {
                ConvertSource::Mbank => {
                    let result = mbank::convert(&path).unwrap();
                    println!("{}", result);
                }
                ConvertSource::InteractiveBrokers => {
                    let result = interactive_brokers::convert(&path).unwrap();
                    println!("{}", result);
                }
            }
        }
    }
}
