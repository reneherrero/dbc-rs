use crate::{Error, error::messages};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

/// Represents a signal within a CAN message.
///
/// A signal defines how a portion of the message payload should be
/// interpreted, including bit position, length, scaling factors,
/// and value ranges.
///
/// # Examples
///
/// ```rust
/// use dbc_rs::{Signal, ByteOrder, Receivers};
///
/// let signal = Signal::builder()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .byte_order(ByteOrder::BigEndian)
///     .unsigned(true)
///     .factor(0.25)
///     .offset(0.0)
///     .min(0.0)
///     .max(8000.0)
///     .unit("rpm")
///     .receivers(Receivers::Broadcast)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    name: Box<str>,
    start_bit: u8,
    length: u8,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<Box<str>>,
    receivers: Receivers,
}

/// Byte order (endianness) for signal encoding.
///
/// Determines how multi-byte signals are encoded within the CAN message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Little-endian byte order (Intel format).
    LittleEndian = 0,
    /// Big-endian byte order (Motorola format).
    BigEndian = 1,
}

/// Receiver specification for a signal.
///
/// Defines which nodes on the CAN bus should receive and process this signal.
#[derive(Debug, Clone, PartialEq)]
pub enum Receivers {
    /// Signal is broadcast to all nodes.
    Broadcast,
    /// Signal is sent to specific nodes.
    Nodes(Vec<Box<str>>),
    /// Signal has no receivers specified.
    None,
}

impl Signal {
    /// Validate signal parameters
    fn validate(name: &str, start_bit: u8, length: u8, min: f64, max: f64) -> Result<(), Error> {
        if name.trim().is_empty() {
            return Err(Error::Signal(messages::SIGNAL_NAME_EMPTY.to_string()));
        }

        // Validate length: must be between 1 and 64 bits
        if length == 0 {
            return Err(Error::Signal(messages::SIGNAL_LENGTH_TOO_SMALL.to_string()));
        }
        if length > 64 {
            return Err(Error::Signal(messages::SIGNAL_LENGTH_TOO_LARGE.to_string()));
        }

        // Validate start_bit + length doesn't exceed 64 (CAN message max size)
        let end_bit = start_bit as u16 + length as u16;
        if end_bit > 64 {
            return Err(Error::Signal(messages::signal_extends_beyond_can(
                start_bit, length, end_bit,
            )));
        }

        // Validate min <= max
        if min > max {
            return Err(Error::Signal(messages::invalid_range(min, max)));
        }

        Ok(())
    }

    /// Create a new Signal with the given parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `name` is empty
    /// - `length` is 0 or greater than 64
    /// - `start_bit + length` exceeds 64 (signal would overflow CAN message)
    /// - `min > max` (invalid range)
    ///
    /// This is an internal constructor. For public API usage, use [`Signal::builder()`] instead.
    #[allow(clippy::too_many_arguments)] // Internal method, builder pattern is the public API
    pub(crate) fn new(
        name: impl AsRef<str>,
        start_bit: u8,
        length: u8,
        byte_order: ByteOrder,
        unsigned: bool,
        factor: f64,
        offset: f64,
        min: f64,
        max: f64,
        unit: Option<impl AsRef<str>>,
        receivers: Receivers,
    ) -> Result<Self, Error> {
        let name_str = name.as_ref();
        Self::validate(name_str, start_bit, length, min, max)?;

        Ok(Self {
            name: name_str.into(),
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit: unit.map(|u| u.as_ref().into()),
            receivers,
        })
    }

    /// Create a new builder for constructing a `Signal`
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::{Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::builder()
    ///     .name("RPM")
    ///     .start_bit(0)
    ///     .length(16)
    ///     .byte_order(ByteOrder::BigEndian)
    ///     .unsigned(true)
    ///     .factor(0.25)
    ///     .offset(0.0)
    ///     .min(0.0)
    ///     .max(8000.0)
    ///     .unit("rpm")
    ///     .receivers(Receivers::Broadcast)
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn builder() -> SignalBuilder {
        SignalBuilder::new()
    }

    pub(super) fn parse(line: &str) -> Result<Self, Error> {
        // INSERT_YOUR_CODE

        // Trim and check for SG_
        let line = line.trim_start();
        let line = line
            .strip_prefix("SG_")
            .or_else(|| line.strip_prefix("SG_"))
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_EXPECTED_SG.to_string()))?;
        let line = line.trim();

