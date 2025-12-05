use crate::{
    ByteOrder, Parser, Signal,
    error::{ParseError, ParseResult},
};

#[cfg(feature = "std")]
mod message_builder;

#[cfg(feature = "std")]
pub use message_builder::MessageBuilder;

#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    id: u32,
    name: &'a str,
    dlc: u8,
    sender: &'a str,
    // Store signals in a fixed-size array (no alloc needed)
    signals: [Option<Signal<'a>>; crate::MAX_SIGNALS_PER_MESSAGE],
    signal_count: usize,
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
    fn validate_internal(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Option<Signal<'a>>],
        signal_count: usize,
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
        const MAX_SIGNALS_PER_MESSAGE: usize = 64;
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
            return Err(ParseError::Version(
                crate::error::lang::MESSAGE_DLC_TOO_SMALL,
            ));
        }
        if dlc > 64 {
            return Err(ParseError::Version(
                crate::error::lang::MESSAGE_DLC_TOO_LARGE,
            ));
        }

        // Validate that ID is within valid CAN ID range
        // Extended CAN IDs can be 0x00000000 to 0x1FFFFFFF (0 to 536870911)
        // IDs exceeding 0x1FFFFFFF are invalid
        if id > MAX_EXTENDED_ID {
            #[cfg(feature = "std")]
            {
                use crate::error::messages;
                let msg = messages::message_id_out_of_range(id);
                return Err(ParseError::Version(Box::leak(msg.into_boxed_str())));
            }
            #[cfg(not(feature = "std"))]
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
                #[cfg(feature = "std")]
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
                    return Err(ParseError::Version(Box::leak(msg.into_boxed_str())));
                }
                #[cfg(not(feature = "std"))]
                {
                    return Err(ParseError::Version(
                        crate::error::lang::SIGNAL_LENGTH_TOO_LARGE,
                    ));
                }
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
                    #[cfg(feature = "std")]
                    {
                        use crate::error::messages;
                        let msg = messages::signal_overlap(sig1.name(), sig2.name(), name);
                        return Err(ParseError::Version(Box::leak(msg.into_boxed_str())));
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        return Err(ParseError::Version(crate::error::lang::SIGNAL_OVERLAP));
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)] // Used by MessageBuilder (std-gated)
    fn new(id: u32, name: &'a str, dlc: u8, sender: &'a str, signals: &'a [Signal<'a>]) -> Self {
        // Validation should have been done prior (by builder or parse)
        // Convert slice to array by cloning signals
        let mut signals_array: [Option<Signal<'a>>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let count = signals.len().min(crate::MAX_SIGNALS_PER_MESSAGE);
        for (i, signal) in signals.iter().take(crate::MAX_SIGNALS_PER_MESSAGE).enumerate() {
            signals_array[i] = Some(signal.clone());
        }
        Self {
            id,
            name,
            dlc,
            sender,
            signals: signals_array,
            signal_count: count,
        }
    }

    fn new_from_options(
        id: u32,
        name: &'a str,
        dlc: u8,
        sender: &'a str,
        signals: [Option<Signal<'a>>; crate::MAX_SIGNALS_PER_MESSAGE],
        signal_count: usize,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
            id,
            name,
            dlc,
            sender,
            signals,
            signal_count,
        }
    }

    pub(crate) fn parse<'b: 'a>(
        parser: &mut Parser<'b>,
        signals: [Option<Signal<'a>>; crate::MAX_SIGNALS_PER_MESSAGE],
        signal_count: usize,
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
        Self::validate_internal(id, name, dlc, sender, &signals[..], signal_count)?;
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

    #[inline]
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    #[must_use]
    pub fn name(&self) -> &'a str {
        self.name
    }

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

    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn signals(&self) -> impl Iterator<Item = &Signal<'a>> + '_ {
        self.signals.iter().take(self.signal_count).filter_map(|opt| opt.as_ref())
    }

    /// Get the number of signals in this message
    #[inline]
    #[must_use]
    pub fn signal_count(&self) -> usize {
        self.signal_count
    }

    /// Get a signal by index, or None if index is out of bounds
    #[inline]
    #[must_use]
    pub fn signal_at(&self, index: usize) -> Option<&Signal<'a>> {
        self.signals().nth(index)
    }

    #[must_use]
    pub fn find_signal(&self, name: &str) -> Option<&Signal<'a>> {
        self.signals().find(|s| s.name() == name)
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        format!(
            "BO_ {} {} : {} {}",
            self.id(),
            self.name(),
            self.dlc(),
            self.sender()
        )
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string_with_signals(&self) -> String {
        let mut result = String::with_capacity(200 + (self.signal_count * 100));
        result.push_str(&self.to_dbc_string());
        result.push('\n');

        for signal in self.signals() {
            result.push_str(&signal.to_dbc_string());
            result.push('\n');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{
        ByteOrder, Error, Parser, Receivers,
        error::{ParseError, lang},
    };
    #[cfg(feature = "std")]
    use crate::{MessageBuilder, ReceiversBuilder, SignalBuilder};

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_valid() {
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
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signal_count(), 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_empty_name() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_NAME_EMPTY)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_empty_sender() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_SENDER_EMPTY)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_zero_dlc() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(0)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_SMALL)),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_dlc_too_large() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        // DLC > 64 should fail (CAN FD maximum is 64 bytes)
        let result = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(65)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_LARGE)),
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_signal_overflow() {
        // Signal extends beyond DLC boundary
        // Test with a signal that fits Signal validation (length <= 512) but exceeds message DLC
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(32) // 32 bits = 4 bytes, fits in 8-byte message
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
        .unwrap();

        // Create message with DLC=2 (16 bits), but signal is 32 bits
        let result = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(2)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic)
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_new_multiple_signals() {
        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Temperature")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build()
            .unwrap();
        assert_eq!(message.signal_count(), 2);
    }

    #[test]
    fn test_message_parse_invalid_dlc() {
        // Test that parse also validates DLC (CAN FD maximum is 64 bytes)
        let line = "BO_ 256 EngineData : 65 ECM";
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_LARGE)),
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_zero_dlc() {
        // Test that parse also validates DLC
        let line = "BO_ 256 EngineData : 0 ECM";
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_SMALL)),
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_parse_signal_overflow() {
        // Test that parse validates signals fit within message DLC
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(32) // 32 bits = 4 bytes
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
        .unwrap();

        // Message with DLC=2 (16 bits), but signal is 32 bits
        let line = "BO_ 256 EngineData : 2 ECM";
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let mut signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        signals[0] = Some(signal);
        let result = Message::parse(&mut parser, signals, 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => {
                // Check for format template text (language-agnostic)
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_invalid_format() {
        // Test parse with too few parts (only 3 parts)
        let line = "BO_ 256 EngineData";
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Expected(_) | ParseError::Version(_) => {
                // Format errors can be either Expected or Version
            }
            _ => panic!("Expected ParseError"),
        }

        // Test parse with too many parts (7 parts)
        let line = "BO_ 256 EngineData : 8 ECM extra";
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Expected(_) | ParseError::Version(_) => {
                // Format errors can be either Expected or Version
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_message_parse_invalid_id() {
        // Test parse with invalid message ID
        let line = "BO_ abc EngineData : 8 ECM";
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_ID)),
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    fn test_message_parse_invalid_dlc_string() {
        // Test parse with invalid DLC (non-numeric)
        let line = "BO_ 256 EngineData : abc ECM";
        let mut parser = Parser::new(line.as_bytes()).unwrap();
        let signals: [Option<Signal>; crate::MAX_SIGNALS_PER_MESSAGE] =
            [const { None }; crate::MAX_SIGNALS_PER_MESSAGE];
        let result = Message::parse(&mut parser, signals, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_DLC)),
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_to_dbc_string() {
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
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();
        assert_eq!(message.to_dbc_string(), "BO_ 256 EngineData : 8 ECM");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_to_dbc_string_with_signals() {
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
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

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
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build()
            .unwrap();
        let dbc_str = message.to_dbc_string_with_signals();

        assert!(dbc_str.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_str.contains("SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *")); // BigEndian = @0
        assert!(dbc_str.contains("SG_ Temperature : 16|8@1- (1,-40) [-40|215] \"°C\"")); // LittleEndian = @1
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_message_id_out_of_range() {
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        // Test ID that exceeds extended 29-bit range
        let result = MessageBuilder::new()
            .id(0x2000_0000)
            .name("Test")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_MESSAGE_ID_OUT_OF_RANGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }

        // Test valid standard ID (11-bit) - minimum
        let result = MessageBuilder::new()
            .id(0)
            .name("Test")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(result.is_ok());

        // Test valid standard ID (11-bit) - maximum
        let result = MessageBuilder::new()
            .id(0x7FF)
            .name("Test")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - minimum
        let result = MessageBuilder::new()
            .id(0x800)
            .name("Test")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - maximum
        let result = MessageBuilder::new()
            .id(0x1FFF_FFFF)
            .name("Test")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_signal_overlap() {
        // Two big-endian signals that overlap
        // Signal1: BE start=0, length=16 -> physical range [7, 17] (MSB at 7, LSB at 17)
        // Signal2: BE start=1, length=16 -> physical range [6, 18] (MSB at 6, LSB at 18)
        // They overlap in physical bits [7, 17]
        let signal1 = SignalBuilder::new()
            .name("Signal1")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Signal2")
            .start_bit(1)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text and signal names (language-agnostic)
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_SIGNAL_OVERLAP.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
                assert!(msg.contains("Signal1"));
                assert!(msg.contains("Signal2"));
                assert!(msg.contains("TestMessage"));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_signal_no_overlap() {
        // Two signals that don't overlap: signal1 at 0-15, signal2 at 16-31
        let signal1 = SignalBuilder::new()
            .name("Signal1")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Signal2")
            .start_bit(16)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_signal_overlap_adjacent() {
        // Two signals that are adjacent but don't overlap: signal1 at 0-15, signal2 at 16-23
        let signal1 = SignalBuilder::new()
            .name("Signal1")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Signal2")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_signal_overlap_identical_position() {
        // Two signals at the exact same position (definitely overlap)
        let signal1 = SignalBuilder::new()
            .name("Signal1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Signal2")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic)
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_SIGNAL_OVERLAP.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_too_many_signals() {
        // Create a message with 65 signals (exceeds limit of 64)
        // Use 1-bit signals, but we need to check count BEFORE boundary validation
        // So we'll create signals that would fit if not for the count limit
        // Note: Signal count check happens before boundary check in validate()
        let mut signals = Vec::new();
        for i in 0..65 {
            // Use modulo to wrap signals within 64 bits (8 bytes)
            let start_bit = i % 64;
            let name = format!("Signal{i}");
            let name_leaked = Box::leak(name.into_boxed_str());
            let signal = SignalBuilder::new()
                .name(name_leaked)
                .start_bit(start_bit)
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
            signals.push(signal);
        }
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signals(signals)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                assert!(msg.contains(lang::MESSAGE_TOO_MANY_SIGNALS));
            }
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_at_signal_limit() {
        // Create a message with exactly 64 signals (at the limit)
        // Use 1-bit signals to fit many in a single byte
        let mut signals = Vec::new();
        for i in 0..64 {
            let name = format!("Signal{i}");
            let name_leaked = Box::leak(name.into_boxed_str());
            let signal = SignalBuilder::new()
                .name(name_leaked)
                .start_bit(i)
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
            signals.push(signal);
        }
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signals(signals)
            .build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().signal_count(), 64);
    }

    #[test]
    fn test_message_can_2_0a_dlc_limits() {
        // Classic CAN Standard (CAN 2.0A): DLC 1-8 bytes
        // Note: The library supports all CAN variants (2.0A, 2.0B, CAN FD),
        // so DLC 1-64 is valid. This test verifies CAN 2.0A typical usage.
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(0)
            .length(64) // 64 bits = 8 bytes, fits exactly in DLC=8
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // DLC=8 should work (64 bits) - typical CAN 2.0A maximum
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_ok(), "CAN 2.0A should support DLC=8 (64 bits)");
    }

    #[test]
    fn test_message_can_2_0b_dlc_limits() {
        // Classic CAN Extended (CAN 2.0B): DLC 1-8 bytes (same as 2.0A)
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(0)
            .length(32) // 32 bits = 4 bytes
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // DLC=8 should work (64 bits)
        let result = MessageBuilder::new()
            .id(0x800)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_ok(), "CAN 2.0B should support DLC=8 (64 bits)");
    }

    #[test]
    fn test_message_can_fd_dlc_limits() {
        // CAN FD (Flexible Data Rate): DLC 1-64 bytes (512 bits)
        // Test maximum DLC for CAN FD
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(0)
            .length(64) // 64 bits = 8 bytes
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // DLC=64 should work (512 bits = 64 bytes)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(64)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(result.is_ok(), "CAN FD should support DLC=64 (512 bits)");

        // DLC=65 should fail (exceeds CAN FD limit)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(65)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err(), "CAN FD should reject DLC > 64");
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_LARGE)),
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_signal_boundary_validation_can_2_0a() {
        // Classic CAN (2.0A): DLC * 8 <= 64 bits
        // Signal must fit within message boundary
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(56) // Start at bit 56
            .length(8)  // 8 bits, ends at bit 63 (fits in 8 bytes)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // Should work: signal fits in DLC=8 (64 bits)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(
            result.is_ok(),
            "Signal should fit in CAN 2.0A message (DLC=8)"
        );

        // Should fail: signal extends beyond DLC=7 (56 bits)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(7)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(result.is_err(), "Signal should not fit in DLC=7 (56 bits)");
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_signal_boundary_validation_can_fd() {
        // CAN FD: DLC * 8 <= 512 bits
        // Signal must fit within message boundary
        // Note: start_bit is u16 (0-65535), supporting CAN FD up to 512 bits (64 bytes)
        // For DLC=64 (512 bits), we can test with start_bit=248, length=8 (ends at bit 255)
        // But for a more realistic test, we'll use start_bit=248, length=8 which fits in DLC=32 (256 bits)
        let signal = SignalBuilder::new()
            .name("TestSignal")
            .start_bit(248) // Start at bit 248 (max u8 is 255)
            .length(8)   // 8 bits, ends at bit 255 (fits in 32 bytes = 256 bits)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // Should work: signal fits in DLC=32 (256 bits)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(32)
            .sender("ECM")
            .add_signal(signal.clone())
            .build();
        assert!(
            result.is_ok(),
            "Signal should fit in CAN FD message (DLC=32)"
        );

        // Should fail: signal extends beyond DLC=31 (248 bits)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(31)
            .sender("ECM")
            .add_signal(signal)
            .build();
        assert!(
            result.is_err(),
            "Signal should not fit in DLC=31 (248 bits)"
        );
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_multiple_signals_boundary_validation() {
        // Test that multiple signals must all fit within DLC * 8 bits
        // Classic CAN: DLC=8 (64 bits)
        let signal1 = SignalBuilder::new()
            .name("Signal1")
            .start_bit(0)
            .length(32) // 32 bits
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Signal2")
            .start_bit(32)
            .length(32) // 32 bits, total = 64 bits
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
        .unwrap();

        // Should work: both signals fit in DLC=8 (64 bits total)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1.clone())
            .add_signal(signal2.clone())
            .build();
        assert!(result.is_ok(), "Both signals should fit in DLC=8");

        // Should fail: signals don't fit in DLC=7 (56 bits)
        let result = MessageBuilder::new()
            .id(256)
            .name("TestMessage")
            .dlc(7)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build();
        assert!(result.is_err(), "Signals should not fit in DLC=7");
        match result.unwrap_err() {
            Error::Message(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text =
                    lang::FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Message error"),
        }
    }
}
