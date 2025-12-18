use super::{Position, Range, Scaling, Signal};
use crate::{ByteOrder, Error, MAX_NAME_SIZE, Parser, Receivers, Result, compat::Name};

impl Signal {
    fn parse_position<'b>(parser: &mut Parser<'b>) -> Result<Position> {
        // Parse start_bit
        let start_bit = match parser.parse_u32() {
            Ok(v) => v as u16,
            Err(_) => {
                return Err(parser.err_signal(Error::SIGNAL_PARSE_INVALID_START_BIT));
            }
        };

        // Validate start_bit range
        if start_bit > 511 {
            return Err(parser.err_signal(Error::SIGNAL_PARSE_INVALID_START_BIT));
        }

        // Expect pipe
        parser.expect_with_msg(b"|", "Expected '|' after start_bit")?;

        // Parse length
        let length = parser
            .parse_u32()
            .map_err(|_| parser.err_signal(Error::SIGNAL_PARSE_INVALID_LENGTH))?
            as u16;

        // Expect @
        parser.expect_with_msg(b"@", "Expected '@' after signal length")?;

        // Parse byte order (0 or 1)
        // Try to expect '0' or '1' directly
        let bo_byte = if parser.expect(b"0").is_ok() {
            b'0'
        } else if parser.expect(b"1").is_ok() {
            b'1'
        } else {
            return Err(parser.err_expected("Expected byte order (0=big-endian, 1=little-endian)"));
        };

        let byte_order = match bo_byte {
            b'0' => ByteOrder::BigEndian,    // 0 = Motorola (big-endian)
            b'1' => ByteOrder::LittleEndian, // 1 = Intel (little-endian)
            _ => return Err(parser.err_invalid_char(bo_byte as char)),
        };

        // Parse sign (+ or -)
        let sign_byte = if parser.expect(b"+").is_ok() {
            b'+'
        } else if parser.expect(b"-").is_ok() {
            b'-'
        } else {
            return Err(
                parser.err_expected("Expected sign indicator ('+' for unsigned, '-' for signed)")
            );
        };

        let unsigned = match sign_byte {
            b'+' => true,
            b'-' => false,
            _ => return Err(parser.err_invalid_char(sign_byte as char)),
        };

        Ok((start_bit, length, byte_order, unsigned))
    }

    fn parse_factor_offset<'b>(parser: &mut Parser<'b>) -> Result<Scaling> {
        // Expect opening parenthesis
        parser.expect_with_msg(b"(", "Expected '(' to start factor/offset")?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse factor (may be empty, default to 0.0)
        let factor = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| parser.err_signal(Error::SIGNAL_PARSE_INVALID_FACTOR))?;

        // Expect comma, then skip whitespace
        parser.expect_then_skip(b",")?;

        // Parse offset (may be empty, default to 0.0)
        let offset = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| parser.err_signal(Error::SIGNAL_PARSE_INVALID_OFFSET))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing parenthesis
        parser.expect_with_msg(b")", "Expected ')' to close factor/offset")?;

        Ok((factor, offset))
    }

    fn parse_range<'b>(parser: &mut Parser<'b>) -> Result<Range> {
        // Expect opening bracket
        parser.expect_with_msg(b"[", "Expected '[' to start min/max range")?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse min (may be empty, default to 0.0)
        let min = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| parser.err_signal(Error::SIGNAL_PARSE_INVALID_MIN))?;

        // Expect pipe, then skip whitespace
        parser.expect_then_skip(b"|")?;

        // Parse max (may be empty, default to 0.0)
        let max = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| parser.err_signal(Error::SIGNAL_PARSE_INVALID_MAX))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing bracket
        parser.expect_with_msg(b"]", "Expected ']' to close min/max range")?;

        Ok((min, max))
    }

    fn parse_unit(parser: &mut Parser) -> Result<Option<Name>> {
        // Expect opening quote
        parser.expect_with_msg(b"\"", "Expected '\"' to start unit string")?;

        // Use take_until_quote to read the unit (allow any printable characters)
        let unit_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).map_err(|e| match e {
            Error::MaxStrLength { .. } => parser.err_signal(Error::SIGNAL_PARSE_UNIT_TOO_LONG),
            _ => parser.err_expected("Expected closing '\"' for unit string"),
        })?;

        // Convert bytes to string slice
        let unit_str = core::str::from_utf8(unit_bytes)
            .map_err(|_e| parser.err_expected(Error::INVALID_UTF8))?;

        let unit: Name = Name::try_from(unit_str)
            .map_err(|_| parser.err_version(Error::MAX_NAME_SIZE_EXCEEDED))?;

        let unit = if unit.is_empty() { None } else { Some(unit) };
        Ok(unit)
    }

    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Signal parsing must always start with "SG_" keyword
        parser.expect_keyword_then_skip(crate::SG_.as_bytes(), "Expected SG_ keyword")?;

        // Parse signal name (identifier)
        let name =
            parser.parse_identifier_with_error(|| Error::signal(Error::SIGNAL_NAME_EMPTY))?;

        // Parse multiplexer indicator
        // According to spec: multiplexer_indicator = ' ' | 'M' | 'm' multiplexer_switch_value
        // Extended multiplexing (Vector Informatik): 'm' switch_value 'M' means the signal is
        // both a multiplexed signal (dependent on higher-level multiplexer) and a multiplexer
        // switch itself (controlling lower-level signals).
        parser.skip_newlines_and_spaces();

        let mut is_multiplexer_switch = false;
        let mut multiplexer_switch_value: Option<u64> = None;

        // Check for 'M' (multiplexer switch) or 'm' followed by optional digits (multiplexed signal)
        if parser.expect(b"M").is_ok() {
            // This is a multiplexer switch signal (but not multiplexed itself)
            is_multiplexer_switch = true;
            parser.skip_newlines_and_spaces();
        } else if parser.expect(b"m").is_ok() {
            // This is a multiplexed signal - parse the switch value
            // Manually parse digits because parse_u64() stops at specific chars and 'M' isn't one
            let mut switch_value: Option<u64> = None;

            // Manually parse digits until we hit 'M', whitespace, or colon
            let mut value = 0u64;
            let mut found_digits = false;
            loop {
                if parser.eof() {
                    break;
                }
                let Some(byte) = parser.current_byte() else {
                    break;
                };
                if byte.is_ascii_digit() {
                    found_digits = true;
                    value = value
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((byte - b'0') as u64))
                        .ok_or_else(|| parser.err_signal(Error::SIGNAL_ERROR_PREFIX))?;
                    parser.advance_one();
                } else if byte == b'M' || parser.matches_any(b" \t:") || parser.at_newline() {
                    // Stop at 'M', whitespace, or colon
                    break;
                } else {
                    // Invalid character - stop parsing
                    break;
                }
            }

            if found_digits {
                switch_value = Some(value);
            }

            multiplexer_switch_value = switch_value;

            // Check if 'M' follows the switch value (extended multiplexing: m65M)
            // This means the signal is both multiplexed AND acts as a multiplexer switch
            if parser.expect(b"M").is_ok() {
                is_multiplexer_switch = true;
            }
            parser.skip_newlines_and_spaces();
        }

        // Expect colon
        parser.expect_with_msg(b":", "Expected ':' after signal name")?;

        // Skip whitespace after colon
        parser.skip_newlines_and_spaces();

        // Parse position: start_bit|length@byteOrderSign
        let (start_bit, length, byte_order, unsigned) = Self::parse_position(parser)?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse factor and offset: (factor,offset)
        let (factor, offset) = Self::parse_factor_offset(parser)?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse range: [min|max]
        let (min, max) = Self::parse_range(parser)?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse unit: "unit" or ""
        let unit = Self::parse_unit(parser)?;

        // Skip whitespace (but not newlines) before parsing receivers
        // Newlines indicate end of signal line, so we need to preserve them for Receivers::parse
        parser.skip_whitespace_optional();

        // Parse receivers (may be empty/None if at end of line)
        let receivers = Receivers::parse(parser)?;
        // TODO: Receivers need to be validated

        // Validate before construction
        Self::validate(name, length, min, max).map_err(|e| {
            crate::error::map_val_error_with_line(
                e,
                |msg| parser.err_signal(msg),
                || parser.err_signal(Error::SIGNAL_ERROR_PREFIX),
            )
        })?;

        let name = crate::compat::validate_name(name)?;

        // Construct directly (validation already done)
        Ok(Self {
            name,
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            receivers,
            is_multiplexer_switch,
            multiplexer_switch_value,
        })
    }
}
