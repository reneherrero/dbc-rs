//! Command-line interface for DBC file manipulation.
//!
//! This binary provides a CLI tool for parsing, validating, and manipulating
//! DBC (CAN Database) files.

mod commands;
mod storage;

use clap::Parser;
use commands::{Cli, execute_command};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
