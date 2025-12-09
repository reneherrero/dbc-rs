//! CLI command definitions and handlers.
//!
//! This module defines the CLI structure and command implementations.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::storage;

/// Main CLI structure
#[derive(Parser)]
#[command(name = "dbc-cli")]
#[command(about = "Command-line interface for DBC file manipulation", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Parse and store a DBC file
    Parse {
        /// Path to the DBC file to parse and store
        file: PathBuf,
    },
    /// Print the stored DBC file
    Print,
    /// Describe the contents of the stored DBC file
    Describe,
    /// Decode a CAN message from candump format
    Decode {
        /// CAN message in short candump format (e.g., "1F334455#11223344")
        input: String,
    },
    /// Clear the stored DBC file
    Clear,
}

/// Execute a CLI command.
///
/// # Arguments
///
/// * `command` - The command to execute
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the command fails.
pub fn execute_command(command: Option<Commands>) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Some(Commands::Version) | None => {
            println!("dbc-rs library version {}", dbc_rs::PKG_VERSION);
            Ok(())
        }
        Some(Commands::Parse { file }) => storage::parse_and_store(&file),
        Some(Commands::Print) => storage::get_stored(),
        Some(Commands::Describe) => storage::describe_stored(),
        Some(Commands::Decode { input }) => storage::decode_message(&input),
        Some(Commands::Clear) => storage::clear_stored(),
    }
}
