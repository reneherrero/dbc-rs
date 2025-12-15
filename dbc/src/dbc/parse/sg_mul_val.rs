use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{ExtendedMultiplexing, compat::Vec};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sg_mul_val(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume SG_MUL_VAL_ keyword (peek_next_keyword already positioned us at the keyword)
        let _ = parser.expect(crate::SG_MUL_VAL_.as_bytes()).ok();
        parser.skip_newlines_and_spaces();

        // Parse: SG_MUL_VAL_ message_id signal_name multiplexer_switch value_ranges ;
        // Example: SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
        let message_id = match parser.parse_u32() {
            Ok(id) => id,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        let signal_name_str = match parser.parse_identifier() {
            Ok(name) => name,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        let signal_name = match crate::validate_name(signal_name_str) {
            Ok(name) => name,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        let multiplexer_switch_str = match parser.parse_identifier() {
            Ok(switch) => switch,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        let multiplexer_switch = match crate::validate_name(multiplexer_switch_str) {
            Ok(switch) => switch,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        let mut value_ranges = Vec::<(u64, u64), 64>::new();

        // Parse value ranges (at least one required)
        loop {
            parser.skip_newlines_and_spaces();
            // Parse value range: min-max
            let min_u32 = match parser.parse_u32() {
                Ok(min) => min,
                Err(_) => {
                    parser.skip_to_end_of_line();
                    break;
                }
            };
            let min = min_u32 as u64;
            parser.skip_newlines_and_spaces();
            if parser.expect(b"-").is_err() {
                parser.skip_to_end_of_line();
                break;
            }
            parser.skip_newlines_and_spaces();
            let max_u32 = match parser.parse_u32() {
                Ok(max) => max,
                Err(_) => {
                    parser.skip_to_end_of_line();
                    break;
                }
            };
            let max = max_u32 as u64;

            if value_ranges.push((min, max)).is_err() {
                // Vector full, skip remaining
                parser.skip_to_end_of_line();
                break;
            }

            // Check for comma (more ranges) or semicolon (end)
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b",") {
                parser.expect(b",").ok();
                // Continue to next range
            } else if parser.starts_with(b";") {
                parser.expect(b";").ok();
                break;
            } else {
                // End of ranges (no comma or semicolon)
                break;
            }
        }

        // Only add if we parsed at least one range
        if !value_ranges.is_empty() {
            state.extended_multiplexing_buffer.push(ExtendedMultiplexing::new(
                message_id,
                signal_name,
                multiplexer_switch,
                value_ranges,
            ));
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume SG_MUL_VAL_ keyword and skip the rest
        let _ = parser.expect(crate::SG_MUL_VAL_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
