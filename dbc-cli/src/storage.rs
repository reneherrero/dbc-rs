//! Storage and persistence module for DBC files.
//!
//! This module handles storing a single parsed DBC file to disk and retrieving it.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const STORAGE_FILENAME: &str = "stored.dbc";

/// Get the cache directory for storing DBC files.
///
/// Returns the platform-specific cache directory (e.g., `~/.cache/dbc-cli/` on Linux).
pub fn get_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = dirs::cache_dir().ok_or("Could not determine cache directory")?.join("dbc-cli");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Parse a DBC file and store it in the cache.
///
/// # Arguments
///
/// * `file_path` - Path to the DBC file to parse
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if parsing or storage fails.
pub fn parse_and_store(file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read the DBC file
    let content = fs::read_to_string(file_path)?;

    // Parse the DBC file
    let dbc =
        dbc_rs::Dbc::parse(&content).map_err(|e| format!("Failed to parse DBC file: {}", e))?;

    // Validate by converting back to string (this exercises the full validation)
    let dbc_string = dbc.to_dbc_string();

    // Get cache directory
    let cache_dir = get_cache_dir()?;
    let storage_path = cache_dir.join(STORAGE_FILENAME);

    // Store the DBC string
    fs::write(&storage_path, &dbc_string)?;

    println!("✓ Successfully parsed and stored DBC file");

    Ok(())
}

/// Retrieve the stored DBC file and output to stdout.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the file is not found or cannot be read.
pub fn get_stored() -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let storage_path = cache_dir.join(STORAGE_FILENAME);

    if !storage_path.exists() {
        return Err(
            "No stored DBC file found. Use 'parse' command to store a DBC file first.".into(),
        );
    }

    let dbc_content = fs::read_to_string(&storage_path)?;
    io::stdout().write_all(dbc_content.as_bytes())?;

    Ok(())
}

/// Describe the contents of the stored DBC file.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the file is not found or cannot be read.
pub fn describe_stored() -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let storage_path = cache_dir.join(STORAGE_FILENAME);

    if !storage_path.exists() {
        return Err(
            "No stored DBC file found. Use 'parse' command to store a DBC file first.".into(),
        );
    }

    let dbc_content = fs::read_to_string(&storage_path)?;
    let dbc = dbc_rs::Dbc::parse(&dbc_content)
        .map_err(|e| format!("Failed to parse stored DBC file: {}", e))?;

    // Version
    if let Some(version) = dbc.version() {
        println!("Version: {}", version.as_str());
    } else {
        println!("Version: (not specified)");
    }

    // Nodes
    let nodes = dbc.nodes();
    println!("\nNodes: {}", nodes.len());
    if !nodes.is_empty() {
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", node);
        }
        println!();
    }

    // Messages
    let messages = dbc.messages();
    println!("\nMessages: {}", messages.len());
    if !messages.is_empty() {
        for message in messages.iter() {
            println!(
                "  {} (ID: 0x{:X}, DLC: {}, Sender: {}, Signals: {})",
                message.name(),
                message.id(),
                message.dlc(),
                message.sender(),
                message.signals().len()
            );
            // Signals within the message
            for signal in message.signals().iter() {
                let unit_str = signal.unit().map(|u| format!(" {}", u)).unwrap_or_default();

                // Receiver information
                let receivers_str = match signal.receivers() {
                    dbc_rs::Receivers::Nodes(nodes) if !nodes.is_empty() => {
                        let nodes: Vec<String> =
                            signal.receivers().iter().map(|s| s.to_string()).collect();
                        format!(" (receivers: {})", nodes.join(", "))
                    }
                    _ => String::new(), // None or empty nodes list
                };

                println!(
                    "    - {}: start_bit={}, length={}, factor={}, offset={}, range=[{}, {}]{}{}",
                    signal.name(),
                    signal.start_bit(),
                    signal.length(),
                    signal.factor(),
                    signal.offset(),
                    signal.min(),
                    signal.max(),
                    unit_str,
                    receivers_str
                );
            }
        }
    }

    Ok(())
}

/// Decode a CAN message from candump format.
///
/// # Arguments
///
/// * `input` - CAN message in (compact) candump format (e.g., "1F334455#1122334455667788")
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if decoding fails.
pub fn decode_message(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let storage_path = cache_dir.join(STORAGE_FILENAME);

    if !storage_path.exists() {
        return Err(
            "No stored DBC file found. Use 'parse' command to store a DBC file first.".into(),
        );
    }

    let dbc_content = fs::read_to_string(&storage_path)?;
    let dbc = dbc_rs::Dbc::parse(&dbc_content)
        .map_err(|e| format!("Failed to parse stored DBC file: {}", e))?;

    // Parse short candump format: "can_id#data_bytes"
    // Example: "1F334455#1122334455667788"
    let (can_id_str, data_str) = input.split_once('#').ok_or_else(|| {
        "Invalid candump format. Expected: 'can_id#data_bytes' (e.g., '1F334455#11223344')"
            .to_string()
    })?;

    // Parse CAN ID (hex, with or without 0x prefix)
    let can_id_str = can_id_str.trim_start_matches("0x").trim_start_matches("0X");
    let can_id = u32::from_str_radix(can_id_str, 16)
        .map_err(|_| format!("Invalid CAN ID: {}", can_id_str))?;

    // Parse data bytes (hex string, each byte is 2 hex digits)
    let data_str = data_str.trim();
    if data_str.len() % 2 != 0 {
        return Err("Data bytes must be an even number of hex digits".into());
    }

    let dlc = data_str.len() / 2;
    if dlc > 64 {
        return Err("CAN message data cannot exceed 64 bytes (CAN FD maximum)".into());
    }

    let mut data = Vec::with_capacity(dlc);
    for i in 0..dlc {
        let byte_str = &data_str[i * 2..i * 2 + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| format!("Invalid hex byte: {}", byte_str))?;
        data.push(byte);
    }

    // Decode message using high-performance decode method
    let decoded_signals = dbc
        .decode(can_id, &data)
        .map_err(|e| format!("Failed to decode message: {}", e))?;

    // Get message info for header display (message exists if decode succeeded)
    let message = dbc
        .messages()
        .find_by_id(can_id)
        .expect("Message should exist if decode succeeded");
    println!("Message: {} (ID: 0x{:X})", message.name(), can_id);

    // Display decoded signals with units
    for (signal_name, value, unit) in decoded_signals.iter() {
        let unit_str = unit.map(|u| format!(" {}", u)).unwrap_or_default();
        println!("  {}: {}{}", signal_name, value, unit_str);
    }

    Ok(())
}

/// Clear the stored DBC file from the cache.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if clearing fails.
pub fn clear_stored() -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let storage_path = cache_dir.join(STORAGE_FILENAME);

    if !storage_path.exists() {
        println!("No stored DBC file to clear.");
        return Ok(());
    }

    fs::remove_file(&storage_path)?;

    // Try to remove the cache directory if it's empty (ignore errors)
    let _ = fs::remove_dir(&cache_dir);

    println!("✓ Cleared stored DBC file");

    Ok(())
}
