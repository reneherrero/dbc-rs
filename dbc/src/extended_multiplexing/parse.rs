use super::ExtendedMultiplexing;
use crate::{
    Parser,
    compat::{Vec, validate_name},
};

impl ExtendedMultiplexing {
    /// Parse an SG_MUL_VAL_ entry
    ///
    /// Expects the parser to be positioned after the SG_MUL_VAL_ keyword.
    /// Parses: message_id signal_name multiplexer_switch value_ranges ;
    /// Example: 500 Signal_A Mux1 0-5,10-15 ;
    ///
    /// Returns `None` if parsing fails (caller should skip to end of line).
    pub(crate) fn parse(parser: &mut Parser) -> Option<ExtendedMultiplexing> {
        parser.skip_newlines_and_spaces();

        // Parse message_id
        let message_id = parser.parse_u32().ok()?;
        parser.skip_newlines_and_spaces();

        // Parse signal_name
        let signal_name_str = parser.parse_identifier().ok()?;
        let signal_name = validate_name(signal_name_str).ok()?;
        parser.skip_newlines_and_spaces();

        // Parse multiplexer_switch
        let multiplexer_switch_str = parser.parse_identifier().ok()?;
        let multiplexer_switch = validate_name(multiplexer_switch_str).ok()?;
        parser.skip_newlines_and_spaces();

        // Parse value ranges
        let mut value_ranges: Vec<(u64, u64), 64> = Vec::new();

        // Parse value ranges (at least one required)
        loop {
            parser.skip_newlines_and_spaces();

            // Parse value range: min-max
            let min_u32 = parser.parse_u32().ok()?;
            let min = min_u32 as u64;
            parser.skip_newlines_and_spaces();

            if parser.expect(b"-").is_err() {
                break;
            }
            parser.skip_newlines_and_spaces();

            let max_u32 = parser.parse_u32().ok()?;
            let max = max_u32 as u64;

            if value_ranges.push((min, max)).is_err() {
                // Vector full, stop parsing
                break;
            }

            // Check for comma (more ranges) or semicolon (end)
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b",") {
                parser.expect(b",").ok()?;
                // Continue to next range
            } else if parser.starts_with(b";") {
                parser.expect(b";").ok()?;
                break;
            } else {
                // End of ranges (no comma or semicolon)
                break;
            }
        }

        // Only return if we parsed at least one range
        if value_ranges.is_empty() {
            return None;
        }

        Some(ExtendedMultiplexing::new(
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        ))
    }
}
