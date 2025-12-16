use super::Signal;
use crate::compat::String;
use crate::{ByteOrder, Error, MAX_NAME_SIZE, Parser, Receivers, Result};

impl Signal {
    fn parse_position<'b>(parser: &mut Parser<'b>) -> Result<(u16, u16, ByteOrder, bool)> {
        // Parse start_bit
        let start_bit = match parser.parse_u32() {
            Ok(v) => v as u16,
            Err(_) => {
                return Err(Error::Signal(Error::SIGNAL_PARSE_INVALID_START_BIT));
            }
        };

        // Validate start_bit range
        if start_bit > 511 {
            return Err(Error::Signal(Error::SIGNAL_PARSE_INVALID_START_BIT));
        }

        // Expect pipe
        parser.expect_with_msg(b"|", "Expected pipe")?;

        // Parse length
        let length = parser
            .parse_u32()
            .map_err(|_| Error::Signal(Error::SIGNAL_PARSE_INVALID_LENGTH))?
            as u16;

        // Expect @
        parser.expect_with_msg(b"@", "Expected @")?;

        // Parse byte order (0 or 1)
        // Try to expect '0' or '1' directly
        let bo_byte = if parser.expect(b"0").is_ok() {
            b'0'
        } else if parser.expect(b"1").is_ok() {
            b'1'
        } else {
            return Err(Error::Expected("Expected byte order"));
        };

        let byte_order = match bo_byte {
            b'0' => ByteOrder::BigEndian,    // 0 = Motorola (big-endian)
            b'1' => ByteOrder::LittleEndian, // 1 = Intel (little-endian)
            _ => return Err(Error::InvalidChar(bo_byte as char)),
        };

        // Parse sign (+ or -)
        let sign_byte = if parser.expect(b"+").is_ok() {
            b'+'
        } else if parser.expect(b"-").is_ok() {
            b'-'
        } else {
            return Err(Error::Expected("Expected sign (+ or -)"));
        };

        let unsigned = match sign_byte {
            b'+' => true,
            b'-' => false,
            _ => return Err(Error::InvalidChar(sign_byte as char)),
        };

        Ok((start_bit, length, byte_order, unsigned))
    }

    fn parse_factor_offset<'b>(parser: &mut Parser<'b>) -> Result<(f64, f64)> {
        // Expect opening parenthesis
        parser.expect_with_msg(b"(", "Expected opening parenthesis")?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse factor (may be empty, default to 0.0)
        let factor = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| Error::Signal(Error::SIGNAL_PARSE_INVALID_FACTOR))?;

        // Expect comma, then skip whitespace
        parser.expect_then_skip(b",")?;

        // Parse offset (may be empty, default to 0.0)
        let offset = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| Error::Signal(crate::error::Error::SIGNAL_PARSE_INVALID_OFFSET))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing parenthesis
        parser.expect_with_msg(b")", "Expected closing parenthesis")?;

        Ok((factor, offset))
    }

    fn parse_range<'b>(parser: &mut Parser<'b>) -> Result<(f64, f64)> {
        // Expect opening bracket
        parser.expect_with_msg(b"[", "Expected opening bracket")?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse min (may be empty, default to 0.0)
        let min = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| Error::Signal(crate::error::Error::SIGNAL_PARSE_INVALID_MIN))?;

        // Expect pipe, then skip whitespace
        parser.expect_then_skip(b"|")?;

        // Parse max (may be empty, default to 0.0)
        let max = parser
            .parse_f64_or_default(0.0)
            .map_err(|_| Error::Signal(crate::error::Error::SIGNAL_PARSE_INVALID_MAX))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing bracket
        parser.expect_with_msg(b"]", "Expected closing bracket")?;

        Ok((min, max))
    }

    fn parse_unit(parser: &mut Parser) -> Result<Option<String<{ MAX_NAME_SIZE }>>> {
        // Expect opening quote
        parser.expect_with_msg(b"\"", "Expected opening quote")?;

        // Use take_until_quote to read the unit (allow any printable characters)
        let unit_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).map_err(|e| match e {
            Error::MaxStrLength(_) => {
                Error::Signal(crate::error::Error::SIGNAL_PARSE_UNIT_TOO_LONG)
            }
            _ => Error::Expected("Expected closing quote"),
        })?;

        // Convert bytes to string slice
        let unit =
            core::str::from_utf8(unit_bytes).map_err(|_e| Error::Expected(Error::INVALID_UTF8))?;

        let unit: String<{ MAX_NAME_SIZE }> =
            String::try_from(unit).map_err(|_| Error::Version(Error::MAX_NAME_SIZE_EXCEEDED))?;

        let unit = if unit.is_empty() { None } else { Some(unit) };
        Ok(unit)
    }

    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Signal parsing must always start with "SG_" keyword
        parser.expect_keyword_then_skip(crate::SG_.as_bytes(), "Expected SG_ keyword")?;

        // Parse signal name (identifier)
        let name = parser.parse_identifier_with_error(|| {
            Error::Signal(crate::error::Error::SIGNAL_NAME_EMPTY)
        })?;

        // Parse multiplexer indicator
        // According to spec: multiplexer_indicator = ' ' | 'M' | 'm' multiplexer_switch_value
        parser.skip_newlines_and_spaces();

        let mut is_multiplexer_switch = false;
        let mut multiplexer_switch_value: Option<u64> = None;

        // Check for 'M' (multiplexer switch) or 'm' followed by digits (multiplexed signal)
        if parser.expect(b"M").is_ok() {
            // This is a multiplexer switch signal
            is_multiplexer_switch = true;
            parser.skip_newlines_and_spaces();
        } else if parser.expect(b"m").is_ok() {
            // This is a multiplexed signal - parse the switch value
            // Parse optional u64 value after 'm'
            let pos_before = parser.pos();
            if let Ok(value) = parser.parse_u64() {
                multiplexer_switch_value = Some(value);
            } else if parser.pos() == pos_before {
                // No digits found, which is optional for multiplexed signals
            } else {
                // Parsing failed after consuming some input - invalid format
                return Err(Error::Signal(crate::error::Error::SIGNAL_ERROR_PREFIX));
            }
            parser.skip_newlines_and_spaces();
        }

        // Expect colon
        parser.expect_with_msg(b":", "Expected colon")?;

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

        // Validate before construction
        Self::validate(name, length, min, max).map_err(|e| {
            crate::error::map_val_error(e, Error::Signal, || {
                Error::Signal(crate::error::Error::SIGNAL_ERROR_PREFIX)
            })
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
