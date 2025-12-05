use super::lang;
use alloc::{format, string::String, vec::Vec};
use core::option::Option::Some;

// Re-export constants from the selected language module
#[cfg(feature = "std")]
pub(crate) use lang::{
    DBC_NODES_REQUIRED, DBC_VERSION_REQUIRED, MESSAGE_DLC_REQUIRED, MESSAGE_ID_REQUIRED,
    MESSAGE_NAME_EMPTY, MESSAGE_SENDER_EMPTY, NODES_TOO_MANY, SIGNAL_LENGTH_REQUIRED,
    SIGNAL_NAME_EMPTY, SIGNAL_RECEIVERS_TOO_MANY, SIGNAL_START_BIT_REQUIRED, VERSION_EMPTY,
};
#[cfg(not(feature = "std"))]
pub(crate) use lang::{NODES_TOO_MANY, SIGNAL_RECEIVERS_TOO_MANY};

// ============================================================================
// Formatting functions
// ============================================================================

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_invalid_data(details: &str) -> String {
    format!("{}: {}", lang::INVALID_DATA_CATEGORY, details)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_signal_error(details: &str) -> String {
    format!("{}: {}", lang::SIGNAL_ERROR_CATEGORY, details)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_message_error(details: &str) -> String {
    format!("{}: {}", lang::MESSAGE_ERROR_CATEGORY, details)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_dbc_error(details: &str) -> String {
    format!("{}: {}", lang::DBC_ERROR_CATEGORY, details)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_version_error(details: &str) -> String {
    format!("{}: {}", lang::VERSION_ERROR_CATEGORY, details)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn format_nodes_error(details: &str) -> String {
    format!("{}: {}", lang::NODES_ERROR_CATEGORY, details)
}

#[allow(dead_code)]
pub(crate) fn with_line_number(msg: &str, line_number: usize) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&msg, &line_number];
    replace_placeholders(lang::FORMAT_LINE_NUMBER, &args)
}

// ============================================================================
// Helper functions for formatted messages
// ============================================================================

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

#[allow(dead_code)]
pub(crate) fn duplicate_message_id(id: u32, msg1: &str, msg2: &str) -> String {
    let args: [&dyn core::fmt::Display; 3] = [
        &id,
        &msg1 as &dyn core::fmt::Display,
        &msg2 as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_DUPLICATE_MESSAGE_ID, &args)
}

#[allow(dead_code)]
pub(crate) fn duplicate_node_name(node_name: &str) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&node_name as &dyn core::fmt::Display];
    replace_placeholders(lang::FORMAT_DUPLICATE_NODE_NAME, &args)
}

#[allow(dead_code)]
pub(crate) fn sender_not_in_nodes(msg_name: &str, sender: &str) -> String {
    let args: [&dyn core::fmt::Display; 2] = [
        &msg_name as &dyn core::fmt::Display,
        &sender as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_SENDER_NOT_IN_NODES, &args)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn invalid_range(min: f64, max: f64) -> String {
    let args: [&dyn core::fmt::Display; 2] = [&min, &max];
    replace_placeholders(lang::FORMAT_INVALID_RANGE, &args)
}

#[allow(dead_code)]
pub(crate) fn unknown_byte_order(bo: char) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&bo];
    replace_placeholders(lang::FORMAT_UNKNOWN_BYTE_ORDER, &args)
}

#[allow(dead_code)]
pub(crate) fn unknown_sign(sign: char) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&sign];
    replace_placeholders(lang::FORMAT_UNKNOWN_SIGN, &args)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn parse_number_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_PARSE_NUMBER_FAILED, &args)
}

#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn invalid_utf8(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_INVALID_UTF8, &args)
}

#[cfg(feature = "std")]
pub(crate) fn read_failed(err: impl core::fmt::Display) -> String {
    let args: [&dyn core::fmt::Display; 1] = [&err];
    replace_placeholders(lang::FORMAT_READ_FAILED, &args)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn signal_overlap(signal1: &str, signal2: &str, message: &str) -> String {
    let args: [&dyn core::fmt::Display; 3] = [
        &signal1 as &dyn core::fmt::Display,
        &signal2 as &dyn core::fmt::Display,
        &message as &dyn core::fmt::Display,
    ];
    replace_placeholders(lang::FORMAT_SIGNAL_OVERLAP, &args)
}
