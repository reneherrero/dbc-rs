use crate::{
    ByteOrder, Parser, Signal,
    error::{ParseError, ParseResult},
};

use super::Signals;

/// Represents a CAN message in a DBC file.
///
/// A `Message` contains:
/// - A unique ID (CAN identifier)
/// - A name
/// - A DLC (Data Length Code) specifying the message size in bytes
/// - A sender node (ECU that transmits this message)
/// - A collection of signals
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 EngineData : 8 ECM
///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// let message = dbc.messages().at(0).unwrap();
/// println!("Message: {} (ID: {})", message.name(), message.id());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message<'a> {
    id: u32,
    name: &'a str,
    dlc: u8,
    sender: &'a str,
    signals: Signals<'a>,
}

impl<'a> Message<'a> {
    #[allow(clippy::similar_names)] // physical_lsb and physical_msb are intentionally similar
    fn calculate_bit_range(start_bit: u16, length: u16, byte_order: ByteOrder) -> (u16, u16) {
        let start = start_bit;
        let len = length;

        match byte_order {
            ByteOrder::LittleEndian => {
                // Little-endian: start_bit is LSB, signal extends forward
                // Range: [start_bit, start_bit + length - 1]
                (start, start + len - 1)
            }
            ByteOrder::BigEndian => {
                // Big-endian: start_bit is MSB in big-endian numbering, signal extends backward
                // The big-endian bit numbering follows Vector convention:
                // be_bits = [7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23, 22, ...]
                // This means: BE bit 0 -> physical bit 7, BE bit 7 -> physical bit 0
                //            BE bit 8 -> physical bit 15, BE bit 15 -> physical bit 8
                // To find the physical bit range:
                // 1. Find the index of start_bit in the be_bits sequence
                // 2. MSB (physical) = be_bits[idx]
                // 3. LSB (physical) = be_bits[idx + length - 1]
                // We can calculate this directly:
                // For BE bit N: byte_num = N / 8, bit_in_byte = N % 8
                // Physical bit = byte_num * 8 + (7 - bit_in_byte)
                let byte_num = start / 8;
                let bit_in_byte = start % 8;
                let physical_msb = byte_num * 8 + (7 - bit_in_byte);

                // Calculate LSB: move forward (length - 1) positions in the BE sequence
                // BE bit (start + length - 1) maps to physical bit
                let lsb_be_bit = start + len - 1;
                let lsb_byte_num = lsb_be_bit / 8;
                let lsb_bit_in_byte = lsb_be_bit % 8;
                let physical_lsb = lsb_byte_num * 8 + (7 - lsb_bit_in_byte);

                // Ensure lsb <= msb (they should be in that order for big-endian)
                if physical_lsb <= physical_msb {
                    (physical_lsb, physical_msb)
                } else {
                    (physical_msb, physical_lsb)
                }
            }
        }
    }

