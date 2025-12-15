use super::parse_state::ParseState;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_val(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume VAL_ keyword
        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
        // Parse VAL_ statement: VAL_ message_id signal_name value1 "desc1" value2 "desc2" ... ;
        // Note: message_id of -1 (0xFFFFFFFF) means the value descriptions apply to
        // all signals with this name in ANY message (global value descriptions)
        parser.skip_newlines_and_spaces();
        let message_id = match parser.parse_i64() {
            Ok(id) => {
                // -1 (0xFFFFFFFF) is the magic number for global value descriptions
                if id == -1 {
                    None
                } else if id >= 0 && id <= u32::MAX as i64 {
                    Some(id as u32)
                } else {
                    parser.skip_to_end_of_line();
                    return Ok(());
                }
            }
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();
        let signal_name = match parser.parse_identifier() {
            Ok(name) => name.to_string(),
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        // Parse value-description pairs
        let mut entries: std::vec::Vec<(u64, std::string::String)> = std::vec::Vec::new();
        loop {
            parser.skip_newlines_and_spaces();
            // Check for semicolon (end of VAL_ statement)
            if parser.starts_with(b";") {
                parser.expect(b";").ok();
                break;
            }
            // Parse value (as i64 first to handle negative values like -1, then convert to u64)
            // Note: -1 (0xFFFFFFFF) is the magic number for global value descriptions in message_id,
            // but values in VAL_ can also be negative
            let value = match parser.parse_i64() {
                Ok(v) => {
                    // Handle -1 specially: convert to 0xFFFFFFFF (u32::MAX) instead of large u64
                    if v == -1 { 0xFFFF_FFFFu64 } else { v as u64 }
                }
                Err(_) => {
                    parser.skip_to_end_of_line();
                    break;
                }
            };
            parser.skip_newlines_and_spaces();
            // Parse description string (expect quote, then take until quote)
            if parser.expect(b"\"").is_err() {
                parser.skip_to_end_of_line();
                break;
            }
            let description_bytes = match parser.take_until_quote(false, 1024) {
                Ok(bytes) => bytes,
                Err(_) => {
                    parser.skip_to_end_of_line();
                    break;
                }
            };
            let description = match core::str::from_utf8(description_bytes) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    parser.skip_to_end_of_line();
                    break;
                }
            };
            entries.push((value, description));
        }
        if !entries.is_empty() {
            state.value_descriptions_buffer.push((message_id, signal_name, entries));
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume VAL_ keyword and skip the rest
        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
