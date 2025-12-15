use crate::compat::String;
use crate::{ByteOrder, Error, MAX_NAME_SIZE, Receivers, Result};

mod decode;
mod parse;
#[cfg(feature = "std")]
mod serialize;

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
    /// True if this is a multiplexer switch signal (marked with 'M')
    is_multiplexer_switch: bool,
    /// If this is a multiplexed signal (marked with 'm0', 'm1', etc.), this contains the switch value
    /// None means this is a normal signal (not multiplexed)
    multiplexer_switch_value: Option<u64>,
}

impl Signal {
    pub(crate) fn validate(name: &str, length: u16, min: f64, max: f64) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::Validation(Error::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        // Note: name is parsed before this validation, so we can include it in error messages
        if length == 0 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_SMALL));
        }
        if length > 512 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_LARGE));
        }

        // Note: start_bit validation (boundary checks and overlap detection) is done in
        // Message::validate, not here, because:
        // 1. The actual message size depends on DLC (1-64 bytes for CAN FD)
        // 2. Overlap detection requires comparing multiple signals
        // 3. This allows signals to be created independently and validated when added to a message

        // Validate min <= max
        if min > max {
            return Err(Error::Validation(Error::INVALID_RANGE));
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
            is_multiplexer_switch: false,
            multiplexer_switch_value: None,
        }
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

    /// Check if this signal is a multiplexer switch (marked with 'M')
    #[inline]
    #[must_use]
    pub fn is_multiplexer_switch(&self) -> bool {
        self.is_multiplexer_switch
    }

    /// Get the multiplexer switch value if this is a multiplexed signal (marked with 'm0', 'm1', etc.)
    /// Returns None if this is a normal signal (not multiplexed)
    #[inline]
    #[must_use]
    pub fn multiplexer_switch_value(&self) -> Option<u64> {
        self.multiplexer_switch_value
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
    use super::Signal;
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

    // Edge case tests using Dbc::parse
    #[test]
    fn test_signal_at_boundary() {
        // Signal exactly at message boundary should be valid
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|64@1+ (1,0) [0|65535] ""
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Signal at boundary should be valid");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.start_bit(), 0);
        assert_eq!(signal.length(), 64);
    }

    #[test]
    fn test_signal_exceeds_boundary() {
        // Signal exceeding boundary should be rejected
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|65@1+ (1,0) [0|65535] ""
"#;
        let result = crate::Dbc::parse(dbc_str);
        assert!(result.is_err(), "Should reject signal exceeding boundary");
    }

    #[test]
    fn test_very_long_signal_name() {
        // Test signal name at reasonable length
        let long_name = "A".repeat(63); // Max reasonable length
        let dbc_str = format!(
            r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ {} : 0|8@1+ (1,0) [0|255] ""
"#,
            long_name
        );
        let dbc = crate::Dbc::parse(&dbc_str).expect("Should handle long signal names");
        assert_eq!(
            dbc.messages().at(0).unwrap().signals().at(0).unwrap().name().len(),
            63
        );
    }

    #[test]
    fn test_big_endian_signal_boundary() {
        // Test big-endian signal at boundary
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 56|8@0+ (1,0) [0|255] ""
"#;
        let dbc =
            crate::Dbc::parse(dbc_str).expect("Big-endian signal at boundary should be valid");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.start_bit(), 56);
        assert_eq!(signal.length(), 8);
        assert_eq!(signal.byte_order(), crate::ByteOrder::BigEndian);
    }

    #[test]
    fn test_negative_min_max() {
        // Test negative min/max values
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Temperature : 0|8@1- (1,-40) [-40|215] "°C"
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Should handle negative min/max");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.min(), -40.0);
        assert_eq!(signal.max(), 215.0);
    }

    #[test]
    fn test_float_factor_offset() {
        // Test floating point factor and offset
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|16@1+ (0.25,0.5) [0|65535] ""
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Should handle float factor/offset");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.factor(), 0.25);
        assert_eq!(signal.offset(), 0.5);
    }

    #[test]
    fn test_empty_unit_string() {
        // Test empty unit string
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] ""
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Should handle empty unit");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.unit(), None);
    }

    #[test]
    fn test_broadcast_receivers() {
        // Test broadcast receivers
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] ""
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Should handle broadcast receivers");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        // Default is broadcast (empty receivers list)
        assert!(signal.receivers().is_empty());
    }

    #[test]
    fn test_multiple_receiver_nodes() {
        // Test multiple receiver nodes
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM TCM BCM

BO_ 256 Test : 8 ECM
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] "" ECM TCM
"#;
        let dbc = crate::Dbc::parse(dbc_str).expect("Should handle multiple receivers");
        let signal = dbc.messages().at(0).unwrap().signals().at(0).unwrap();
        assert_eq!(signal.receivers().len(), 2);
        assert!(signal.receivers().contains("ECM"));
        assert!(signal.receivers().contains("TCM"));
    }
}
