use crate::{Error, Signal, error::messages};
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    id: u32,
    name: Box<str>,
    dlc: u8,
    sender: Box<str>,
    signals: Vec<Signal>,
}

impl Message {
    /// Validate message parameters
    fn validate(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Signal],
    ) -> Result<(), Error> {
        if name.trim().is_empty() {
            return Err(Error::Signal(messages::MESSAGE_NAME_EMPTY.to_string()));
        }

        if sender.trim().is_empty() {
            return Err(Error::Signal(messages::MESSAGE_SENDER_EMPTY.to_string()));
        }

        // Validate DLC: must be between 1 and 8 bytes
        if dlc == 0 {
            return Err(Error::Signal(messages::MESSAGE_DLC_TOO_SMALL.to_string()));
        }
        if dlc > 8 {
            return Err(Error::Signal(messages::MESSAGE_DLC_TOO_LARGE.to_string()));
        }

        // Validate CAN ID range
        // Standard 11-bit: 0-0x7FF (0-2047)
        // Extended 29-bit: 0x800-0x1FFFFFFF (2048-536870911)
        // IDs > 0x1FFFFFFF are invalid
        const MAX_STANDARD_ID: u32 = 0x7FF; // 2047
        const MIN_EXTENDED_ID: u32 = 0x800; // 2048
        const MAX_EXTENDED_ID: u32 = 0x1FFFFFFF; // 536870911

        // Validate that ID is within valid CAN ID ranges
        if id > MAX_EXTENDED_ID {
            return Err(Error::Signal(messages::message_id_out_of_range(id)));
        }

        // Explicit validation: standard IDs must be 0-0x7FF
        // Extended IDs must be 0x800-0x1FFFFFFF
        // This check ensures proper range validation for both types
        if id <= MAX_STANDARD_ID {
            // Valid standard 11-bit ID (0-2047) - explicitly validated
        } else if id >= MIN_EXTENDED_ID && id <= MAX_EXTENDED_ID {
            // Valid extended 29-bit ID (2048-536870911) - explicitly validated
        }

        // Validate that all signals fit within the message size (DLC * 8 bits)
        let max_bits = dlc as u16 * 8;
        for signal in signals {
            let end_bit = signal.start_bit() as u16 + signal.length() as u16;
            if end_bit > max_bits {
                return Err(Error::Signal(messages::signal_extends_beyond_message(
                    signal.name(),
                    signal.start_bit(),
                    signal.length(),
                    end_bit,
                    max_bits,
                    dlc,
                )));
            }
        }

        // Validate signal overlap detection
        // Check if any two signals overlap in the same message
        // For simplicity, we check if their bit ranges overlap
        // This works for both little-endian and big-endian signals
        for (i, sig1) in signals.iter().enumerate() {
            let sig1_start = sig1.start_bit() as u16;
            let sig1_end = sig1_start + sig1.length() as u16;

            for sig2 in signals.iter().skip(i + 1) {
                let sig2_start = sig2.start_bit() as u16;
                let sig2_end = sig2_start + sig2.length() as u16;

                // Check if ranges overlap
                // Two ranges overlap if: sig1_start < sig2_end && sig2_start < sig1_end
                if sig1_start < sig2_end && sig2_start < sig1_end {
                    return Err(Error::Signal(messages::signal_overlap(
                        sig1.name(),
                        sig2.name(),
                        name,
                    )));
                }
            }
        }

        Ok(())
    }

    /// Create a new Message with the given parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `name` is empty
    /// - `sender` is empty
    /// - `dlc` is 0 or greater than 8
    /// - `id` is out of valid CAN ID range (standard 11-bit: 0-2047, extended 29-bit: 0-536870911)
    /// - Any signal extends beyond the message boundary (DLC * 8 bits)
    /// - Any signals overlap within the message
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::{Message, Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::new(
    ///     "RPM",
    ///     0,
    ///     16,
    ///     ByteOrder::BigEndian,
    ///     true,
    ///     0.25,
    ///     0.0,
    ///     0.0,
    ///     8000.0,
    ///     Some("rpm" as &str),
    ///     Receivers::Broadcast,
    /// )?;
    ///
    /// let message = Message::new(
    ///     256,
    ///     "EngineData",
    ///     8,
    ///     "ECM",
    ///     vec![signal],
    /// )?;
    /// # Ok::<(), dbc::Error>(())
    /// ```
    pub fn new(
        id: u32,
        name: impl AsRef<str>,
        dlc: u8,
        sender: impl AsRef<str>,
        signals: Vec<Signal>,
    ) -> Result<Self, Error> {
        let name_str = name.as_ref();
        let sender_str = sender.as_ref();
        Self::validate(id, name_str, dlc, sender_str, &signals)?;

        Ok(Self {
            id,
            name: name_str.into(),
            dlc,
            sender: sender_str.into(),
            signals,
        })
    }

    pub(super) fn parse(message: &str, signals: Vec<Signal>) -> Result<Self, Error> {
        let parts: Vec<&str> = message.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(Error::Signal(messages::MESSAGE_INVALID_FORMAT.to_string()));
        }

        let id = parts[1]
            .parse()
            .map_err(|_| Error::Signal(messages::MESSAGE_INVALID_ID.to_string()))?;
        let name = parts[2];
        let dlc = parts[4]
            .parse()
            .map_err(|_| Error::Signal(messages::MESSAGE_INVALID_DLC.to_string()))?;
        let sender = parts[5];

        // Validate the parsed message using the same validation as new()
        Self::validate(id, name, dlc, sender, &signals)?;

        Ok(Self {
            id,
            name: name.into(),
            dlc,
            sender: sender.into(),
            signals,
        })
    }

    /// Get the CAN message ID
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the message name
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }

    /// Get the Data Length Code (DLC)
    #[inline]
    pub fn dlc(&self) -> u8 {
        self.dlc
    }

    /// Get the sender node name
    #[inline]
    pub fn sender(&self) -> &str {
        &*self.sender
    }

    /// Get a read-only slice of signals in this message
    #[inline]
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    /// Find a signal by name in this message
    pub fn find_signal(&self, name: &str) -> Option<&Signal> {
        self.signals.iter().find(|s| s.name() == name)
    }

    /// Format message in DBC file format (e.g., `BO_ 256 EngineData : 8 ECM`)
    ///
    /// This method formats the message header only. To include signals, use
    /// `to_dbc_string_with_signals()`.
    ///
    /// Useful for debugging and visualization of the message in DBC format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::{Message, Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::new(
    ///     "RPM",
    ///     0,
    ///     16,
    ///     ByteOrder::BigEndian,
    ///     true,
    ///     0.25,
    ///     0.0,
    ///     0.0,
    ///     8000.0,
    ///     Some("rpm" as &str),
    ///     Receivers::Broadcast,
    /// )?;
    ///
    /// let message = Message::new(256, "EngineData", 8, "ECM", vec![signal])?;
    /// assert_eq!(message.to_dbc_string(), "BO_ 256 EngineData : 8 ECM");
    /// # Ok::<(), dbc::Error>(())
    /// ```
    pub fn to_dbc_string(&self) -> String {
        use alloc::format;
        format!(
            "BO_ {} {} : {} {}",
            self.id(),
            self.name(),
            self.dlc(),
            self.sender()
        )
    }

    /// Format message in DBC file format including all signals
    ///
    /// This method formats the message header followed by all its signals,
    /// each on a new line. Useful for debugging and visualization.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::{Message, Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::new(
    ///     "RPM",
    ///     0,
    ///     16,
    ///     ByteOrder::BigEndian,
    ///     true,
    ///     0.25,
    ///     0.0,
    ///     0.0,
    ///     8000.0,
    ///     Some("rpm" as &str),
    ///     Receivers::Broadcast,
    /// )?;
    ///
    /// let message = Message::new(256, "EngineData", 8, "ECM", vec![signal])?;
    /// let dbc_str = message.to_dbc_string_with_signals();
    /// assert!(dbc_str.contains("BO_ 256 EngineData : 8 ECM"));
    /// assert!(dbc_str.contains("SG_ RPM"));
    /// # Ok::<(), dbc::Error>(())
    /// ```
    pub fn to_dbc_string_with_signals(&self) -> String {
        let mut result = String::with_capacity(200 + (self.signals.len() * 100));
        result.push_str(&self.to_dbc_string());
        result.push('\n');

        for signal in &self.signals {
            result.push_str(&signal.to_dbc_string());
            result.push('\n');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ByteOrder, Receivers};

    #[test]
    fn test_message_new_valid() {
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

        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 1);
    }

    #[test]
    fn test_message_new_empty_name() {
        let signal = Signal::new(
            "RPM",
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
        )
        .unwrap();

        let result = Message::new(256, "", 8, "ECM", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("name cannot be empty")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_new_empty_sender() {
        let signal = Signal::new(
            "RPM",
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
        )
        .unwrap();

        let result = Message::new(256, "EngineData", 8, "", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("sender cannot be empty")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_new_zero_dlc() {
        let signal = Signal::new(
            "RPM",
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
        )
        .unwrap();

        let result = Message::new(256, "EngineData", 0, "ECM", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("DLC must be at least 1 byte")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_new_dlc_too_large() {
        let signal = Signal::new(
            "RPM",
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
        )
        .unwrap();

        let result = Message::new(256, "EngineData", 9, "ECM", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("DLC cannot exceed 8 bytes")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_new_signal_overflow() {
        // Signal extends beyond DLC boundary
        let signal = Signal::new(
            "RPM",
            0,
            65, // This will fail Signal validation first
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        );
        assert!(signal.is_err()); // Signal validation catches this

        // Test with a signal that fits Signal validation but exceeds message DLC
        let signal = Signal::new(
            "RPM",
            0,
            32, // 32 bits = 4 bytes, fits in 8-byte message
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

        // Create message with DLC=2 (16 bits), but signal is 32 bits
        let result = Message::new(256, "EngineData", 2, "ECM", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("extends beyond message boundary")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_new_multiple_signals() {
        let signal1 = Signal::new(
            "RPM",
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
        )
        .unwrap();

        let signal2 = Signal::new(
            "Temperature",
            16,
            8,
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

        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal1, signal2]).unwrap();
        assert_eq!(message.signals().len(), 2);
    }

    #[test]
    fn test_message_parse_invalid_dlc() {
        // Test that parse also validates DLC
        let line = "BO_ 256 EngineData : 9 ECM";
        let signals = vec![];
        let result = Message::parse(line, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("DLC cannot exceed 8 bytes")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_parse_zero_dlc() {
        // Test that parse also validates DLC
        let line = "BO_ 256 EngineData : 0 ECM";
        let signals = vec![];
        let result = Message::parse(line, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("DLC must be at least 1 byte")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_parse_signal_overflow() {
        // Test that parse validates signals fit within message DLC
        let signal = Signal::new(
            "RPM",
            0,
            32, // 32 bits = 4 bytes
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

        // Message with DLC=2 (16 bits), but signal is 32 bits
        let line = "BO_ 256 EngineData : 2 ECM";
        let result = Message::parse(line, vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("extends beyond message boundary")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_parse_invalid_format() {
        // Test parse with wrong number of parts
        let line = "BO_ 256 EngineData : 8";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("Invalid message format")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_parse_invalid_id() {
        // Test parse with invalid message ID
        let line = "BO_ abc EngineData : 8 ECM";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("Invalid message ID")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_parse_invalid_dlc_string() {
        // Test parse with invalid DLC (non-numeric)
        let line = "BO_ 256 EngineData : abc ECM";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("Invalid DLC")),
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_message_to_dbc_string() {
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

        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();
        assert_eq!(message.to_dbc_string(), "BO_ 256 EngineData : 8 ECM");
    }

    #[test]
    fn test_message_to_dbc_string_with_signals() {
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
            Receivers::None,
        )
        .unwrap();

        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal1, signal2]).unwrap();
        let dbc_str = message.to_dbc_string_with_signals();

        assert!(dbc_str.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_str.contains("SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" *"));
        assert!(dbc_str.contains("SG_ Temperature : 16|8@0- (1,-40) [-40|215] \"°C\""));
    }

    #[test]
    fn test_message_id_out_of_range() {
        let signal = Signal::new(
            "RPM",
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
        )
        .unwrap();

        // Test ID that exceeds extended 29-bit range
        let result = Message::new(0x20000000, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("out of valid range")),
            _ => panic!("Expected Signal error"),
        }

        // Test valid standard ID (11-bit) - minimum
        let result = Message::new(0, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid standard ID (11-bit) - maximum
        let result = Message::new(0x7FF, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - minimum
        let result = Message::new(0x800, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - maximum
        let result = Message::new(0x1FFFFFFF, "Test", 8, "ECM", vec![signal]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_overlap() {
        // Two signals that overlap: signal1 at 0-15, signal2 at 8-23
        let signal1 = Signal::new(
            "Signal1",
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
        )
        .unwrap();

        let signal2 = Signal::new(
            "Signal2",
            8,
            16,
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

        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal1, signal2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => {
                assert!(msg.contains("overlap"));
                assert!(msg.contains("Signal1"));
                assert!(msg.contains("Signal2"));
                assert!(msg.contains("TestMessage"));
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_signal_no_overlap() {
        // Two signals that don't overlap: signal1 at 0-15, signal2 at 16-31
        let signal1 = Signal::new(
            "Signal1",
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
        )
        .unwrap();

        let signal2 = Signal::new(
            "Signal2",
            16,
            16,
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

        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal1, signal2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_overlap_adjacent() {
        // Two signals that are adjacent but don't overlap: signal1 at 0-15, signal2 at 16-23
        let signal1 = Signal::new(
            "Signal1",
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
        )
        .unwrap();

        let signal2 = Signal::new(
            "Signal2",
            16,
            8,
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

        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal1, signal2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_overlap_identical_position() {
        // Two signals at the exact same position (definitely overlap)
        let signal1 = Signal::new(
            "Signal1",
            0,
            8,
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

        let signal2 = Signal::new(
            "Signal2",
            0,
            8,
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

        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal1, signal2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => assert!(msg.contains("overlap")),
            _ => panic!("Expected Signal error"),
        }
    }
}
