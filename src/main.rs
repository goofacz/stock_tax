use clap::{Parser, Subcommand};

mod activity;
mod convert;
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
    Convert(convert::CommandArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Convert(args) => convert::command(&args),
    };

    match result {
        Err(error) => eprintln!("{}", error),
        _ => { /* nop */},
    }
}
