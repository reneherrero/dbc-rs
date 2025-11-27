use super::lang;
use alloc::{format, string::String, vec::Vec};
use core::option::Option::Some;

/// Message formatting functions.
///
/// These functions format error messages using the selected language constants
/// from the `lang` module. The language is selected at build time via feature flags.
// Re-export constants from the selected language module
pub(crate) use lang::{
    DBC_EMPTY_FILE, DBC_NODES_NOT_DEFINED, DBC_NODES_REQUIRED, DBC_TOO_MANY_MESSAGES,
    DBC_VERSION_REQUIRED, MESSAGE_DLC_REQUIRED, MESSAGE_DLC_TOO_LARGE, MESSAGE_DLC_TOO_SMALL,
    MESSAGE_ID_REQUIRED, MESSAGE_INVALID_DLC, MESSAGE_INVALID_FORMAT, MESSAGE_INVALID_ID,
    MESSAGE_NAME_EMPTY, MESSAGE_SENDER_EMPTY, MESSAGE_TOO_MANY_SIGNALS, NODES_TOO_MANY,
    SIGNAL_LENGTH_REQUIRED, SIGNAL_LENGTH_TOO_LARGE, SIGNAL_LENGTH_TOO_SMALL, SIGNAL_NAME_EMPTY,
    SIGNAL_PARSE_EXPECTED_AT, SIGNAL_PARSE_EXPECTED_PIPE, SIGNAL_PARSE_EXPECTED_SG,
    SIGNAL_PARSE_EXPECTED_UNIT_QUOTE, SIGNAL_PARSE_INVALID_FACTOR, SIGNAL_PARSE_INVALID_LENGTH,
    SIGNAL_PARSE_INVALID_MAX, SIGNAL_PARSE_INVALID_MIN, SIGNAL_PARSE_INVALID_OFFSET,
    SIGNAL_PARSE_INVALID_START_BIT, SIGNAL_PARSE_MISSING_BYTE_ORDER,
    SIGNAL_PARSE_MISSING_CLOSING_BRACKET, SIGNAL_PARSE_MISSING_CLOSING_PAREN,
    SIGNAL_PARSE_MISSING_COLON, SIGNAL_PARSE_MISSING_COMMA, SIGNAL_PARSE_MISSING_OPENING_BRACKET,
    SIGNAL_PARSE_MISSING_OPENING_PAREN, SIGNAL_PARSE_MISSING_PIPE_IN_RANGE,
    SIGNAL_PARSE_MISSING_POSITION, SIGNAL_PARSE_MISSING_REST, SIGNAL_PARSE_MISSING_SIGN,
    SIGNAL_PARSE_UNIT_TOO_LONG, SIGNAL_RECEIVERS_TOO_MANY, SIGNAL_START_BIT_REQUIRED,
    VERSION_EMPTY, VERSION_INVALID, VERSION_MAJOR_REQUIRED, VERSION_PATCH_REQUIRES_MINOR,
};

// ============================================================================
// Formatting functions
// ============================================================================

/// Format an invalid data error message with category
pub(crate) fn format_invalid_data(details: &str) -> String {
    format!("{}: {}", lang::INVALID_DATA_CATEGORY, details)
}

/// Format a signal error message with category
pub(crate) fn format_signal_error(details: &str) -> String {
    format!("{}: {}", lang::SIGNAL_ERROR_CATEGORY, details)
}

/// Format a message error message with category
pub(crate) fn format_message_error(details: &str) -> String {
    format!("{}: {}", lang::MESSAGE_ERROR_CATEGORY, details)
}

/// Format a DBC error message with category
pub(crate) fn format_dbc_error(details: &str) -> String {
    format!("{}: {}", lang::DBC_ERROR_CATEGORY, details)
}

/// Format a version error message with category
pub(crate) fn format_version_error(details: &str) -> String {
    format!("{}: {}", lang::VERSION_ERROR_CATEGORY, details)
}

/// Format a nodes error message with category
pub(crate) fn format_nodes_error(details: &str) -> String {
    format!("{}: {}", lang::NODES_ERROR_CATEGORY, details)
}

/// Format an error message with line number
pub(crate) fn with_line_number(msg: &str, line_number: usize) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&msg, &line_number];
    replace_placeholders(lang::FORMAT_LINE_NUMBER, &args)
}

// ============================================================================
// Helper functions for formatted messages
// ============================================================================

/// Helper to replace {} placeholders sequentially in a format string
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

/// Format duplicate message ID error
pub(crate) fn duplicate_message_id(id: u32, msg1: &str, msg2: &str) -> String {
    let args: [&dyn core::fmt::Display; 3] = [
        &id,
        &msg1 as &dyn core::fmt::Display,
        &msg2 as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_DUPLICATE_MESSAGE_ID, &args)
}

/// Format duplicate node name error
pub(crate) fn duplicate_node_name(node_name: &str) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&node_name as &dyn core::fmt::Display];
    replace_placeholders(lang::FORMAT_DUPLICATE_NODE_NAME, &args)
}

/// Format sender not in nodes error
pub(crate) fn sender_not_in_nodes(msg_name: &str, sender: &str) -> String {
    let args: [&dyn core::fmt::Display; 2] = [
        &msg_name as &dyn core::fmt::Display,
        &sender as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_SENDER_NOT_IN_NODES, &args)
}

/// Format signal extends beyond message boundary error
pub(crate) fn signal_extends_beyond_message(
    signal_name: &str,
    start_bit: u16,
    length: u16,
    end_bit: u16,
    max_bits: u16,
    dlc: u8,
) -> String {
    let args: [&dyn core::fmt::Display; 6] = [
        &signal_name as &dyn core::fmt::Display,
        &start_bit,
        &length,
        &end_bit,
        &max_bits,
        &dlc,
    ];
    replace_placeholders(lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE, &args)
}

/// Format invalid range error
pub(crate) fn invalid_range(min: f64, max: f64) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&min, &max];
    replace_placeholders(lang::FORMAT_INVALID_RANGE, &args)
}

/// Format unknown byte order error
pub(crate) fn unknown_byte_order(bo: char) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&bo];
    replace_placeholders(lang::FORMAT_UNKNOWN_BYTE_ORDER, &args)
}

/// Format unknown sign error
pub(crate) fn unknown_sign(sign: char) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&sign];
    replace_placeholders(lang::FORMAT_UNKNOWN_SIGN, &args)
}

/// Format parse number error
pub(crate) fn parse_number_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_PARSE_NUMBER_FAILED, &args)
}

/// Format invalid UTF-8 error
pub(crate) fn invalid_utf8(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_INVALID_UTF8, &args)
}

/// Format read failed error
/// Only available when std feature is enabled (used by `Dbc::from_reader`)
#[cfg(feature = "std")]
pub(crate) fn read_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_READ_FAILED, &args)
}

/// Format message ID out of range error
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

/// Format a number string with comma separators for thousands
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

/// Format signal overlap error
pub(crate) fn signal_overlap(signal1: &str, signal2: &str, message: &str) -> String {
    let args: [&dyn core::fmt::Display; 3] = [
        &signal1 as &dyn core::fmt::Display,
        &signal2 as &dyn core::fmt::Display,
        &message as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_SIGNAL_OVERLAP, &args)
}
