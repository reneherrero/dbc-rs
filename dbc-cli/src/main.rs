//! Command-line interface for DBC file manipulation.
//!
//! This binary provides a CLI tool for parsing, validating, and manipulating
//! DBC (CAN Database) files.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dbc-cli")]
#[command(about = "Command-line interface for DBC file manipulation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("dbc-cli version {}", env!("CARGO_PKG_VERSION"));
            println!("dbc-rs library version {}", dbc_rs::PKG_VERSION);
        }
    }
}
