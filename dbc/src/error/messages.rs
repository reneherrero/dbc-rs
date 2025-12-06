use super::lang;
#[cfg(feature = "alloc")]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "alloc")]
use core::option::Option::Some;

// Re-export constants from the selected language module
#[cfg(feature = "alloc")]
pub(crate) use lang::{
    DBC_VERSION_REQUIRED, MESSAGE_DLC_REQUIRED, MESSAGE_ID_REQUIRED, MESSAGE_NAME_EMPTY,
    MESSAGE_SENDER_EMPTY, NODES_TOO_MANY, SIGNAL_LENGTH_REQUIRED, SIGNAL_NAME_EMPTY,
    SIGNAL_RECEIVERS_TOO_MANY, SIGNAL_START_BIT_REQUIRED, VERSION_EMPTY,
};
#[cfg(not(feature = "alloc"))]
pub(crate) use lang::{NODES_TOO_MANY, SIGNAL_RECEIVERS_TOO_MANY};

// ============================================================================
// Formatting functions
// ============================================================================

#[cfg(feature = "alloc")]
pub(crate) fn format_invalid_data(details: &str) -> String {
    format!("{}: {}", lang::INVALID_DATA_CATEGORY, details)
}

#[cfg(feature = "alloc")]
pub(crate) fn format_signal_error(details: &str) -> String {
    format!("{}: {}", lang::SIGNAL_ERROR_CATEGORY, details)
}

#[cfg(feature = "alloc")]
pub(crate) fn format_message_error(details: &str) -> String {
    format!("{}: {}", lang::MESSAGE_ERROR_CATEGORY, details)
}

#[cfg(feature = "alloc")]
pub(crate) fn format_dbc_error(details: &str) -> String {
    format!("{}: {}", lang::DBC_ERROR_CATEGORY, details)
}

#[cfg(feature = "alloc")]
pub(crate) fn format_version_error(details: &str) -> String {
    format!("{}: {}", lang::VERSION_ERROR_CATEGORY, details)
}

#[cfg(feature = "alloc")]
pub(crate) fn format_nodes_error(details: &str) -> String {
    format!("{}: {}", lang::NODES_ERROR_CATEGORY, details)
}

// ============================================================================
// Helper functions for formatted messages
// ============================================================================

