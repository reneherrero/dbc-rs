use crate::{
    ByteOrder, Parser, Receivers,
    error::{ParseError, ParseResult},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Signal<'a> {
    name: &'a str,
    start_bit: u16,
    length: u16,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<&'a str>,
    receivers: Receivers<'a>,
}

impl<'a> Signal<'a> {
    pub(crate) fn validate(name: &str, length: u16, min: f64, max: f64) -> ParseResult<()> {
        if name.trim().is_empty() {
            return Err(ParseError::Version(crate::error::lang::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        if length == 0 {
            return Err(ParseError::Version(
                crate::error::lang::SIGNAL_LENGTH_TOO_SMALL,
            ));
        }
        if length > 512 {
            return Err(ParseError::Version(
                crate::error::lang::SIGNAL_LENGTH_TOO_LARGE,
            ));
        }

        // Note: start_bit validation (boundary checks and overlap detection) is done in
        // Message::validate, not here, because:
        // 1. The actual message size depends on DLC (1-64 bytes for CAN FD)
        // 2. Overlap detection requires comparing multiple signals
        // 3. This allows signals to be created independently and validated when added to a message

        // Validate min <= max
        if min > max {
            // In no_std, we can't format the error message, so use a generic error
            return Err(ParseError::Version(
                crate::error::lang::FORMAT_INVALID_RANGE,
            ));
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // Internal method, builder pattern is the public API
    #[allow(dead_code)] // Only used by builders (std-only)
    pub(crate) fn new(
        name: &'a str,
        start_bit: u16,
        length: u16,
        byte_order: ByteOrder,
        unsigned: bool,
        factor: f64,
        offset: f64,
        min: f64,
        max: f64,
        unit: Option<&'a str>,
        receivers: Receivers<'a>,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
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
        }
    }

    fn parse_position<'b>(parser: &mut Parser<'b>) -> ParseResult<(u16, u16, ByteOrder, bool)> {
        // Parse start_bit
        let start_bit = parser
            .parse_u32()
            .map_err(|_| ParseError::Version(crate::error::lang::SIGNAL_PARSE_INVALID_START_BIT))?
            as u16;

        // Expect pipe
        parser.expect(b"|").map_err(|_| ParseError::Expected("Expected pipe"))?;

        // Parse length
        let length = parser
            .parse_u32()
            .map_err(|_| ParseError::Version(crate::error::lang::SIGNAL_PARSE_INVALID_LENGTH))?
            as u16;

        // Expect @
        parser.expect(b"@").map_err(|_| ParseError::Expected("Expected @"))?;

        // Parse byte order (0 or 1)
        // Try to expect '0' or '1' directly
        let bo_byte = if parser.expect(b"0").is_ok() {
            b'0'
        } else if parser.expect(b"1").is_ok() {
            b'1'
        } else {
            return Err(ParseError::Expected("Expected byte order"));
        };

        let byte_order = match bo_byte {
            b'0' => ByteOrder::BigEndian,    // 0 = Motorola (big-endian)
            b'1' => ByteOrder::LittleEndian, // 1 = Intel (little-endian)
            _ => return Err(ParseError::InvalidChar(bo_byte as char)),
        };

        // Parse sign (+ or -)
        let sign_byte = if parser.expect(b"+").is_ok() {
            b'+'
        } else if parser.expect(b"-").is_ok() {
            b'-'
        } else {
            return Err(ParseError::Expected("Expected sign (+ or -)"));
        };

        let unsigned = match sign_byte {
            b'+' => true,
            b'-' => false,
            _ => return Err(ParseError::InvalidChar(sign_byte as char)),
        };

        Ok((start_bit, length, byte_order, unsigned))
    }

    fn parse_factor_offset<'b>(parser: &mut Parser<'b>) -> ParseResult<(f64, f64)> {
        // Expect opening parenthesis
        parser
            .expect(b"(")
            .map_err(|_| ParseError::Expected("Expected opening parenthesis"))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse factor (may be empty, default to 0.0)
        // parse_f64() stops at comma/paren without consuming them
        // If parsing fails immediately (pos unchanged), we're at a delimiter (empty factor)
        let pos_before = parser.pos();
        let factor = match parser.parse_f64() {
            Ok(val) => val,
            Err(_) => {
                // Check if position didn't change (we're at delimiter)
                if parser.pos() == pos_before {
                    0.0 // Empty factor
                } else {
                    // Position changed but parsing failed - invalid format
                    return Err(ParseError::Version(
                        crate::error::lang::SIGNAL_PARSE_INVALID_FACTOR,
                    ));
                }
            }
        };

        // Expect comma
        parser.expect(b",").map_err(|_| ParseError::Expected("Expected comma"))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse offset (may be empty, default to 0.0)
        let pos_before = parser.pos();
        let offset = match parser.parse_f64() {
            Ok(val) => val,
            Err(_) => {
                // Check if position didn't change (we're at closing paren)
                if parser.pos() == pos_before {
                    0.0 // Empty offset
                } else {
                    return Err(ParseError::Version(
                        crate::error::lang::SIGNAL_PARSE_INVALID_OFFSET,
                    ));
                }
            }
        };

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing parenthesis
        parser
            .expect(b")")
            .map_err(|_| ParseError::Expected("Expected closing parenthesis"))?;

        Ok((factor, offset))
    }

    fn parse_range<'b>(parser: &mut Parser<'b>) -> ParseResult<(f64, f64)> {
        // Expect opening bracket
        parser
            .expect(b"[")
            .map_err(|_| ParseError::Expected("Expected opening bracket"))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse min (may be empty, default to 0.0)
        let pos_before = parser.pos();
        let min = match parser.parse_f64() {
            Ok(val) => val,
            Err(_) => {
                // Check if position didn't change (we're at pipe or closing bracket)
                if parser.pos() == pos_before {
                    0.0 // Empty min
                } else {
                    return Err(ParseError::Version(
                        crate::error::lang::SIGNAL_PARSE_INVALID_MIN,
                    ));
                }
            }
        };

        // Expect pipe
        parser.expect(b"|").map_err(|_| ParseError::Expected("Expected pipe"))?;

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Parse max (may be empty, default to 0.0)
        let pos_before = parser.pos();
        let max = match parser.parse_f64() {
            Ok(val) => val,
            Err(_) => {
                // Check if position didn't change (we're at closing bracket)
                if parser.pos() == pos_before {
                    0.0 // Empty max
                } else {
                    return Err(ParseError::Version(
                        crate::error::lang::SIGNAL_PARSE_INVALID_MAX,
                    ));
                }
            }
        };

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing bracket
        parser
            .expect(b"]")
            .map_err(|_| ParseError::Expected("Expected closing bracket"))?;

        Ok((min, max))
    }

    fn parse_unit<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Option<&'a str>> {
        const MAX_UNIT_LENGTH: u16 = 256;

        // Expect opening quote
        parser
            .expect(b"\"")
            .map_err(|_| ParseError::Expected("Expected opening quote"))?;

        // Use take_until_quote to read the unit (allow any printable characters)
        let unit_bytes = parser.take_until_quote(false, MAX_UNIT_LENGTH).map_err(|e| match e {
            ParseError::MaxStrLength(_) => {
                ParseError::Version(crate::error::lang::SIGNAL_PARSE_UNIT_TOO_LONG)
            }
            _ => ParseError::Expected("Expected closing quote"),
        })?;

        // Convert bytes to string slice
        let unit_str = core::str::from_utf8(unit_bytes)
            .map_err(|_| ParseError::Expected("Invalid UTF-8 in unit"))?;

        let unit = if unit_str.is_empty() {
            None
        } else {
            Some(unit_str)
        };

        Ok(unit)
    }

    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        // When called from Dbc::parse, find_next_keyword already consumed "SG_" and advanced past it
        // So the parser is now at the space after "SG_". We just need to skip that whitespace.
        // But if "SG_" wasn't consumed (standalone call), we need to expect it first.
        // Try to expect "SG_" - if it fails, we're already past it from find_next_keyword
        let _ = parser.expect(crate::SG_.as_bytes()).ok(); // Ignore error if already past "SG_"