    #[allow(clippy::similar_names)] // Overlap detection uses intentionally similar variable names (sig1_lsb/sig1_msb, sig2_lsb/sig2_msb)
    pub(crate) fn validate_internal(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Option<Signal<'a>>],
        signal_count: usize,
        options: crate::ParseOptions,
    ) -> ParseResult<()> {
        // Validate CAN ID range
        // CAN specification allows:
        // - Standard 11-bit IDs: 0x000 to 0x7FF (0-2047)
        // - Extended 29-bit IDs: 0x00000000 to 0x1FFFFFFF (0-536870911)
        // Note: Extended IDs can technically be 0-536870911, but DBC files typically
        // use the convention where IDs 0-2047 are treated as standard and 2048+ as extended.
        // We only validate the maximum range here; the distinction between standard/extended
        // is determined by the ID value in practice.
        const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF; // 536870911

        // Check signal count limit per message (DoS protection)
        const MAX_SIGNALS_PER_MESSAGE: usize = crate::Signals::max_capacity();
        if signal_count > MAX_SIGNALS_PER_MESSAGE {
            return Err(ParseError::Version(
                crate::error::lang::MESSAGE_TOO_MANY_SIGNALS,
            ));
        }

        if name.trim().is_empty() {
            return Err(ParseError::Version(crate::error::lang::MESSAGE_NAME_EMPTY));
        }

        if sender.trim().is_empty() {
            return Err(ParseError::Version(
                crate::error::lang::MESSAGE_SENDER_EMPTY,
            ));
        }

        // Validate DLC (Data Length Code): must be between 1 and 64 bytes
        // - Classic CAN Standard (CAN 2.0A): DLC <= 8 bytes (64 bits) maximum payload
        // - Classic CAN Extended (CAN 2.0B): DLC <= 8 bytes (64 bits) maximum payload
        // - CAN FD (Flexible Data Rate, ISO/Bosch): DLC <= 64 bytes (512 bits) maximum payload
        if dlc == 0 {
            #[cfg(feature = "alloc")]
            {
                use crate::error::{messages, version_error_from_string};
                let msg = messages::message_dlc_too_small(name, id, dlc);
                return Err(version_error_from_string(msg));
            }
            #[cfg(not(feature = "alloc"))]
            {
                return Err(ParseError::Version(
                    crate::error::lang::MESSAGE_DLC_TOO_SMALL,
                ));
            }
        }
        if dlc > 64 {
            #[cfg(feature = "alloc")]
            {
                use crate::error::{messages, version_error_from_string};
                let msg = messages::message_dlc_too_large(name, id, dlc);
                return Err(version_error_from_string(msg));
            }
            #[cfg(not(feature = "alloc"))]
            {
                return Err(ParseError::Version(
                    crate::error::lang::MESSAGE_DLC_TOO_LARGE,
                ));
            }
        }

        // Validate that ID is within valid CAN ID range
        // Extended CAN IDs can be 0x00000000 to 0x1FFFFFFF (0 to 536870911)
        // IDs exceeding 0x1FFFFFFF are invalid
        if id > MAX_EXTENDED_ID {
            #[cfg(feature = "alloc")]
            {
                use crate::error::{messages, version_error_from_string};
                let msg = messages::message_id_out_of_range(id);
                return Err(version_error_from_string(msg));
            }
            #[cfg(not(feature = "alloc"))]
            {
                return Err(ParseError::Version(
                    crate::error::lang::MESSAGE_ID_OUT_OF_RANGE,
                ));
            }
        }

        // Validate that all signals fit within the message boundary
        // Each signal must fit within: DLC * 8 bits
        // - Classic CAN (2.0A/2.0B): DLC * 8 <= 64 bits (8 bytes)
        // - CAN FD: DLC * 8 <= 512 bits (64 bytes)
        // This ensures no signal extends beyond the message payload capacity
        let max_bits = u16::from(dlc) * 8;
        for signal in signals.iter().take(signal_count).filter_map(|opt| opt.as_ref()) {
            // Calculate the actual bit range for this signal (accounting for byte order)
            let (lsb, msb) =
                Self::calculate_bit_range(signal.start_bit(), signal.length(), signal.byte_order());
            // Check if the signal extends beyond the message boundary
            // The signal's highest bit position must be less than max_bits
            let signal_max_bit = lsb.max(msb);
            if signal_max_bit >= max_bits {
                // Only fail if strict boundary checking is enabled
                if options.strict_boundary_check {
                    #[cfg(feature = "alloc")]
                    {
                        use crate::error::messages;
                        let msg = messages::signal_extends_beyond_message(
                            signal.name(),
                            signal.start_bit(),
                            signal.length(),
                            signal_max_bit,
                            max_bits,
                            dlc,
                        );
                        use crate::error::version_error_from_string;
                        return Err(version_error_from_string(msg));
                    }
                    #[cfg(not(feature = "alloc"))]
                    {
                        return Err(ParseError::Version(
                            crate::error::lang::SIGNAL_LENGTH_TOO_LARGE,
                        ));
                    }
                }
                // In lenient mode, we allow signals that extend beyond boundaries
            }
        }

        // Validate signal overlap detection
        // Check if any two signals overlap in the same message
        // Must account for byte order: little-endian signals extend forward,
        // big-endian signals extend backward from start_bit
        // We iterate over pairs without collecting to avoid alloc
        let signals_slice = &signals[..signal_count];
        for (i, sig1_opt) in signals_slice.iter().enumerate() {
            let sig1 = match sig1_opt {
                Some(s) => s,
                None => continue, // Should not happen, but be safe
            };
            let (sig1_lsb, sig1_msb) =
                Self::calculate_bit_range(sig1.start_bit(), sig1.length(), sig1.byte_order());

            for sig2_opt in signals_slice.iter().skip(i + 1) {
                let sig2 = match sig2_opt {
                    Some(s) => s,
                    None => continue, // Should not happen, but be safe
                };
                let (sig2_lsb, sig2_msb) =
                    Self::calculate_bit_range(sig2.start_bit(), sig2.length(), sig2.byte_order());

                // Check if ranges overlap
                // Two ranges [lsb1, msb1] and [lsb2, msb2] overlap if:
                // lsb1 <= msb2 && lsb2 <= msb1
                if sig1_lsb <= sig2_msb && sig2_lsb <= sig1_msb {
                    #[cfg(feature = "alloc")]
                    {
                        use crate::error::{messages, version_error_from_string};
                        let msg = messages::signal_overlap(sig1.name(), sig2.name(), name);
                        return Err(version_error_from_string(msg));
                    }
                    #[cfg(not(feature = "alloc"))]
                    {
                        return Err(ParseError::Version(crate::error::lang::SIGNAL_OVERLAP));
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)] // Only used by builders (std-only)
    pub(crate) fn new(
        id: u32,
        name: &'a str,
        dlc: u8,
        sender: &'a str,
        signals: &'a [Signal<'a>],
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
            id,
            name,
            dlc,
            sender,
            signals: Signals::from_signals_slice(signals),
        }
    }

    fn new_from_options(
        id: u32,
        name: &'a str,
        dlc: u8,
        sender: &'a str,
        signals: &[Option<Signal<'a>>],
        signal_count: usize,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
            id,
            name,
            dlc,
            sender,
            signals: Signals::from_options_slice(signals, signal_count),
        }
    }

    pub(crate) fn parse<'b: 'a>(
        parser: &mut Parser<'b>,
        signals: &[Option<Signal<'a>>],
        signal_count: usize,
        options: crate::ParseOptions,
    ) -> ParseResult<Self> {
        // Expect "BO_" keyword (should already be consumed by find_next_keyword, but handle both cases)
        if parser.expect(crate::BO_.as_bytes()).is_err() {
            // Already past "BO_" from find_next_keyword, continue
        }

        // Skip whitespace
        let _ = parser.skip_whitespace();

        // Parse message ID
        let id = parser
            .parse_u32()
            .map_err(|_| ParseError::Version(crate::error::lang::MESSAGE_INVALID_ID))?;

        // Skip whitespace
        parser
            .skip_whitespace()
            .map_err(|_| ParseError::Expected("Expected whitespace"))?;

        // Parse message name (identifier)
        let name = parser
            .parse_identifier()
            .map_err(|_| ParseError::Version(crate::error::lang::MESSAGE_NAME_EMPTY))?;

        // Skip whitespace (optional before colon)
        let _ = parser.skip_whitespace();

        // Expect colon
        parser.expect(b":").map_err(|_| ParseError::Expected("Expected colon"))?;

        // Skip whitespace after colon
        let _ = parser.skip_whitespace();

        // Parse DLC
        let dlc = parser
            .parse_u8()
            .map_err(|_| ParseError::Version(crate::error::lang::MESSAGE_INVALID_DLC))?;

        // Skip whitespace
        parser
            .skip_whitespace()
            .map_err(|_| ParseError::Expected("Expected whitespace"))?;

        // Parse sender (identifier, until end of line or whitespace)
        let sender = parser
            .parse_identifier()
            .map_err(|_| ParseError::Version(crate::error::lang::MESSAGE_SENDER_EMPTY))?;

        // Check for extra content after sender (invalid format)
        parser.skip_newlines_and_spaces();
        if !parser.is_empty() {
            return Err(ParseError::Expected(
                "Unexpected content after message sender",
            ));
        }

        // Validate before construction
        Self::validate_internal(
            id,
            name,
            dlc,
            sender,
            &signals[..signal_count],
            signal_count,
            options,
        )?;
        // Construct directly (validation already done)
        Ok(Self::new_from_options(
            id,
            name,
            dlc,
            sender,
            signals,
            signal_count,
        ))
    }

    /// Returns the CAN message ID.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.id(), 256);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the message name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.name(), "EngineData");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the Data Length Code (DLC) in bytes.
    ///
    /// DLC specifies the size of the message payload. For classic CAN, this is 1-8 bytes.
    /// For CAN FD, this can be up to 64 bytes.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.dlc(), 8);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn dlc(&self) -> u8 {
        self.dlc
    }

    #[inline]
    #[must_use]
    pub fn sender(&self) -> &'a str {
        self.sender
    }

