use super::Signal;
use crate::{ByteOrder, Error, MAX_NAME_SIZE, Parser, Receivers, Result, compat::String};

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

        // Skip whitespace (optional before colon) - handle multiplexer indicator
        // According to spec: multiplexer_indicator = ' ' | [m multiplexer_switch_value] [M]
        // For now, we just skip whitespace and any potential multiplexer indicator
        parser.skip_newlines_and_spaces();

        // Skip potential multiplexer indicator (m followed by number, or M)
        // For simplicity, skip any 'm' or 'M' followed by digits
        if parser.expect(b"m").is_ok() || parser.expect(b"M").is_ok() {
            // Skip any digits that follow
            loop {
                let _pos_before = parser.pos();
                // Try to consume a digit
                if parser.expect(b"0").is_ok()
                    || parser.expect(b"1").is_ok()
                    || parser.expect(b"2").is_ok()
                    || parser.expect(b"3").is_ok()
                    || parser.expect(b"4").is_ok()
                    || parser.expect(b"5").is_ok()
                    || parser.expect(b"6").is_ok()
                    || parser.expect(b"7").is_ok()
                    || parser.expect(b"8").is_ok()
                    || parser.expect(b"9").is_ok()
                {
                    // Consumed a digit, continue
                } else {
                    // Not a digit, stop
                    break;
                }
            }
            // Skip whitespace after multiplexer indicator
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
        Signal::validate(name, length, min, max).map_err(|e| {
            crate::error::map_val_error(e, Error::Signal, || {
                Error::Signal(crate::error::Error::SIGNAL_ERROR_PREFIX)
            })
        })?;

        let name = crate::validate_name(name)?;

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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ByteOrder, Error, Parser, Receivers};

    #[test]
    fn test_parse_valid_signal() {
        let line = r#"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let sig = Signal::parse(&mut parser).unwrap();
        assert_eq!(sig.name(), "RPM");
        assert_eq!(sig.start_bit(), 0);
        assert_eq!(sig.length(), 16);
        assert_eq!(sig.byte_order(), ByteOrder::BigEndian); // @0 = BigEndian (Motorola)
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 0.25);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 8000.);
        assert_eq!(sig.unit(), Some("rpm"));
        // Check receivers using iter
        let mut receivers_iter = sig.receivers().iter();
        let receiver1 = receivers_iter.next().unwrap();
        assert_eq!(receiver1.as_str(), "TCM");
        assert_eq!(receivers_iter.next(), None);
    }

    #[test]
    fn test_parse_signal_with_empty_unit_and_broadcast() {
        let line = r#"SG_ ABSActive : 16|1@0+ (1,0) [0|1] "" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let sig = Signal::parse(&mut parser).unwrap();
        assert_eq!(sig.name(), "ABSActive");
        assert_eq!(sig.start_bit(), 16);
        assert_eq!(sig.length(), 1);
        assert_eq!(sig.byte_order(), ByteOrder::BigEndian); // @0 = BigEndian (Motorola)
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 1.);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 1.);
        assert_eq!(sig.unit(), None);
        assert_eq!(sig.receivers(), &Receivers::Broadcast);
    }

    #[test]
    fn test_parse_signal_with_negative_offset_and_min() {
        let line = r#"SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM BCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let sig = Signal::parse(&mut parser).unwrap();
        assert_eq!(sig.name(), "Temperature");
        assert_eq!(sig.start_bit(), 16);
        assert_eq!(sig.length(), 8);
        assert_eq!(sig.byte_order(), ByteOrder::BigEndian); // @0 = BigEndian (Motorola)
        assert!(!sig.is_unsigned());
        assert_eq!(sig.factor(), 1.);
        assert_eq!(sig.offset(), -40.);
        assert_eq!(sig.min(), -40.);
        assert_eq!(sig.max(), 215.);
        assert_eq!(sig.unit(), Some("°C"));
        // Check receivers using iter
        let mut receivers_iter = sig.receivers().iter();
        let receiver1 = receivers_iter.next().unwrap();
        assert_eq!(receiver1.as_str(), "TCM");
        let receiver2 = receivers_iter.next().unwrap();
        assert_eq!(receiver2.as_str(), "BCM");
        assert_eq!(receivers_iter.next(), None);
    }

    #[test]
    fn test_parse_signal_with_percent_unit() {
        let line = r#"SG_ ThrottlePosition : 24|8@0+ (0.392157,0) [0|100] "%" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let sig = Signal::parse(&mut parser).unwrap();
        assert_eq!(sig.name(), "ThrottlePosition");
        assert_eq!(sig.start_bit(), 24);
        assert_eq!(sig.length(), 8);
        assert_eq!(sig.byte_order(), ByteOrder::BigEndian); // @0 = BigEndian (Motorola)
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 0.392_157);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 100.);
        assert_eq!(sig.unit(), Some("%"));
        assert_eq!(sig.receivers(), &Receivers::Broadcast);
    }

    #[test]
    fn test_parse_signal_missing_factors_and_limits() {
        // Should use default values where missing
        let line = r#"SG_ Simple : 10|4@0+ ( , ) [ | ] "" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let sig = Signal::parse(&mut parser).unwrap();
        assert_eq!(sig.name(), "Simple");
        assert_eq!(sig.start_bit(), 10);
        assert_eq!(sig.length(), 4);
        assert_eq!(sig.byte_order(), ByteOrder::BigEndian); // @0 = BigEndian (Motorola)
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 0.);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 0.);
        assert_eq!(sig.unit(), None);
        assert_eq!(sig.receivers(), &Receivers::Broadcast);
    }

    #[test]
    fn test_parse_signal_missing_start_bit() {
        let line = r#"SG_ RPM : |16@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            Error::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(Error::SIGNAL_PARSE_INVALID_START_BIT)
                        || msg.contains("Signal 'RPM'")
                );
            }
            _ => panic!("Expected Error::Signal"),
        }
    }

    #[test]
    fn test_parse_signal_invalid_range() {
        // min > max should fail validation
        let line = r#"SG_ Test : 0|8@0+ (1,0) [100|50] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            Error::Signal(msg) => {
                assert!(msg.contains(Error::INVALID_RANGE));
            }
            e => panic!("Expected Error::Signal, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_overflow() {
        // Signal with start_bit + length > 64 should parse successfully
        // (validation against message DLC happens in Message::validate)
        // This signal would fit in a CAN FD message (64 bytes = 512 bits)
        let line = r#"SG_ Test : 60|10@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signal = Signal::parse(&mut parser).unwrap();
        assert_eq!(signal.start_bit(), 60);
        assert_eq!(signal.length(), 10);
        // Note: Message validation tests are in message.rs and message_builder.rs
    }

    #[test]
    fn test_parse_signal_length_too_large() {
        // length > 512 should fail validation (CAN FD maximum is 512 bits)
        let line = r#"SG_ Test : 0|513@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            Error::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(Error::SIGNAL_LENGTH_TOO_LARGE)
                        || msg.contains("Signal 'Test'")
                        || msg.contains("513")
                );
            }
            e => panic!("Expected Error::Signal, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_zero_length() {
        // length = 0 should fail validation
        let line = r#"SG_ Test : 0|0@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            Error::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(Error::SIGNAL_LENGTH_TOO_SMALL)
                        || msg.contains("Signal 'Test'")
                        || msg.contains("0 bits")
                );
            }
            e => panic!("Expected Error::Signal, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_missing_length() {
        let line = r#"SG_ RPM : 0|@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            Error::Signal(msg) => assert!(msg.contains(Error::SIGNAL_PARSE_INVALID_LENGTH)),
            e => panic!("Expected Error::Signal, got: {:?}", e),
        }
    }
}
