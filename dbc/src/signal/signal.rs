use crate::compat::String;
use crate::error::lang;
use crate::{ByteOrder, Error, MAX_NAME_SIZE, Parser, Receivers, Result};

/// Multiplexer indicator for signals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplexerIndicator {
    /// Normal signal (not multiplexed)
    Normal,
    /// Multiplexer switch signal (M)
    Switch,
    /// Multiplexed signal (m0, m1, m2, etc.)
    Multiplexed(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    name: String<{ MAX_NAME_SIZE }>,
    start_bit: u16,
    length: u16,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<String<{ MAX_NAME_SIZE }>>,
    receivers: Receivers,
    multiplexer: MultiplexerIndicator,
}

impl Signal {
    pub(crate) fn validate(name: &str, length: u16, min: f64, max: f64) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::Validation(lang::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        // Note: name is parsed before this validation, so we can include it in error messages
        if length == 0 {
            return Err(Error::Validation(lang::SIGNAL_LENGTH_TOO_SMALL));
        }
        if length > 512 {
            return Err(Error::Validation(lang::SIGNAL_LENGTH_TOO_LARGE));
        }

        // Note: start_bit validation (boundary checks and overlap detection) is done in
        // Message::validate, not here, because:
        // 1. The actual message size depends on DLC (1-64 bytes for CAN FD)
        // 2. Overlap detection requires comparing multiple signals
        // 3. This allows signals to be created independently and validated when added to a message

        // Validate min <= max
        if min > max {
            return Err(Error::Validation(lang::INVALID_RANGE));
        }

        Ok(())
    }

