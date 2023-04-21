#![feature(slice_group_by)]
use clap::{Parser, Subcommand};

mod activity;
mod compute;
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
    Compute(compute::CommandArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Convert(args) => convert::command(&args),
        Command::Compute(args) => compute::command(&args),
    };

    match result {
        Err(error) => eprintln!("{}", error),
        _ => { /* nop */ }
    }
}