        // name until ':'
        let (name, rest) = match line.split_once(':') {
            Some((n, r)) => (n.trim(), r.trim()),
            None => {
                return Err(Error::Signal(
                    messages::SIGNAL_PARSE_MISSING_COLON.to_string(),
                ));
            }
        };

        // startBit|length@byteOrderSign
        // e.g. 0|16@0+
        let mut rest = rest;
        let mut tokens = rest.splitn(2, ' ');
        let pos = tokens
            .next()
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_MISSING_POSITION.to_string()))?
            .trim();
        rest = tokens
            .next()
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_MISSING_REST.to_string()))?
            .trim();

        // e.g. 0|16@0+
        let (bitlen, bosign) = pos
            .split_once('@')
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_EXPECTED_AT.to_string()))?;
        let (startbit, length) = bitlen
            .split_once('|')
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_EXPECTED_PIPE.to_string()))?;

        let start_bit: u8 = startbit
            .trim()
            .parse()
            .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_START_BIT.to_string()))?;
        let length: u8 = length
            .trim()
            .parse()
            .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_LENGTH.to_string()))?;

        // Parse byte order and sign
        let bosign = bosign.trim();
        let (byte_order, unsigned) = {
            let mut chars = bosign.chars();
            let bo = chars.next().ok_or_else(|| {
                Error::Signal(messages::SIGNAL_PARSE_MISSING_BYTE_ORDER.to_string())
            })?;
            let sign = chars
                .next()
                .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_MISSING_SIGN.to_string()))?;
            let byte_order = match bo {
                '0' => ByteOrder::LittleEndian,
                '1' => ByteOrder::BigEndian,
                _ => return Err(Error::Signal(messages::unknown_byte_order(bo))),
            };
            let unsigned = match sign {
                '+' => true,
                '-' => false,
                _ => return Err(Error::Signal(messages::unknown_sign(sign))),
            };
            (byte_order, unsigned)
        };

        // Now next token: (factor,offset)
        let rest = rest.trim_start();
        let (f_and_rest, rest) = match rest.trim_start().split_once(')') {
            Some((f, r)) => (f, r),
            None => {
                return Err(Error::Signal(
                    messages::SIGNAL_PARSE_MISSING_CLOSING_PAREN.to_string(),
                ));
            }
        };
        let f_and_rest = f_and_rest.trim_start();
        let f_and_rest = f_and_rest.strip_prefix('(').ok_or_else(|| {
            Error::Signal(messages::SIGNAL_PARSE_MISSING_OPENING_PAREN.to_string())
        })?;
        let (factor_str, offset_str) = f_and_rest
            .split_once(',')
            .ok_or_else(|| Error::Signal(messages::SIGNAL_PARSE_MISSING_COMMA.to_string()))?;
        let (factor_str, offset_str) = (factor_str.trim(), offset_str.trim());
        let factor: f64 = if factor_str.is_empty() {
            0.
        } else {
            factor_str
                .parse()
                .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_FACTOR.to_string()))?
        };
        let offset: f64 = if offset_str.is_empty() {
            0.
        } else {
            offset_str
                .parse()
                .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_OFFSET.to_string()))?
        };
        let rest = rest.trim_start();

        // Next token: [min|max]
        let (minmax, rest) = match rest.split_once(']') {
            Some((m, r)) => (m, r),
            None => {
                return Err(Error::Signal(
                    messages::SIGNAL_PARSE_MISSING_CLOSING_BRACKET.to_string(),
                ));
            }
        };
        let minmax = minmax.trim_start().strip_prefix('[').ok_or_else(|| {
            Error::Signal(messages::SIGNAL_PARSE_MISSING_OPENING_BRACKET.to_string())
        })?;
        let (min_str, max_str) = minmax.split_once('|').ok_or_else(|| {
            Error::Signal(messages::SIGNAL_PARSE_MISSING_PIPE_IN_RANGE.to_string())
        })?;
        let (min_str, max_str) = (min_str.trim(), max_str.trim());
        let min: f64 = if min_str.is_empty() {
            0.
        } else {
            min_str
                .parse()
                .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_MIN.to_string()))?
        };
        let max: f64 = if max_str.is_empty() {
            0.
        } else {
            max_str
                .parse()
                .map_err(|_| Error::Signal(messages::SIGNAL_PARSE_INVALID_MAX.to_string()))?
        };
        let mut rest = rest.trim_start();

        // Now: unit in double quotes
        if !rest.starts_with('"') {
            return Err(Error::Signal(
                messages::SIGNAL_PARSE_EXPECTED_UNIT_QUOTE.to_string(),
            ));
        }
        rest = &rest[1..];
        // Pre-allocate unit string (most units are short, 1-10 chars)
        let mut unit_str = String::with_capacity(10);
        for c in rest.chars() {
            if c == '"' {
                break;
            }
            unit_str.push(c);
        }
        // Advance past the closing quote in rest
        rest = rest[unit_str.len()..].trim_start();
        if rest.starts_with('"') {
            rest = &rest[1..];
        }
        let unit = if unit_str.is_empty() {
            None
        } else {
            Some(unit_str.into_boxed_str())
        };
        let rest = rest.trim_start();

        // Receivers
        let receivers = if rest.is_empty() {
            Receivers::None
        } else if rest == "*" {
            Receivers::Broadcast
        } else {
            // Pre-allocate receivers Vec (most signals have 1-3 receivers)
            let nodes: Vec<Box<str>> = rest.split_whitespace().map(|s| s.into()).collect();
            if nodes.is_empty() {
                Receivers::None
            } else {
                Receivers::Nodes(nodes)
            }
        };

        // Validate the parsed signal using the same validation as new()
        Self::validate(name, start_bit, length, min, max)?;

        Ok(Signal {
            name: name.into(),
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

    /// Get the signal name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the starting bit position within the message.
    #[inline]
    pub fn start_bit(&self) -> u8 {
        self.start_bit
    }

    /// Get the signal length in bits.
    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    /// Get the byte order (endianness) of the signal.
    #[inline]
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    /// Check if the signal is unsigned.
    #[inline]
    pub fn is_unsigned(&self) -> bool {
        self.unsigned
    }

    /// Get the scaling factor for converting raw value to physical value.
    #[inline]
    pub fn factor(&self) -> f64 {
        self.factor
    }

    /// Get the offset for converting raw value to physical value.
    #[inline]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// Get the minimum physical value.
    #[inline]
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Get the maximum physical value.
    #[inline]
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Get the unit string, if present.
    #[inline]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|s| s.as_ref())
    }

    /// Get the receiver specification for this signal.
    #[inline]
    pub fn receivers(&self) -> &Receivers {
        &self.receivers
    }

    /// Format signal in DBC file format (e.g., ` SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *`)
    ///
    /// Useful for debugging and visualization of the signal in DBC format.
    /// Note: The leading space is included to match DBC file formatting conventions.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::{Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::builder()
    ///     .name("RPM")
    ///     .start_bit(0)
    ///     .length(16)
    ///     .byte_order(ByteOrder::BigEndian)
    ///     .unsigned(true)
    ///     .factor(0.25)
    ///     .offset(0.0)
    ///     .min(0.0)
    ///     .max(8000.0)
    ///     .unit("rpm")
    ///     .receivers(Receivers::Broadcast)
    ///     .build()?;
    /// assert_eq!(signal.to_dbc_string(), " SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" *");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn to_dbc_string(&self) -> String {
        use alloc::format;

        let mut result = String::with_capacity(100); // Pre-allocate reasonable capacity

        result.push_str(" SG_ ");
        result.push_str(self.name());
        result.push_str(" : ");
        result.push_str(&self.start_bit().to_string());
        result.push('|');
        result.push_str(&self.length().to_string());
        result.push('@');

        // Byte order: 0 for LittleEndian, 1 for BigEndian
        match self.byte_order() {
            ByteOrder::LittleEndian => result.push('0'),
            ByteOrder::BigEndian => result.push('1'),
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
            Receivers::Nodes(nodes) => {
                if !nodes.is_empty() {
                    result.push(' ');
                    for (i, node) in nodes.iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(node.as_ref());
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

/// Builder for constructing a `Signal` with a fluent API
///
/// This builder provides a more ergonomic way to construct `Signal` instances,
/// especially when many parameters are optional or have sensible defaults.
///
/// # Examples
///
/// ```
/// use dbc_rs::{Signal, ByteOrder, Receivers};
///
/// // Minimal signal with defaults
/// let signal = Signal::builder()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .build()?;
///
/// // Full signal with all parameters
/// let signal = Signal::builder()
///     .name("Temperature")
///     .start_bit(16)
///     .length(8)
///     .byte_order(ByteOrder::LittleEndian)
///     .unsigned(false)
///     .factor(1.0)
///     .offset(-40.0)
///     .min(-40.0)
///     .max(215.0)
///     .unit("°C")
///     .receivers(Receivers::Broadcast)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct SignalBuilder {
    name: Option<Box<str>>,
    start_bit: Option<u8>,
    length: Option<u8>,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<Box<str>>,
    receivers: Receivers,
}

impl SignalBuilder {
    fn new() -> Self {
        Self {
            name: None,
            start_bit: None,
            length: None,
            byte_order: ByteOrder::BigEndian,
            unsigned: true,
            factor: 1.0,
            offset: 0.0,
            min: 0.0,
            max: 0.0,
            unit: None,
            receivers: Receivers::None,
        }
    }

    /// Set the signal name (required)
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().into());
        self
    }

    /// Set the start bit position (required)
    pub fn start_bit(mut self, start_bit: u8) -> Self {
        self.start_bit = Some(start_bit);
        self
    }

    /// Set the signal length in bits (required)
    pub fn length(mut self, length: u8) -> Self {
        self.length = Some(length);
        self
    }

    /// Set the byte order (default: `BigEndian`)
    pub fn byte_order(mut self, byte_order: ByteOrder) -> Self {
        self.byte_order = byte_order;
        self
    }

    /// Set whether the signal is unsigned (default: `true`)
    pub fn unsigned(mut self, unsigned: bool) -> Self {
        self.unsigned = unsigned;
        self
    }

    /// Set the scaling factor (default: `1.0`)
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Set the offset value (default: `0.0`)
    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    /// Set the minimum value (default: `0.0`)
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value (default: `0.0`)
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Set the unit string (optional)
    pub fn unit(mut self, unit: impl AsRef<str>) -> Self {
        self.unit = Some(unit.as_ref().into());
        self
    }

    /// Clear the unit (set to `None`)
    pub fn no_unit(mut self) -> Self {
        self.unit = None;
        self
    }

    /// Set the receivers (default: `Receivers::None`)
    pub fn receivers(mut self, receivers: Receivers) -> Self {
        self.receivers = receivers;
        self
    }

    /// Validate the current builder state
    ///
    /// This method performs the same validation as `Signal::validate()` but on the
    /// builder's current state. Useful for checking validity before calling `build()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields (`name`, `start_bit`, `length`) are missing
    /// - Validation fails (same as `Signal::validate()`)
    pub fn validate(&self) -> Result<(), Error> {
        let name = self
            .name
            .as_ref()
            .ok_or_else(|| Error::Signal(messages::SIGNAL_NAME_EMPTY.to_string()))?;
        let start_bit = self
            .start_bit
            .ok_or_else(|| Error::Signal(messages::SIGNAL_START_BIT_REQUIRED.to_string()))?;
        let length = self
            .length
            .ok_or_else(|| Error::Signal(messages::SIGNAL_LENGTH_REQUIRED.to_string()))?;

        Signal::validate(name.as_ref(), start_bit, length, self.min, self.max)
    }

    /// Build the `Signal` from the builder
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields (`name`, `start_bit`, `length`) are missing
    /// - Validation fails (same validation logic as the internal constructor)
    pub fn build(self) -> Result<Signal, Error> {
        let name = self
            .name
            .ok_or_else(|| Error::Signal(messages::SIGNAL_NAME_EMPTY.to_string()))?;
        let start_bit = self
            .start_bit
            .ok_or_else(|| Error::Signal(messages::SIGNAL_START_BIT_REQUIRED.to_string()))?;
        let length = self
            .length
            .ok_or_else(|| Error::Signal(messages::SIGNAL_LENGTH_REQUIRED.to_string()))?;

        Signal::new(
            name.as_ref(),
            start_bit,
            length,
            self.byte_order,
            self.unsigned,
            self.factor,
            self.offset,
            self.min,
            self.max,
            self.unit.as_ref().map(|s| s.as_ref()),
            self.receivers,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use crate::error::lang;
    extern crate std;

    #[test]
    fn test_signal_new_valid() {
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            0.25,
            0.0,
            0.0,
            8000.0,
            Some("rpm" as &str),
            Receivers::Broadcast,
        )
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
    fn test_signal_new_empty_name() {
        let result = Signal::new(
            "",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_NAME_EMPTY)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_new_zero_length() {
        let result = Signal::new(
            "Test",
            0,
            0,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_new_length_too_large() {
        let result = Signal::new(
            "Test",
            0,
            65,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_new_overflow() {
        let result = Signal::new(
            "Test",
            60,
            10,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_CAN.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_new_invalid_range() {
        let result = Signal::new(
            "Test",
            0,
            8,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            100.0,
            50.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_INVALID_RANGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_new_max_boundary() {
        // Test that 64 bits at position 0 is valid
        let signal = Signal::new(
            "FullMessage",
            0,
            64,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        assert_eq!(signal.length(), 64);
    }

    #[test]
    fn test_signal_new_with_receivers() {
        let nodes = vec!["ECM".into(), "TCM".into()];
        let unit: Option<&str> = Some("°C");
        let signal = Signal::new(
            "TestSignal",
            8,
            16,
            ByteOrder::LittleEndian,
            false,
            0.1,
            -40.0,
            -40.0,
            215.0,
            unit,
            Receivers::Nodes(nodes),
        )
        .unwrap();
        assert_eq!(signal.name(), "TestSignal");
        assert!(!signal.is_unsigned());
        assert_eq!(signal.unit(), Some("°C"));
        match signal.receivers() {
            Receivers::Nodes(n) => assert_eq!(n.len(), 2),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_parse_valid_signal() {
        let line = r#" SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let sig = Signal::parse(line).unwrap();
        assert_eq!(sig.name(), "RPM");
        assert_eq!(sig.start_bit(), 0);
        assert_eq!(sig.length(), 16);
        assert_eq!(sig.byte_order(), ByteOrder::LittleEndian);
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 0.25);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 8000.);
        assert_eq!(sig.unit(), Some("rpm"));
        assert_eq!(sig.receivers(), &Receivers::Nodes(vec!["TCM".into()]));
    }

    #[test]
    fn test_parse_signal_with_empty_unit_and_broadcast() {
        let line = r#" SG_ ABSActive : 16|1@0+ (1,0) [0|1] "" *"#;
        let sig = Signal::parse(line).unwrap();
        assert_eq!(sig.name(), "ABSActive");
        assert_eq!(sig.start_bit(), 16);
        assert_eq!(sig.length(), 1);
        assert_eq!(sig.byte_order(), ByteOrder::LittleEndian);
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
        let line = r#" SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM BCM"#;
        let sig = Signal::parse(line).unwrap();
        assert_eq!(sig.name(), "Temperature");
        assert_eq!(sig.start_bit(), 16);
        assert_eq!(sig.length(), 8);
        assert_eq!(sig.byte_order(), ByteOrder::LittleEndian);
        assert!(!sig.is_unsigned());
        assert_eq!(sig.factor(), 1.);
        assert_eq!(sig.offset(), -40.);
        assert_eq!(sig.min(), -40.);
        assert_eq!(sig.max(), 215.);
        assert_eq!(sig.unit(), Some("°C"));
        assert_eq!(
            sig.receivers(),
            &Receivers::Nodes(vec!["TCM".into(), "BCM".into()])
        );
    }

    #[test]
    fn test_parse_signal_with_percent_unit() {
        let line = r#" SG_ ThrottlePosition : 24|8@0+ (0.392157,0) [0|100] "%" *"#;
        let sig = Signal::parse(line).unwrap();
        assert_eq!(sig.name(), "ThrottlePosition");
        assert_eq!(sig.start_bit(), 24);
        assert_eq!(sig.length(), 8);
        assert_eq!(sig.byte_order(), ByteOrder::LittleEndian);
        assert!(sig.is_unsigned());
        assert_eq!(sig.factor(), 0.392157);
        assert_eq!(sig.offset(), 0.);
        assert_eq!(sig.min(), 0.);
        assert_eq!(sig.max(), 100.);
        assert_eq!(sig.unit(), Some("%"));
        assert_eq!(sig.receivers(), &Receivers::Broadcast);
    }

    #[test]
    fn test_parse_signal_missing_factors_and_limits() {
        // Should use default values where missing
        let line = r#" SG_ Simple : 10|4@0+ ( , ) [ | ] "" *"#;
        let sig = Signal::parse(line).unwrap();
        assert_eq!(sig.name(), "Simple");
        assert_eq!(sig.start_bit(), 10);
        assert_eq!(sig.length(), 4);
        assert_eq!(sig.byte_order(), ByteOrder::LittleEndian);
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
        let line = r#" SG_ RPM : |16@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_START_BIT)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_parse_signal_invalid_range() {
        // min > max should fail validation
        let line = r#" SG_ Test : 0|8@0+ (1,0) [100|50] "unit" *"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_INVALID_RANGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_parse_signal_overflow() {
        // start_bit + length > 64 should fail validation
        let line = r#" SG_ Test : 60|10@0+ (1,0) [0|100] "unit" *"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_CAN.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_parse_signal_length_too_large() {
        // length > 64 should fail validation
        let line = r#" SG_ Test : 0|65@0+ (1,0) [0|100] "unit" *"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_LARGE)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_parse_signal_zero_length() {
        // length = 0 should fail validation
        let line = r#" SG_ Test : 0|0@0+ (1,0) [0|100] "unit" *"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_LENGTH_TOO_SMALL)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_parse_signal_missing_length() {
        let line = r#" SG_ RPM : 0|@0+ (0.25,0) [0|8000] "rpm" TCM"#;
        let err = Signal::parse(line).unwrap_err();
        match err {
            Error::Signal(msg) => assert!(msg.contains(lang::SIGNAL_PARSE_INVALID_LENGTH)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_to_dbc_string() {
        // Test with Broadcast receiver
        let signal1 = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            0.25,
            0.0,
            0.0,
            8000.0,
            Some("rpm" as &str),
            Receivers::Broadcast,
        )
        .unwrap();
        assert_eq!(
            signal1.to_dbc_string(),
            " SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" *"
        );

        // Test with Nodes receiver
        let signal2 = Signal::new(
            "Temperature",
            16,
            8,
            ByteOrder::LittleEndian,
            false,
            1.0,
            -40.0,
            -40.0,
            215.0,
            Some("°C" as &str),
            Receivers::Nodes(vec!["TCM".into(), "BCM".into()]),
        )
        .unwrap();
        assert_eq!(
            signal2.to_dbc_string(),
            " SG_ Temperature : 16|8@0- (1,-40) [-40|215] \"°C\" TCM BCM"
        );

        // Test with None receiver and empty unit
        let signal3 = Signal::new(
            "Flag",
            24,
            1,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            1.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        assert_eq!(
            signal3.to_dbc_string(),
            " SG_ Flag : 24|1@1+ (1,0) [0|1] \"\""
        );
    }
}