    /// Get a reference to the signals collection
    #[inline]
    #[must_use]
    pub fn signals(&self) -> &Signals<'a> {
        &self.signals
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> alloc::string::String {
        use alloc::format;
        format!(
            "BO_ {} {} : {} {}",
            self.id(),
            self.name(),
            self.dlc(),
            self.sender()
        )
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string_with_signals(&self) -> alloc::string::String {
        use alloc::string::String;
        let mut result = String::with_capacity(200 + (self.signals.len() * 100));
        result.push_str(&self.to_dbc_string());
        result.push('\n');

        for signal in self.signals().iter() {
            result.push_str(&signal.to_dbc_string());
            result.push('\n');
        }

        result
    }
}

#[cfg(feature = "alloc")]
impl<'a> core::fmt::Display for Message<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string_with_signals())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{Parser, error::ParseError};
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use alloc::{format, vec};
    // Note: Builder tests have been moved to message_builder.rs
    // This module only tests Message parsing and direct API usage

    // Note: All test_message_new_* tests have been removed - they belong in message_builder.rs
    // This module only tests Message parsing and direct API usage

    #[test]
    fn test_message_parse_valid() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 0);
    }

    #[test]
    fn test_message_parse_invalid_id() {
        let data = b"BO_ invalid EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let result = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new());
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(_) => {
                // Expected
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_empty_name() {
        let data = b"BO_ 256  : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let result = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new());
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(_) => {
                // Expected
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_invalid_dlc() {
        let data = b"BO_ 256 EngineData : invalid ECM";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let result = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new());
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(_) => {
                // Expected
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_empty_sender() {
        let data = b"BO_ 256 EngineData : 8 ";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let result = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new());
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(_) => {
                // Expected
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_with_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Create test signals
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 2);
    }

    #[test]
    fn test_message_signals_iterator() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Create test signals
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();
        let mut signals_iter = message.signals().iter();
        assert_eq!(signals_iter.next().unwrap().name(), "RPM");
        assert_eq!(signals_iter.next().unwrap().name(), "Temp");
        assert!(signals_iter.next().is_none());
    }

    #[test]
    fn test_message_signal_count() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.signals().len(), 0);

        // Create a new parser for the second parse since the first one consumed the input
        let data2 = b"BO_ 256 EngineData : 8 ECM";
        let mut parser2 = Parser::new(data2).unwrap();
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        let message =
            Message::parse(&mut parser2, &signals, 1, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.signals().len(), 1);
    }

    #[test]
    fn test_message_signal_at() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.signals().at(0).unwrap().name(), "RPM");
        assert_eq!(message.signals().at(1).unwrap().name(), "Temp");
        assert!(message.signals().at(2).is_none());
    }

    #[test]
    fn test_message_find_signal() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.signals().find("RPM").unwrap().name(), "RPM");
        assert_eq!(message.signals().find("Temp").unwrap().name(), "Temp");
        assert!(message.signals().find("Nonexistent").is_none());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        const MAX_CAP: usize = crate::Signals::max_capacity();
        let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_with_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();
        let dbc_string = message.to_dbc_string_with_signals();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    fn test_message_can_2_0a_dlc_limits() {
        use alloc::format;
        // CAN 2.0A: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            const MAX_CAP: usize = crate::Signals::max_capacity();
            let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
            let message =
                Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    fn test_message_can_2_0b_dlc_limits() {
        use alloc::format;
        // CAN 2.0B: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            const MAX_CAP: usize = crate::Signals::max_capacity();
            let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
            let message =
                Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    fn test_message_can_fd_dlc_limits() {
        use alloc::format;
        // CAN FD: DLC can be 1-64 bytes (8-512 bits)
        // Test valid DLC values up to 64
        for dlc in [1, 8, 12, 16, 20, 24, 32, 48, 64] {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            const MAX_CAP: usize = crate::Signals::max_capacity();
            let signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
            let message =
                Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_multiple_signals_boundary_validation() {
        // Test that signals at message boundaries are validated correctly
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Create signals that exactly fill the message (8 bytes = 64 bits)
        // Signal 1: bits 0-15 (16 bits)
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|16@0+ (1,0) [0|65535] \"\"").unwrap())
                .unwrap();
        // Signal 2: bits 16-31 (16 bits)
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Signal2 : 16|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 3: bits 32-47 (16 bits)
        let signal3 = Signal::parse(
            &mut Parser::new(b"SG_ Signal3 : 32|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 4: bits 48-63 (16 bits) - exactly at boundary
        let signal4 = Signal::parse(
            &mut Parser::new(b"SG_ Signal4 : 48|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);
        signals[2] = Some(signal3);
        signals[3] = Some(signal4);

        let message = Message::parse(&mut parser, &signals, 4, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.signals().len(), 4);
    }

    #[test]
    fn test_message_big_endian_bit_range_calculation() {
        // Test big-endian bit range calculation
        // BE bit 0 -> physical bit 7
        // BE bit 7 -> physical bit 0
        // BE bit 8 -> physical bit 15
        // BE bit 15 -> physical bit 8
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal starting at BE bit 0, length 8 -> should map to physical bits 0-7
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@1+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal);

        let message = Message::parse(&mut parser, &signals, 1, crate::ParseOptions::new()).unwrap();
        // The signal should be valid and fit within the message
        assert_eq!(message.signals().len(), 1);
    }

    #[test]
    fn test_message_little_endian_bit_range_calculation() {
        // Test little-endian bit range calculation
        // LE bit N -> physical bit N (straightforward mapping)
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal starting at LE bit 0, length 8 -> should map to physical bits 0-7
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal);

        let message = Message::parse(&mut parser, &signals, 1, crate::ParseOptions::new()).unwrap();
        // The signal should be valid and fit within the message
        assert_eq!(message.signals().len(), 1);
    }

    // Note: Big-endian signal overlap and boundary tests have been moved to message_builder.rs

    #[test]
    fn test_parse_with_lenient_boundary_check() {
        // Test that lenient mode allows signals that extend beyond message boundaries
        let data = b"BO_ 256 Test : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal that extends beyond 8-byte boundary (start_bit 63, length 8 = bits 63-70, exceeds 64 bits)
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ CHECKSUM : 63|8@1+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal);

        // Strict mode should fail
        let result = Message::parse(&mut parser, &signals, 1, crate::ParseOptions::new());
        assert!(result.is_err());

        // Lenient mode should succeed
        let mut parser2 = Parser::new(data).unwrap();
        let message =
            Message::parse(&mut parser2, &signals, 1, crate::ParseOptions::lenient()).unwrap();
        assert_eq!(message.signals().len(), 1);
        assert_eq!(message.signals().at(0).unwrap().name(), "CHECKSUM");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_empty_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");

        let dbc_string_with_signals = message.to_dbc_string_with_signals();
        assert_eq!(dbc_string_with_signals, "BO_ 256 EngineData : 8 ECM\n");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_special_characters() {
        let data = b"BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(
            dbc_string,
            "BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node"
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_extended_id() {
        // Use a valid extended ID (max is 0x1FFF_FFFF = 536870911)
        let data = b"BO_ 536870911 ExtendedID : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 536870911 ExtendedID : 8 ECM");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_dlc_edge_cases() {
        // Test DLC = 1
        let data = b"BO_ 256 MinDLC : 1 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
        assert_eq!(message.to_dbc_string(), "BO_ 256 MinDLC : 1 ECM");

        // Test DLC = 64 (CAN FD max)
        let data2 = b"BO_ 257 MaxDLC : 64 ECM";
        let mut parser2 = Parser::new(data2).unwrap();
        let message2 =
            Message::parse(&mut parser2, &signals, 0, crate::ParseOptions::new()).unwrap();
        assert_eq!(message2.to_dbc_string(), "BO_ 257 MaxDLC : 64 ECM");
    }

    #[test]
    fn test_message_signals_is_empty() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();
        assert!(message.signals().is_empty());
        assert_eq!(message.signals().len(), 0);
    }

    #[test]
    fn test_message_signals_at_out_of_bounds() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal);

        let message = Message::parse(&mut parser, &signals, 1, crate::ParseOptions::new()).unwrap();

        // Valid index
        assert!(message.signals().at(0).is_some());
        assert_eq!(message.signals().at(0).unwrap().name(), "RPM");

        // Out of bounds
        assert!(message.signals().at(1).is_none());
        assert!(message.signals().at(100).is_none());
        assert!(message.signals().at(usize::MAX).is_none());
    }

    #[test]
    fn test_message_signals_find_case_sensitive() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();

        // Exact match
        assert!(message.signals().find("RPM").is_some());
        assert_eq!(message.signals().find("RPM").unwrap().name(), "RPM");

        // Case sensitive - should not find
        assert!(message.signals().find("rpm").is_none());
        assert!(message.signals().find("Rpm").is_none());

        // Find second signal
        assert!(message.signals().find("Temp").is_some());
        assert_eq!(message.signals().find("Temp").unwrap().name(), "Temp");

        // Not found
        assert!(message.signals().find("Nonexistent").is_none());
        assert!(message.signals().find("").is_none());
    }

    #[test]
    fn test_message_signals_find_empty_collection() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        assert!(message.signals().find("RPM").is_none());
        assert!(message.signals().find("").is_none());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_display_trait() {
        use alloc::format;
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signal from DBC string instead of using builder
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal);

        let message = Message::parse(&mut parser, &signals, 1, crate::ParseOptions::new()).unwrap();

        let display_str = format!("{}", message);
        assert!(display_str.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(display_str.contains("SG_ RPM"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_message_to_dbc_string_with_signals_multiple() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signals from DBC strings instead of using builders
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\" *").unwrap(),
        )
        .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);

        let message = Message::parse(&mut parser, &signals, 2, crate::ParseOptions::new()).unwrap();

        let dbc_string = message.to_dbc_string_with_signals();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
        // Should have newlines between signals
        #[cfg(any(feature = "alloc", feature = "kernel"))]
        use crate::compat::Vec;
        let lines: Vec<&str> = dbc_string.lines().collect();
        assert!(lines.len() >= 3); // Message line + at least 2 signal lines
    }

    #[test]
    fn test_message_getters_edge_cases() {
        // Test with minimum values
        let data = b"BO_ 0 A : 1 B";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        assert_eq!(message.id(), 0);
        assert_eq!(message.name(), "A");
        assert_eq!(message.dlc(), 1);
        assert_eq!(message.sender(), "B");
    }

    #[test]
    fn test_message_signals_iterator_empty() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: [Option<Signal>; crate::Signals::max_capacity()] =
            [const { None }; crate::Signals::max_capacity()];
        let message = Message::parse(&mut parser, &signals, 0, crate::ParseOptions::new()).unwrap();

        let mut iter = message.signals().iter();
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_message_signals_iterator_multiple() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal2 =
            Signal::parse(&mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal3 =
            Signal::parse(&mut Parser::new(b"SG_ Signal3 : 16|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        const MAX_CAP: usize = crate::Signals::max_capacity();
        let mut signals: [Option<Signal>; MAX_CAP] = [const { None }; MAX_CAP];
        signals[0] = Some(signal1);
        signals[1] = Some(signal2);
        signals[2] = Some(signal3);

        let message = Message::parse(&mut parser, &signals, 3, crate::ParseOptions::new()).unwrap();

        let mut iter = message.signals().iter();
        assert_eq!(iter.next().unwrap().name(), "Signal1");
        assert_eq!(iter.next().unwrap().name(), "Signal2");
        assert_eq!(iter.next().unwrap().name(), "Signal3");
        assert!(iter.next().is_none());

        // Test that iterator can be used multiple times
        #[cfg(any(feature = "alloc", feature = "kernel"))]
        {
            use crate::compat::Vec;
            use alloc::vec;
            let names: Vec<&str> = message.signals().iter().map(|s| s.name()).collect();
            // Convert kernel Vec to alloc Vec for comparison
            #[cfg(feature = "kernel")]
            let names: alloc::vec::Vec<&str> = names.into();
            assert_eq!(names, vec!["Signal1", "Signal2", "Signal3"]);
        }
    }
}