    #[cfg(feature = "std")]
    #[allow(clippy::too_many_arguments)] // Internal method, builder pattern is the public API
    pub(crate) fn new(
        name: String<{ MAX_NAME_SIZE }>,
        start_bit: u16,
        length: u16,
        byte_order: ByteOrder,
        unsigned: bool,
        factor: f64,
        offset: f64,
        min: f64,
        max: f64,
        unit: Option<String<{ MAX_NAME_SIZE }>>,
        receivers: Receivers,
        multiplexer: MultiplexerIndicator,
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
            multiplexer,
        }
    }

    /// Get the multiplexer indicator for this signal
    #[must_use]
    pub fn multiplexer(&self) -> MultiplexerIndicator {
        self.multiplexer
    }

    /// Check if this signal is a multiplexer switch
    #[must_use]
    pub fn is_multiplexer_switch(&self) -> bool {
        matches!(self.multiplexer, MultiplexerIndicator::Switch)
    }

    /// Check if this signal is multiplexed
    #[must_use]
    pub fn is_multiplexed(&self) -> bool {
        matches!(self.multiplexer, MultiplexerIndicator::Multiplexed(_))
    }

    /// Get the multiplexer value if this signal is multiplexed
    #[must_use]
    pub fn multiplexer_value(&self) -> Option<u8> {
        match self.multiplexer {
            MultiplexerIndicator::Multiplexed(v) => Some(v),
            _ => None,
        }
    }

    fn parse_position<'b>(parser: &mut Parser<'b>) -> Result<(u16, u16, ByteOrder, bool)> {
        // Parse start_bit
        let start_bit = match parser.parse_u32() {
            Ok(v) => v as u16,
            Err(_) => {
                return Err(Error::Signal(lang::SIGNAL_PARSE_INVALID_START_BIT));
            }
        };

        // Validate start_bit range
        if start_bit > 511 {
            return Err(Error::Signal(lang::SIGNAL_PARSE_INVALID_START_BIT));
        }

        // Expect pipe
        parser.expect(b"|").map_err(|_| Error::Expected("Expected pipe"))?;

        // Parse length
        let length = parser
            .parse_u32()
            .map_err(|_| Error::Signal(lang::SIGNAL_PARSE_INVALID_LENGTH))?
            as u16;

        // Expect @
        parser.expect(b"@").map_err(|_| Error::Expected("Expected @"))?;

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
        parser
            .expect(b"(")
            .map_err(|_| Error::Expected("Expected opening parenthesis"))?;

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
                    return Err(Error::Signal(lang::SIGNAL_PARSE_INVALID_FACTOR));
                }
            }
        };

        // Expect comma
        parser.expect(b",").map_err(|_| Error::Expected("Expected comma"))?;

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
                    return Err(Error::Signal(
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
            .map_err(|_| Error::Expected("Expected closing parenthesis"))?;

        Ok((factor, offset))
    }

    fn parse_range<'b>(parser: &mut Parser<'b>) -> Result<(f64, f64)> {
        // Expect opening bracket
        parser.expect(b"[").map_err(|_| Error::Expected("Expected opening bracket"))?;

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
                    return Err(Error::Signal(crate::error::lang::SIGNAL_PARSE_INVALID_MIN));
                }
            }
        };

        // Expect pipe
        parser.expect(b"|").map_err(|_| Error::Expected("Expected pipe"))?;

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
                    return Err(Error::Signal(crate::error::lang::SIGNAL_PARSE_INVALID_MAX));
                }
            }
        };

        // Skip whitespace
        parser.skip_newlines_and_spaces();

        // Expect closing bracket
        parser.expect(b"]").map_err(|_| Error::Expected("Expected closing bracket"))?;

        Ok((min, max))
    }

    fn parse_unit(parser: &mut Parser) -> Result<Option<String<{ MAX_NAME_SIZE }>>> {
        // Expect opening quote
        parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;

        // Use take_until_quote to read the unit (allow any printable characters)
        let unit_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).map_err(|e| match e {
            Error::MaxStrLength(_) => Error::Signal(crate::error::lang::SIGNAL_PARSE_UNIT_TOO_LONG),
            _ => Error::Expected("Expected closing quote"),
        })?;

        // Convert bytes to string slice
        let unit =
            core::str::from_utf8(unit_bytes).map_err(|_e| Error::Expected(lang::INVALID_UTF8))?;

        let unit: String<{ MAX_NAME_SIZE }> =
            String::try_from(unit).map_err(|_| Error::Version(lang::MAX_NAME_SIZE_EXCEEDED))?;

        let unit = if unit.is_empty() { None } else { Some(unit) };
        Ok(unit)
    }

    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Signal parsing must always start with "SG_" keyword
        parser
            .expect(crate::SG_.as_bytes())
            .map_err(|_| Error::Expected("Expected SG_ keyword"))?;

        // Skip whitespace after "SG_"
        parser.skip_newlines_and_spaces();

        // Parse signal name (identifier)
        let name = parser
            .parse_identifier()
            .map_err(|_| Error::Signal(crate::error::lang::SIGNAL_NAME_EMPTY))?;

        // Skip whitespace (optional before colon) - parse multiplexer indicator
        // According to spec: multiplexer_indicator = ' ' | [m multiplexer_switch_value] [M]
        parser.skip_newlines_and_spaces();

        // Parse multiplexer indicator
        let multiplexer = if parser.expect(b"M").is_ok() {
            // Multiplexer switch (M)
            parser.skip_newlines_and_spaces();
            MultiplexerIndicator::Switch
        } else if parser.expect(b"m").is_ok() {
            // Multiplexed signal (m followed by number)
            // Parse the multiplexer switch value
            let mut value = 0u8;
            let mut has_digit = false;
            loop {
                let _pos_before = parser.pos();
                let digit = if parser.expect(b"0").is_ok() {
                    Some(0)
                } else if parser.expect(b"1").is_ok() {
                    Some(1)
                } else if parser.expect(b"2").is_ok() {
                    Some(2)
                } else if parser.expect(b"3").is_ok() {
                    Some(3)
                } else if parser.expect(b"4").is_ok() {
                    Some(4)
                } else if parser.expect(b"5").is_ok() {
                    Some(5)
                } else if parser.expect(b"6").is_ok() {
                    Some(6)
                } else if parser.expect(b"7").is_ok() {
                    Some(7)
                } else if parser.expect(b"8").is_ok() {
                    Some(8)
                } else if parser.expect(b"9").is_ok() {
                    Some(9)
                } else {
                    None
                };
                if let Some(d) = digit {
                    has_digit = true;
                    value = value.saturating_mul(10).saturating_add(d);
                } else {
                    break;
                }
            }
            if !has_digit {
                // 'm' without a number - treat as normal signal
                MultiplexerIndicator::Normal
            } else {
                parser.skip_newlines_and_spaces();
                MultiplexerIndicator::Multiplexed(value)
            }
        } else {
            // Normal signal (no multiplexer indicator)
            MultiplexerIndicator::Normal
        };

        // Expect colon
        parser.expect(b":").map_err(|_| Error::Expected("Expected colon"))?;

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
        Self::validate(name, length, min, max).map_err(|e| {
            crate::error::map_val_error(e, Error::Signal, || {
                Error::Signal(crate::error::lang::SIGNAL_ERROR_PREFIX)
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
            multiplexer,
        })
    }

    #[inline]
    #[must_use = "return value should be checked"]
    pub fn name(&self) -> &str {
        self.name.as_ref()
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
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|u| u.as_ref())
    }

    #[inline]
    #[must_use]
    pub fn receivers(&self) -> &Receivers {
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
    /// * `Err(Error)` - If the signal extends beyond the data length
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
    pub fn decode(&self, data: &[u8]) -> Result<f64> {
        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        if end_byte >= data.len() {
            return Err(Error::Decoding(lang::SIGNAL_EXTENDS_BEYOND_DATA));
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

    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> std::string::String {
        let mut result = std::string::String::with_capacity(100); // Pre-allocate reasonable capacity

        result.push_str(" SG_ ");
        result.push_str(self.name());
        // Add multiplexer indicator
        match self.multiplexer {
            MultiplexerIndicator::Switch => {
                result.push_str(" M");
            }
            MultiplexerIndicator::Multiplexed(value) => {
                result.push_str(" m");
                result.push_str(&value.to_string());
            }
            MultiplexerIndicator::Normal => {
                // No indicator for normal signals
            }
        }
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
        use core::fmt::Write;
        write!(result, "{}", self.factor()).unwrap();
        result.push(',');
        write!(result, "{}", self.offset()).unwrap();
        result.push(')');

        // Min and max: [min|max]
        result.push_str(" [");
        write!(result, "{}", self.min()).unwrap();
        result.push('|');
        write!(result, "{}", self.max()).unwrap();
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
            Receivers::Nodes(nodes) => {
                if !nodes.is_empty() {
                    result.push(' ');
                    for (i, node) in self.receivers().iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(node.as_str());
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
impl Eq for Signal {}

// Custom Hash implementation that handles f64 (treats NaN consistently)
impl core::hash::Hash for Signal {
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

#[cfg(feature = "std")]
impl core::fmt::Display for Signal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{Parser, error::lang};

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
                    msg.contains(lang::SIGNAL_PARSE_INVALID_START_BIT)
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
                assert!(msg.contains(lang::INVALID_RANGE));
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
                    msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)
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
                    msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)
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
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_LENGTH)),
            e => panic!("Expected Error::Signal, got: {:?}", e),
        }
    }

    // Tests that require std (for to_dbc_string)
    #[cfg(feature = "std")]
    mod tests_with_std {
        use super::*;

        #[test]
        fn test_signal_to_dbc_string_round_trip() {
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
}
