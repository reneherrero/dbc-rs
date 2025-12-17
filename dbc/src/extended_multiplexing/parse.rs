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
            // We need to manually parse numbers because parse_u32() stops at whitespace/colon/pipe/@,
            // but not at "-", which causes it to error when it encounters "-" after the number.
            // We'll parse digits manually until we hit "-", whitespace, comma, or semicolon.
            let mut min_value = 0u32;
            let mut found_min_digits = false;
            loop {
                if parser.eof() {
                    break;
                }
                let Some(byte) = parser.current_byte() else {
                    break;
                };
                if byte.is_ascii_digit() {
                    found_min_digits = true;
                    min_value = min_value
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((byte - b'0') as u32))?;
                    parser.advance_one();
                } else if parser.matches_any(b" \t-,;") || parser.at_newline() {
                    // Stop at whitespace, -, comma, or semicolon
                    break;
                } else {
                    // Invalid character - stop parsing
                    break;
                }
            }

            if !found_min_digits {
                break;
            }
            let min = min_value as u64;
            parser.skip_newlines_and_spaces();

            if parser.expect(b"-").is_err() {
                break;
            }
            parser.skip_newlines_and_spaces();

            // Parse max value similarly
            let mut max_value = 0u32;
            let mut found_max_digits = false;
            loop {
                if parser.eof() {
                    break;
                }
                let Some(byte) = parser.current_byte() else {
                    break;
                };
                if byte.is_ascii_digit() {
                    found_max_digits = true;
                    max_value = max_value
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((byte - b'0') as u32))?;
                    parser.advance_one();
                } else if parser.matches_any(b" \t-,;") || parser.at_newline() {
                    // Stop at whitespace, -, comma, or semicolon
                    break;
                } else {
                    // Invalid character - stop parsing
                    break;
                }
            }

            if !found_max_digits {
                break;
            }
            let max = max_value as u64;

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

#[cfg(test)]
mod tests {
    use super::ExtendedMultiplexing;
    use crate::Parser;

    #[test]
    fn test_parse_extended_multiplexing_entry() {
        let input = b" 500 Signal_A Mux1 5-10 ;";
        let mut parser = Parser::new(input).unwrap();
        let result = ExtendedMultiplexing::parse(&mut parser);
        assert!(
            result.is_some(),
            "ExtendedMultiplexing::parse should succeed"
        );
        let ext_mux = result.unwrap();
        assert_eq!(ext_mux.message_id(), 500);
        assert_eq!(ext_mux.signal_name(), "Signal_A");
        assert_eq!(ext_mux.multiplexer_switch(), "Mux1");
        assert_eq!(ext_mux.value_ranges(), [(5, 10)]);
    }

    #[test]
    fn test_parse_extended_multiplexing_multiple_ranges() {
        let input = b" 500 Signal_B Mux1 0-5,10-15,20-25 ;";
        let mut parser = Parser::new(input).unwrap();
        let result = ExtendedMultiplexing::parse(&mut parser);
        assert!(result.is_some());
        let ext_mux = result.unwrap();
        assert_eq!(ext_mux.value_ranges(), [(0, 5), (10, 15), (20, 25)]);
    }
}
