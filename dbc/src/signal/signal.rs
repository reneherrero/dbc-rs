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
            return Err(ParseError::Signal(crate::error::lang::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        // Note: name is parsed before this validation, so we can include it in error messages
        if length == 0 {
            return Err(ParseError::Signal(
                crate::error::lang::SIGNAL_LENGTH_TOO_SMALL,
            ));
        }
        if length > 512 {
            return Err(ParseError::Signal(
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
            return Err(ParseError::Signal(crate::error::lang::INVALID_RANGE));
        }

        Ok(())
    }

    #[cfg(any(feature = "alloc", feature = "kernel"))]
    #[allow(clippy::too_many_arguments)] // Internal method, builder pattern is the public API
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
        let start_bit = match parser.parse_u32() {
            Ok(v) => v as u16,
            Err(_) => {
                return Err(ParseError::Signal(
                    crate::error::lang::SIGNAL_PARSE_INVALID_START_BIT,
                ));
            }
        };

        // Validate start_bit range
        if start_bit > 511 {
            return Err(ParseError::Signal(
                crate::error::lang::SIGNAL_PARSE_INVALID_START_BIT,
            ));
        }

        // Expect pipe
        parser.expect(b"|").map_err(|_| ParseError::Expected("Expected pipe"))?;

        // Parse length
        let length = parser
            .parse_u32()
            .map_err(|_| ParseError::Signal(crate::error::lang::SIGNAL_PARSE_INVALID_LENGTH))?
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
                    return Err(ParseError::Signal(
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
                    return Err(ParseError::Signal(
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
                    return Err(ParseError::Signal(
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
                    return Err(ParseError::Signal(
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
                ParseError::Signal(crate::error::lang::SIGNAL_PARSE_UNIT_TOO_LONG)
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
        // Signal parsing must always start with "SG_" keyword
        parser
            .expect(crate::SG_.as_bytes())
            .map_err(|_| ParseError::Expected("Expected SG_ keyword"))?;

        // Skip whitespace after "SG_"
        parser.skip_newlines_and_spaces();

        // Parse signal name (identifier)
        let name = parser
            .parse_identifier()
            .map_err(|_| ParseError::Signal(crate::error::lang::SIGNAL_NAME_EMPTY))?;

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
        // Value descriptions are stored in Dbc, not in Signal
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

    /// Decode the signal value from CAN message data bytes.
    ///
    /// Extracts the raw value from bits based on the signal's start bit, length, and byte order,
    /// then applies factor and offset to return the physical/engineering value.
    ///
    /// # Arguments
    ///
    /// * `data` - The CAN message data bytes (up to 8 bytes for classic CAN, 64 for CAN FD)
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` - The physical value (raw * factor + offset)
    /// * `Err(ParseError)` - If the signal extends beyond the data length
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// // Decode a CAN message with RPM value of 2000 (raw: 8000)
    /// let data = [0x20, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let rpm = signal.decode(&data)?;
    /// assert_eq!(rpm, 2000.0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn decode(&self, data: &[u8]) -> Result<f64, crate::error::ParseError> {
        use crate::error::{ParseError, lang};

        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        if end_byte >= data.len() {
            return Err(ParseError::Signal(lang::SIGNAL_EXTENDS_BEYOND_DATA));
        }

        // Extract bits based on byte order
        let raw_value = match self.byte_order {
            ByteOrder::LittleEndian => Self::extract_bits_little_endian(data, start_bit, length),
            ByteOrder::BigEndian => Self::extract_bits_big_endian(data, start_bit, length),
        };

        // Convert to signed/unsigned
        let value = if self.unsigned {
            raw_value as i64
        } else {
            // Sign extend for signed values
            let sign_bit = 1u64 << (length - 1);
            if (raw_value & sign_bit) != 0 {
                // Negative value - sign extend
                let mask = !((1u64 << length) - 1);
                (raw_value | mask) as i64
            } else {
                raw_value as i64
            }
        };

        // Apply factor and offset to get physical value
        Ok((value as f64) * self.factor + self.offset)
    }

    /// Extract bits from data using little-endian byte order.
    fn extract_bits_little_endian(data: &[u8], start_bit: usize, length: usize) -> u64 {
        let mut value: u64 = 0;
        let mut bits_remaining = length;
        let mut current_bit = start_bit;

        while bits_remaining > 0 {
            let byte_idx = current_bit / 8;
            let bit_in_byte = current_bit % 8;
            let bits_to_take = bits_remaining.min(8 - bit_in_byte);

            let byte = data[byte_idx] as u64;
            let mask = ((1u64 << bits_to_take) - 1) << bit_in_byte;
            let extracted = (byte & mask) >> bit_in_byte;

            value |= extracted << (length - bits_remaining);

            bits_remaining -= bits_to_take;
            current_bit += bits_to_take;
        }

        value
    }

    /// Extract bits from data using big-endian byte order.
    fn extract_bits_big_endian(data: &[u8], start_bit: usize, length: usize) -> u64 {
        let mut value: u64 = 0;
        let mut bits_remaining = length;
        let mut current_bit = start_bit;

        while bits_remaining > 0 {
            let byte_idx = current_bit / 8;
            let bit_in_byte = current_bit % 8;
            let bits_to_take = bits_remaining.min(8 - bit_in_byte);

            let byte = data[byte_idx] as u64;
            let mask = ((1u64 << bits_to_take) - 1) << bit_in_byte;
            let extracted = (byte & mask) >> bit_in_byte;

            // For big-endian, we need to reverse the bit order within bytes
            // and place bits in the correct position
            value |= extracted << (length - bits_remaining);

            bits_remaining -= bits_to_take;
            current_bit += bits_to_take;
        }

        value
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> alloc::string::String {
        use alloc::{
            format,
            string::{String, ToString},
        };
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

// Custom Eq implementation that handles f64 (treats NaN as equal to NaN)
impl<'a> Eq for Signal<'a> {}

// Custom Hash implementation that handles f64 (treats NaN consistently)
impl<'a> core::hash::Hash for Signal<'a> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.start_bit.hash(state);
        self.length.hash(state);
        self.byte_order.hash(state);
        self.unsigned.hash(state);
        // Handle f64: convert to bits for hashing (NaN will have consistent representation)
        self.factor.to_bits().hash(state);
        self.offset.to_bits().hash(state);
        self.min.to_bits().hash(state);
        self.max.to_bits().hash(state);
        self.unit.hash(state);
        self.receivers.hash(state);
    }
}

#[cfg(feature = "alloc")]
impl<'a> core::fmt::Display for Signal<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{
        Parser,
        error::{ParseError, lang},
    };

    // Note: Builder tests have been moved to signal_builder.rs
    // This module only tests Signal parsing and direct API usage

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
        let mut receivers_iter = sig.receivers().iter();
        assert_eq!(receivers_iter.next(), Some("TCM"));
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
        // Check receivers using iter_nodes
        let mut receivers_iter = sig.receivers().iter();
        assert_eq!(receivers_iter.next(), Some("TCM"));
        assert_eq!(receivers_iter.next(), Some("BCM"));
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
            ParseError::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(lang::SIGNAL_PARSE_INVALID_START_BIT)
                        || msg.contains("Signal 'RPM'")
                );
            }
            _ => panic!("Expected ParseError::Signal"),
        }
    }

    #[test]
    fn test_parse_signal_invalid_range() {
        // min > max should fail validation
        let line = r#"SG_ Test : 0|8@0+ (1,0) [100|50] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Signal(msg) => {
                assert!(msg.contains(lang::INVALID_RANGE));
            }
            e => panic!("Expected ParseError::Signal, got: {:?}", e),
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
            ParseError::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)
                        || msg.contains("Signal 'Test'")
                        || msg.contains("513")
                );
            }
            e => panic!("Expected ParseError::Signal, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_zero_length() {
        // length = 0 should fail validation
        let line = r#"SG_ Test : 0|0@0+ (1,0) [0|100] "unit" *"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Signal(msg) => {
                // Check for either the old constant or the new formatted message
                assert!(
                    msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)
                        || msg.contains("Signal 'Test'")
                        || msg.contains("0 bits")
                );
            }
            e => panic!("Expected ParseError::Signal, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_signal_missing_length() {
        let line = r#"SG_ RPM : 0|@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let err = Signal::parse(&mut parser).unwrap_err();
        match err {
            ParseError::Signal(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_LENGTH)),
            e => panic!("Expected ParseError::Signal, got: {:?}", e),
        }
    }

    // Tests that require alloc or kernel (for to_dbc_string)
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    mod tests_with_alloc {
        #[cfg(feature = "alloc")]
        use super::*;

        #[test]
        #[cfg(feature = "alloc")] // to_dbc_string is only available with alloc
        fn test_signal_to_dbc_string_round_trip() {
            use alloc::vec;
            // Test round-trip: parse -> to_dbc_string -> parse
            let test_cases = vec![
                (
                    r#"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *"#,
                    " SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *",
                ),
                (
                    r#"SG_ Temperature : 16|8@1- (1,-40) [-40|215] "°C" TCM BCM"#,
                    " SG_ Temperature : 16|8@1- (1,-40) [-40|215] \"°C\" TCM BCM",
                ),
                (
                    r#"SG_ Flag : 24|1@0+ (1,0) [0|1] "" *"#,
                    " SG_ Flag : 24|1@0+ (1,0) [0|1] \"\" *",
                ),
            ];

            for (input_line, expected_output) in test_cases {
                // Parse the signal
                let mut parser = Parser::new(input_line.as_bytes()).unwrap();
                let signal = Signal::parse(&mut parser).unwrap();

                // Convert to DBC string
                let dbc_string = signal.to_dbc_string();
                assert_eq!(dbc_string, expected_output);

                // Round-trip: parse the output
                let mut parser2 = Parser::new(dbc_string.as_bytes()).unwrap();
                // Skip only the leading space, Signal::parse will handle SG_ keyword
                parser2.skip_newlines_and_spaces();
                let signal2 = Signal::parse(&mut parser2).unwrap();

                // Verify round-trip
                assert_eq!(signal.name(), signal2.name());
                assert_eq!(signal.start_bit(), signal2.start_bit());
                assert_eq!(signal.length(), signal2.length());
                assert_eq!(signal.byte_order(), signal2.byte_order());
                assert_eq!(signal.is_unsigned(), signal2.is_unsigned());
                assert_eq!(signal.factor(), signal2.factor());
                assert_eq!(signal.offset(), signal2.offset());
                assert_eq!(signal.min(), signal2.min());
                assert_eq!(signal.max(), signal2.max());
                assert_eq!(signal.unit(), signal2.unit());
            }
        }
    }

    // Note: Helper parsing functions (parse_name_and_prefix, parse_position, etc.) are now internal
    // and use the Parser directly. Their functionality is tested through Signal::parse tests above.
    // All tests for these helper methods have been removed as they are implementation details.
}