        // Skip whitespace after "SG_"
        parser.skip_newlines_and_spaces();

        // Parse signal name (identifier)
        let name = parser
            .parse_identifier()
            .map_err(|_| ParseError::Version(crate::error::lang::SIGNAL_NAME_EMPTY))?;

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
        parser.expect(b":").map_err(|_| ParseError::Expected("Expected colon"))?;

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
        let _ = parser.skip_whitespace().ok(); // Ignore error if no whitespace

        // Parse receivers (may be empty/None if at end of line)
        let receivers = Receivers::parse(parser)?;

        // Validate before construction
        Self::validate(name, length, min, max)?;
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

    #[inline]
    #[must_use = "return value should be checked"]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    #[must_use]
    pub fn start_bit(&self) -> u16 {
        self.start_bit
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> u16 {
        self.length
    }

    #[inline]
    #[must_use]
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    #[inline]
    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        self.unsigned
    }

    #[inline]
    #[must_use]
    pub fn factor(&self) -> f64 {
        self.factor
    }

    #[inline]
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    #[inline]
    #[must_use]
    pub fn min(&self) -> f64 {
        self.min
    }

    #[inline]
    #[must_use]
    pub fn max(&self) -> f64 {
        self.max
    }

    #[inline]
    #[must_use]
    pub fn unit(&self) -> Option<&'a str> {
        self.unit
    }

    #[inline]
    #[must_use]
    pub fn receivers(&self) -> &Receivers<'a> {
        &self.receivers
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        let mut result = String::with_capacity(100); // Pre-allocate reasonable capacity

        result.push_str(" SG_ ");
        result.push_str(self.name());
        result.push_str(" : ");
        result.push_str(&self.start_bit().to_string());
        result.push('|');
        result.push_str(&self.length().to_string());
        result.push('@');

        // Byte order: 0 for BigEndian (Motorola), 1 for LittleEndian (Intel)
        // Per Vector DBC spec v1.0.1: "Big endian is stored as '0', little endian is stored as '1'"
        match self.byte_order() {
            ByteOrder::BigEndian => result.push('0'), // @0 = Big Endian (Motorola)
            ByteOrder::LittleEndian => result.push('1'), // @1 = Little Endian (Intel)
        }

        // Sign: + for unsigned, - for signed
        if self.is_unsigned() {
            result.push('+');
        } else {
            result.push('-');
        }

        // Factor and offset: (factor,offset)
        result.push_str(" (");
        result.push_str(&format!("{}", self.factor()));
        result.push(',');
        result.push_str(&format!("{}", self.offset()));
        result.push(')');

        // Min and max: [min|max]
        result.push_str(" [");
        result.push_str(&format!("{}", self.min()));
        result.push('|');
        result.push_str(&format!("{}", self.max()));
        result.push(']');

        // Unit: "unit" or ""
        result.push(' ');
        if let Some(unit) = self.unit() {
            result.push('"');
            result.push_str(unit);
            result.push('"');
        } else {
            result.push_str("\"\"");
        }

        // Receivers: * for Broadcast, space-separated list for Nodes, or empty
        match self.receivers() {
            Receivers::Broadcast => {
                result.push(' ');
                result.push('*');
            }
            Receivers::Nodes(_, count) => {
                if *count > 0 {
                    result.push(' ');
                    for (i, node) in self.receivers().iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(node);
                    }
                }
            }
            Receivers::None => {
                // No receivers specified - nothing to add
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{
        Error, Parser,
        error::{ParseError, lang},
    };
    #[cfg(feature = "alloc")]
    use crate::{MessageBuilder, ReceiversBuilder, SignalBuilder};

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_valid() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(0.25)
            .offset(0.0)
            .min(0.0)
            .max(8000.0)
            .unit("rpm")
            .receivers(ReceiversBuilder::new().broadcast().build().unwrap())
            .build()
            .unwrap();
        assert_eq!(signal.name(), "RPM");
        assert_eq!(signal.start_bit(), 0);
        assert_eq!(signal.length(), 16);
        assert_eq!(signal.byte_order(), ByteOrder::BigEndian);
        assert!(signal.is_unsigned());
        assert_eq!(signal.factor(), 0.25);
        assert_eq!(signal.offset(), 0.0);
        assert_eq!(signal.min(), 0.0);
        assert_eq!(signal.max(), 8000.0);
        assert_eq!(signal.unit(), Some("rpm"));
        assert_eq!(signal.receivers(), &Receivers::Broadcast);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_empty_name() {
        let result = SignalBuilder::new()
            .name("")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_NAME_EMPTY)),
            _ => panic!("Expected Error::Signal"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_zero_length() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(0)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)),
            _ => panic!("Expected Error::Signal"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_length_too_large() {
        // length > 512 should fail validation (CAN FD maximum is 512 bits)
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(513)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)),
            _ => panic!("Expected Error::Signal"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_overflow() {
        // Signal with start_bit + length > 64 should be created successfully
        // (validation against message DLC happens in Message::validate)
        // This signal would fit in a CAN FD message (64 bytes = 512 bits)
        let signal = SignalBuilder::new()
            .name("Test")
            .start_bit(60)
            .length(10) // 60 + 10 = 70, fits in CAN FD (512 bits)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
            .unwrap();

        // But it should fail when added to a message with DLC < 9 bytes
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Error::Dbc"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_invalid_range() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(100.0)
            .max(50.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_INVALID_RANGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Error::Signal"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_max_boundary() {
        // Test that 64 bits at position 0 is valid
        let signal = SignalBuilder::new()
            .name("FullMessage")
            .start_bit(0)
            .length(64)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
            .unwrap();
        assert_eq!(signal.length(), 64);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_new_with_receivers() {
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(8)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(false)
            .factor(0.1)
            .offset(-40.0)
            .min(-40.0)
            .max(215.0)
            .unit("°C")
            .receivers(ReceiversBuilder::new().add_node("ECM").add_node("TCM").build().unwrap())
            .build()
            .unwrap();
        assert_eq!(signal.name(), "TestSignal");
        assert!(!signal.is_unsigned());
        assert_eq!(signal.unit(), Some("°C"));
        match signal.receivers() {
            Receivers::Nodes(_, count) => assert_eq!(*count, 2),
            _ => panic!("Expected Nodes variant"),
        }
    }

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
        // Check receivers using iter_nodes
        let nodes: Vec<&str> = sig.receivers().iter().collect();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], "TCM");
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
        // Check receivers using iter_nodes
        let nodes: Vec<&str> = sig.receivers().iter().collect();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], "TCM");
        assert_eq!(nodes[1], "BCM");
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
            ParseError::Version(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_START_BIT)),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_signal_invalid_range() {
        // min > max should fail validation
        let line = r#"SG_ Test : 0|8@0+ (1,0) [100|50] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Version(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_INVALID_RANGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            e => panic!("Expected ParseError::Version, got: {:?}", e),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_parse_signal_overflow() {
        // Signal with start_bit + length > 64 should parse successfully
        // (validation against message DLC happens in Message::validate)
        // This signal would fit in a CAN FD message (64 bytes = 512 bits)
        let line = r#"SG_ Test : 60|10@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signal = Signal::parse(&mut parser).unwrap();
        assert_eq!(signal.start_bit(), 60);
        assert_eq!(signal.length(), 10);

        // But it should fail when added to a message with DLC < 9 bytes
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Error::Dbc"),
        }
    }

    #[test]
    fn test_parse_signal_length_too_large() {
        // length > 512 should fail validation (CAN FD maximum is 512 bits)
        let line = r#"SG_ Test : 0|513@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Version(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)),
            e => panic!("Expected ParseError::Version, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_zero_length() {
        // length = 0 should fail validation
        let line = r#"SG_ Test : 0|0@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Version(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)),
            e => panic!("Expected ParseError::Version, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_missing_length() {
        let line = r#"SG_ RPM : 0|@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Version(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_LENGTH)),
            e => panic!("Expected ParseError::Version, got: {:?}", e),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_signal_to_dbc_string() {
        // Test with Broadcast receiver
        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(0.25)
            .offset(0.0)
            .min(0.0)
            .max(8000.0)
            .unit("rpm")
            .receivers(ReceiversBuilder::new().broadcast().build().unwrap())
            .build()
            .unwrap();
        assert_eq!(
            signal1.to_dbc_string(),
            " SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *"
        );

        // Test with Nodes receiver
        let signal2 = SignalBuilder::new()
            .name("Temperature")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(false)
            .factor(1.0)
            .offset(-40.0)
            .min(-40.0)
            .max(215.0)
            .unit("°C")
            .receivers(ReceiversBuilder::new().add_node("TCM").add_node("BCM").build().unwrap())
            .build()
            .unwrap();
        assert_eq!(
            signal2.to_dbc_string(),
            " SG_ Temperature : 16|8@1- (1,-40) [-40|215] \"°C\" TCM BCM"
        );

        // Test with None receiver and empty unit
        let signal3 = SignalBuilder::new()
            .name("Flag")
            .start_bit(24)
            .length(1)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(1.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
            .unwrap();
        assert_eq!(
            signal3.to_dbc_string(),
            " SG_ Flag : 24|1@0+ (1,0) [0|1] \"\""
        );
    }

    // Note: Helper parsing functions (parse_name_and_prefix, parse_position, etc.) are now internal
    // and use the Parser directly. Their functionality is tested through Signal::parse tests above.
    // All tests for these helper methods have been removed as they are implementation details.
}