#[cfg(feature = "alloc")]
fn replace_placeholders(fmt: &str, args: &[&dyn core::fmt::Display]) -> String {
    let mut result = String::with_capacity(fmt.len() + args.len() * 10);
    let mut arg_idx = 0;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consume '}'
            if arg_idx < args.len() {
                use alloc::string::ToString;
                result.push_str(&args[arg_idx].to_string());
                arg_idx += 1;
            } else {
                result.push_str("{}");
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(feature = "alloc")]
pub(crate) fn duplicate_message_id(id: u32, msg1: &str, msg2: &str) -> String {
    let args: [&dyn core::fmt::Display; 3] = [
        &id,
        &msg1 as &dyn core::fmt::Display,
        &msg2 as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_DUPLICATE_MESSAGE_ID, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn sender_not_in_nodes(msg_name: &str, sender: &str) -> String {
    let args: [&dyn core::fmt::Display; 2] = [
        &msg_name as &dyn core::fmt::Display,
        &sender as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_SENDER_NOT_IN_NODES, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn signal_extends_beyond_message(
    signal_name: &str,
    start_bit: u16,
    length: u16,
    end_bit: u16,
    max_bits: u16,
    dlc: u8,
) -> String {
    // Calculate minimum DLC needed (round up to next byte boundary)
    let required_bits = end_bit + 1; // +1 because bits are 0-indexed
    let required_bytes = required_bits.div_ceil(8); // Round up to bytes
    let min_dlc = required_bytes.min(64) as u8; // Cap at CAN FD maximum

    // Generate suggestion based on current DLC and required DLC
    let suggestion = if dlc <= 8 && min_dlc > 8 {
        // Classic CAN (DLC <= 8) but needs CAN FD (DLC > 8)
        replace_placeholders(lang::SUGGEST_CAN_FD, &[&min_dlc as &dyn core::fmt::Display])
    } else if min_dlc > dlc && min_dlc <= 64 {
        // CAN FD but DLC needs to be increased
        replace_placeholders(
            lang::SUGGEST_INCREASE_DLC,
            &[&min_dlc as &dyn core::fmt::Display],
        )
    } else {
        // Signal exceeds even CAN FD maximum (64 bytes = 512 bits)
        "Signal exceeds CAN FD maximum (64 bytes = 512 bits)".to_string()
    };

    let args: [&dyn core::fmt::Display; 7] = [
        &signal_name as &dyn core::fmt::Display,
        &start_bit,
        &length,
        &end_bit,
        &max_bits,
        &dlc,
        &suggestion,
    ];
    replace_placeholders(lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn parse_number_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_PARSE_NUMBER_FAILED, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn invalid_utf8(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_INVALID_UTF8, &args)
}

#[cfg(feature = "std")]
pub(crate) fn read_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_READ_FAILED, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn message_id_out_of_range(id: u32) -> String {
    // Format ID in hex with underscores for readability (e.g., 0x1234_5678)
    let hex_str = format!("{id:08X}");
    let hex_id = if hex_str.len() == 8 {
        format!("0x{}_{}", &hex_str[..4], &hex_str[4..])
    } else {
        format!("0x{hex_str}")
    };

    // Format ID in decimal with commas for readability (no_std compatible)
    let decimal_str = format!("{id}");
    let decimal_id = format_number_with_commas(&decimal_str);

    let args: [&dyn core::fmt::Display; 2] = [&hex_id as &dyn core::fmt::Display, &decimal_id];
    replace_placeholders(lang::FORMAT_MESSAGE_ID_OUT_OF_RANGE, &args)
}

#[cfg(feature = "alloc")]
fn format_number_with_commas(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        // Add comma before every group of 3 digits from the right (but not at the start)
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result
}

#[cfg(feature = "alloc")]
pub(crate) fn signal_overlap(signal1: &str, signal2: &str, message: &str) -> String {
    let suggestion = lang::SUGGEST_MULTIPLEXING;
    let args: [&dyn core::fmt::Display; 4] = [
        &signal1 as &dyn core::fmt::Display,
        &signal2 as &dyn core::fmt::Display,
        &message as &dyn core::fmt::Display,
        &suggestion,
    ];
    replace_placeholders(lang::FORMAT_SIGNAL_OVERLAP, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn message_dlc_too_small(msg_name: &str, msg_id: u32, dlc: u8) -> String {
    let args: [&dyn core::fmt::Display; 3] = [&msg_name as &dyn core::fmt::Display, &msg_id, &dlc];
    replace_placeholders(lang::FORMAT_MESSAGE_DLC_TOO_SMALL, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn message_dlc_too_large(msg_name: &str, msg_id: u32, dlc: u8) -> String {
    let args: [&dyn core::fmt::Display; 3] = [&msg_name as &dyn core::fmt::Display, &msg_id, &dlc];
    replace_placeholders(lang::FORMAT_MESSAGE_DLC_TOO_LARGE, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn signal_length_too_small(signal_name: &str, length: u16) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&signal_name as &dyn core::fmt::Display, &length];
    replace_placeholders(lang::FORMAT_SIGNAL_LENGTH_TOO_SMALL, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn signal_length_too_large(signal_name: &str, length: u16) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&signal_name as &dyn core::fmt::Display, &length];
    replace_placeholders(lang::FORMAT_SIGNAL_LENGTH_TOO_LARGE, &args)
}

#[cfg(feature = "alloc")]
pub(crate) fn signal_start_bit_invalid(signal_name: &str, start_bit: u16) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&signal_name as &dyn core::fmt::Display, &start_bit];
    replace_placeholders(lang::FORMAT_SIGNAL_PARSE_INVALID_START_BIT, &args)
}
