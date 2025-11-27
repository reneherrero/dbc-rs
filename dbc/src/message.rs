use crate::byte_order::ByteOrder;
use crate::{Error, Result, Signal, error::messages};
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};
use core::{
    convert::{AsRef, Into},
    iter::IntoIterator,
    option::{
        Option,
        Option::{None, Some},
    },
    result::Result::{Err, Ok},
};

/// Calculate the actual bit range (LSB to MSB) for a signal based on its byte order.
///
/// For little-endian signals: `start_bit` is the LSB, signal extends forward
/// For big-endian signals: `start_bit` is the MSB, signal extends backward
///
/// Returns (lsb, msb) where lsb <= msb
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

/// Represents a CAN message within a DBC file.
///
/// A message contains a CAN ID, name, data length code (DLC), sender node,
/// and a list of signals that make up the message payload.
///
/// # CAN Protocol Support
///
/// The library supports multiple CAN protocol variants:
/// - **Classic CAN Standard (CAN 2.0A)**: DLC 1-8 bytes (64 bits maximum payload)
/// - **Classic CAN Extended (CAN 2.0B)**: DLC 1-8 bytes (64 bits maximum payload)
/// - **CAN FD (Flexible Data Rate, ISO/Bosch)**: DLC 1-64 bytes (512 bits maximum payload)
///
/// All signals must fit within the message boundary: `DLC * 8 bits`.
///
/// # Limits
///
/// For security reasons (`DoS` protection), the maximum number of signals
/// per message is **64**. Attempting to create a [`Message`] instance with
/// more than 64 signals will result in a validation error.
///
/// # Examples
///
/// ```rust
/// use dbc_rs::Message;
///
/// let message = Message::builder()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
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
    #[allow(clippy::similar_names)] // Overlap detection uses intentionally similar variable names (sig1_lsb/sig1_msb, sig2_lsb/sig2_msb)
    fn validate(id: u32, name: &str, dlc: u8, sender: &str, signals: &[Signal]) -> Result<()> {
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
        if signals.len() > MAX_SIGNALS_PER_MESSAGE {
            return Err(Error::Message(
                messages::MESSAGE_TOO_MANY_SIGNALS.to_string(),
            ));
        }

        if name.trim().is_empty() {
            return Err(Error::Message(messages::MESSAGE_NAME_EMPTY.to_string()));
        }

        if sender.trim().is_empty() {
            return Err(Error::Message(messages::MESSAGE_SENDER_EMPTY.to_string()));
        }

        // Validate DLC (Data Length Code): must be between 1 and 64 bytes
        // - Classic CAN Standard (CAN 2.0A): DLC <= 8 bytes (64 bits) maximum payload
        // - Classic CAN Extended (CAN 2.0B): DLC <= 8 bytes (64 bits) maximum payload
        // - CAN FD (Flexible Data Rate, ISO/Bosch): DLC <= 64 bytes (512 bits) maximum payload
        if dlc == 0 {
            return Err(Error::Message(messages::MESSAGE_DLC_TOO_SMALL.to_string()));
        }
        if dlc > 64 {
            return Err(Error::Message(messages::MESSAGE_DLC_TOO_LARGE.to_string()));
        }

        // Validate that ID is within valid CAN ID range
        // Extended CAN IDs can be 0x00000000 to 0x1FFFFFFF (0 to 536870911)
        // IDs exceeding 0x1FFFFFFF are invalid
        if id > MAX_EXTENDED_ID {
            return Err(Error::Message(messages::message_id_out_of_range(id)));
        }

        // Validate that all signals fit within the message boundary
        // Each signal must fit within: DLC * 8 bits
        // - Classic CAN (2.0A/2.0B): DLC * 8 <= 64 bits (8 bytes)
        // - CAN FD: DLC * 8 <= 512 bits (64 bytes)
        // This ensures no signal extends beyond the message payload capacity
        let max_bits = u16::from(dlc) * 8;
        for signal in signals {
            // Calculate the actual bit range for this signal (accounting for byte order)
            let (lsb, msb) =
                calculate_bit_range(signal.start_bit(), signal.length(), signal.byte_order());
            // Check if the signal extends beyond the message boundary
            // The signal's highest bit position must be less than max_bits
            let signal_max_bit = lsb.max(msb);
            if signal_max_bit >= max_bits {
                return Err(Error::Message(messages::signal_extends_beyond_message(
                    signal.name(),
                    signal.start_bit(),
                    signal.length(),
                    signal_max_bit + 1, // end_bit (exclusive, for error message)
                    max_bits,
                    dlc,
                )));
            }
        }

        // Validate signal overlap detection
        // Check if any two signals overlap in the same message
        // Must account for byte order: little-endian signals extend forward,
        // big-endian signals extend backward from start_bit
        for (i, sig1) in signals.iter().enumerate() {
            let (sig1_lsb, sig1_msb) =
                calculate_bit_range(sig1.start_bit(), sig1.length(), sig1.byte_order());

            for sig2 in signals.iter().skip(i + 1) {
                let (sig2_lsb, sig2_msb) =
                    calculate_bit_range(sig2.start_bit(), sig2.length(), sig2.byte_order());

                // Check if ranges overlap
                // Two ranges [lsb1, msb1] and [lsb2, msb2] overlap if:
                // lsb1 <= msb2 && lsb2 <= msb1
                if sig1_lsb <= sig2_msb && sig2_lsb <= sig1_msb {
                    return Err(Error::Message(messages::signal_overlap(
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
    /// This is an internal constructor. For public API usage, use [`Message::builder()`] instead.
    pub(crate) fn new(
        id: u32,
        name: impl AsRef<str>,
        dlc: u8,
        sender: impl AsRef<str>,
        signals: Vec<Signal>,
    ) -> Result<Self> {
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

    /// Create a new builder for constructing a `Message`
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::{Message, Signal, ByteOrder, Receivers};
    ///
    /// let signal = Signal::builder()
    ///     .name("RPM")
    ///     .start_bit(0)
    ///     .length(16)
    ///     .build()?;
    ///
    /// let message = Message::builder()
    ///     .id(256)
    ///     .name("EngineData")
    ///     .dlc(8)
    ///     .sender("ECM")
    ///     .add_signal(signal)
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub(super) fn parse(message: &str, signals: Vec<Signal>) -> Result<Self> {
        // Split by whitespace and colon to normalize both formats:
        // - Standard: "BO_ <id> <name> : <dlc> <sender>" -> 5 parts
        // - Real-world: "BO_ <id> <name>: <dlc> <sender>" -> 5 parts
        let parts: Vec<&str> = message
            .split(|c: char| c.is_whitespace() || c == ':')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() != 5 {
            return Err(Error::Message(messages::MESSAGE_INVALID_FORMAT.to_string()));
        }

        let id = parts[1]
            .parse()
            .map_err(|_| Error::Message(messages::MESSAGE_INVALID_ID.to_string()))?;
        let name = parts[2];
        let dlc = parts[3]
            .parse()
            .map_err(|_| Error::Message(messages::MESSAGE_INVALID_DLC.to_string()))?;
        let sender = parts[4];

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
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the message name
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the Data Length Code (DLC)
    #[inline]
    #[must_use]
    pub fn dlc(&self) -> u8 {
        self.dlc
    }

    /// Get the sender node name
    #[inline]
    #[must_use]
    pub fn sender(&self) -> &str {
        &self.sender
    }

    /// Get a read-only slice of signals in this message
    #[inline]
    #[must_use]
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    /// Find a signal by name in this message
    #[must_use]
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
    /// use dbc_rs::{Message, Signal, ByteOrder, Receivers};
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
    ///
    /// let message = Message::builder()
    ///     .id(256)
    ///     .name("EngineData")
    ///     .dlc(8)
    ///     .sender("ECM")
    ///     .add_signal(signal)
    ///     .build()?;
    /// assert_eq!(message.to_dbc_string(), "BO_ 256 EngineData : 8 ECM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
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
    /// use dbc_rs::{Message, Signal, ByteOrder, Receivers};
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
    ///
    /// let message = Message::builder()
    ///     .id(256)
    ///     .name("EngineData")
    ///     .dlc(8)
    ///     .sender("ECM")
    ///     .add_signal(signal)
    ///     .build()?;
    /// let dbc_str = message.to_dbc_string_with_signals();
    /// assert!(dbc_str.contains("BO_ 256 EngineData : 8 ECM"));
    /// assert!(dbc_str.contains("SG_ RPM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
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

/// Builder for constructing a `Message` with a fluent API
///
/// This builder provides a more ergonomic way to construct `Message` instances,
/// especially when building messages with multiple signals.
///
/// # Examples
///
/// ```
/// use dbc_rs::{Message, Signal, ByteOrder, Receivers};
///
/// // Message with no signals
/// let message = Message::builder()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .build()?;
///
/// // Message with multiple signals
/// let signal1 = Signal::builder()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .build()?;
///
/// let signal2 = Signal::builder()
///     .name("Temperature")
///     .start_bit(16)
///     .length(8)
///     .build()?;
///
/// let message = Message::builder()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .add_signal(signal1)
///     .add_signal(signal2)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    id: Option<u32>,
    name: Option<Box<str>>,
    dlc: Option<u8>,
    sender: Option<Box<str>>,
    signals: Vec<Signal>,
}

impl MessageBuilder {
    fn new() -> Self {
        Self {
            id: None,
            name: None,
            dlc: None,
            sender: None,
            signals: Vec::new(),
        }
    }

    /// Set the CAN message ID (required)
    #[must_use]
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the message name (required)
    #[must_use]
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().into());
        self
    }

    /// Set the Data Length Code (DLC) (required)
    #[must_use]
    pub fn dlc(mut self, dlc: u8) -> Self {
        self.dlc = Some(dlc);
        self
    }

    /// Set the sender node name (required)
    #[must_use]
    pub fn sender(mut self, sender: impl AsRef<str>) -> Self {
        self.sender = Some(sender.as_ref().into());
        self
    }

    /// Add a signal to the message
    #[must_use]
    pub fn add_signal(mut self, signal: Signal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Add multiple signals to the message
    #[must_use]
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = Signal>) -> Self {
        self.signals.extend(signals);
        self
    }

    /// Set all signals at once (replaces any existing signals)
    #[must_use]
    pub fn signals(mut self, signals: Vec<Signal>) -> Self {
        self.signals = signals;
        self
    }

    /// Clear all signals
    #[must_use]
    pub fn clear_signals(mut self) -> Self {
        self.signals.clear();
        self
    }

    /// Validate the current builder state
    ///
    /// This method performs the same validation as `Message::validate()` but on the
    /// builder's current state. Useful for checking validity before calling `build()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields (`id`, `name`, `dlc`, `sender`) are missing
    /// - Validation fails (same as `Message::validate()`)
    pub fn validate(&self) -> Result<()> {
        let id = self
            .id
            .ok_or_else(|| Error::Message(messages::MESSAGE_ID_REQUIRED.to_string()))?;
        let name = self
            .name
            .as_ref()
            .ok_or_else(|| Error::Message(messages::MESSAGE_NAME_EMPTY.to_string()))?;
        let dlc = self
            .dlc
            .ok_or_else(|| Error::Message(messages::MESSAGE_DLC_REQUIRED.to_string()))?;
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| Error::Message(messages::MESSAGE_SENDER_EMPTY.to_string()))?;

        Message::validate(id, name.as_ref(), dlc, sender.as_ref(), &self.signals)
    }

    /// Build the `Message` from the builder
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields (`id`, `name`, `dlc`, `sender`) are missing
    /// - Validation fails (same validation logic as the internal constructor)
    pub fn build(self) -> Result<Message> {
        let id = self
            .id
            .ok_or_else(|| Error::Message(messages::MESSAGE_ID_REQUIRED.to_string()))?;
        let name = self
            .name
            .ok_or_else(|| Error::Message(messages::MESSAGE_NAME_EMPTY.to_string()))?;
        let dlc = self
            .dlc
            .ok_or_else(|| Error::Message(messages::MESSAGE_DLC_REQUIRED.to_string()))?;
        let sender = self
            .sender
            .ok_or_else(|| Error::Message(messages::MESSAGE_SENDER_EMPTY.to_string()))?;

        Message::new(id, name.as_ref(), dlc, sender.as_ref(), self.signals)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::error::lang;
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
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_NAME_EMPTY)),
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
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_SENDER_EMPTY)),
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
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_SMALL)),
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

        // DLC > 64 should fail (CAN FD maximum is 64 bytes)
        let result = Message::new(256, "EngineData", 65, "ECM", vec![signal]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_LARGE)),
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_new_signal_overflow() {
        // Signal extends beyond DLC boundary
        // Test with a signal that fits Signal validation (length <= 512) but exceeds message DLC
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
        // Test that parse also validates DLC (CAN FD maximum is 64 bytes)
        let line = "BO_ 256 EngineData : 65 ECM";
        let signals = vec![];
        let result = Message::parse(line, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_LARGE)),
            _ => panic!("Expected Message error"),
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
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_DLC_TOO_SMALL)),
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
    fn test_message_parse_invalid_format() {
        // Test parse with too few parts (only 3 parts)
        let line = "BO_ 256 EngineData";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_FORMAT)),
            _ => panic!("Expected Message error"),
        }

        // Test parse with too many parts (7 parts)
        let line = "BO_ 256 EngineData : 8 ECM extra";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_FORMAT)),
            _ => panic!("Expected Message error"),
        }
    }

    #[test]
    fn test_message_parse_invalid_id() {
        // Test parse with invalid message ID
        let line = "BO_ abc EngineData : 8 ECM";
        let result = Message::parse(line, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_ID)),
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
            Error::Message(msg) => assert!(msg.contains(lang::MESSAGE_INVALID_DLC)),
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
        let result = Message::new(0x2000_0000, "Test", 8, "ECM", vec![signal.clone()]);
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
        let result = Message::new(0, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid standard ID (11-bit) - maximum
        let result = Message::new(0x7FF, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - minimum
        let result = Message::new(0x800, "Test", 8, "ECM", vec![signal.clone()]);
        assert!(result.is_ok());

        // Test valid extended ID (29-bit) - maximum
        let result = Message::new(0x1FFF_FFFF, "Test", 8, "ECM", vec![signal]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_overlap() {
        // Two big-endian signals that overlap
        // Signal1: BE start=0, length=16 -> physical range [7, 17] (MSB at 7, LSB at 17)
        // Signal2: BE start=1, length=16 -> physical range [6, 18] (MSB at 6, LSB at 18)
        // They overlap in physical bits [7, 17]
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
            1,
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
            let signal = Signal::new(
                format!("Signal{i}").as_str(),
                start_bit,
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
            signals.push(signal);
        }
        let result = Message::new(256, "TestMessage", 8, "ECM", signals);
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
            let signal = Signal::new(
                format!("Signal{i}").as_str(),
                i,
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
            signals.push(signal);
        }
        let result = Message::new(256, "TestMessage", 8, "ECM", signals);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().signals().len(), 64);
    }

    #[test]
    fn test_message_can_2_0a_dlc_limits() {
        // Classic CAN Standard (CAN 2.0A): DLC 1-8 bytes
        // Note: The library supports all CAN variants (2.0A, 2.0B, CAN FD),
        // so DLC 1-64 is valid. This test verifies CAN 2.0A typical usage.
        let signal = Signal::new(
            "TestSignal",
            0,
            64, // 64 bits = 8 bytes, fits exactly in DLC=8
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // DLC=8 should work (64 bits) - typical CAN 2.0A maximum
        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal]);
        assert!(result.is_ok(), "CAN 2.0A should support DLC=8 (64 bits)");
    }

    #[test]
    fn test_message_can_2_0b_dlc_limits() {
        // Classic CAN Extended (CAN 2.0B): DLC 1-8 bytes (same as 2.0A)
        let signal = Signal::new(
            "TestSignal",
            0,
            32, // 32 bits = 4 bytes
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // DLC=8 should work (64 bits)
        let result = Message::new(0x800, "TestMessage", 8, "ECM", vec![signal]);
        assert!(result.is_ok(), "CAN 2.0B should support DLC=8 (64 bits)");
    }

    #[test]
    fn test_message_can_fd_dlc_limits() {
        // CAN FD (Flexible Data Rate): DLC 1-64 bytes (512 bits)
        // Test maximum DLC for CAN FD
        let signal = Signal::new(
            "TestSignal",
            0,
            64, // 64 bits = 8 bytes
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // DLC=64 should work (512 bits = 64 bytes)
        let result = Message::new(256, "TestMessage", 64, "ECM", vec![signal.clone()]);
        assert!(result.is_ok(), "CAN FD should support DLC=64 (512 bits)");

        // DLC=65 should fail (exceeds CAN FD limit)
        let result = Message::new(256, "TestMessage", 65, "ECM", vec![signal]);
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
        let signal = Signal::new(
            "TestSignal",
            56, // Start at bit 56
            8,  // 8 bits, ends at bit 63 (fits in 8 bytes)
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // Should work: signal fits in DLC=8 (64 bits)
        let result = Message::new(256, "TestMessage", 8, "ECM", vec![signal.clone()]);
        assert!(
            result.is_ok(),
            "Signal should fit in CAN 2.0A message (DLC=8)"
        );

        // Should fail: signal extends beyond DLC=7 (56 bits)
        let result = Message::new(256, "TestMessage", 7, "ECM", vec![signal]);
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
        let signal = Signal::new(
            "TestSignal",
            248, // Start at bit 248 (max u8 is 255)
            8,   // 8 bits, ends at bit 255 (fits in 32 bytes = 256 bits)
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // Should work: signal fits in DLC=32 (256 bits)
        let result = Message::new(256, "TestMessage", 32, "ECM", vec![signal.clone()]);
        assert!(
            result.is_ok(),
            "Signal should fit in CAN FD message (DLC=32)"
        );

        // Should fail: signal extends beyond DLC=31 (248 bits)
        let result = Message::new(256, "TestMessage", 31, "ECM", vec![signal]);
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
        let signal1 = Signal::new(
            "Signal1",
            0,
            32, // 32 bits
            ByteOrder::LittleEndian,
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
            32,
            32, // 32 bits, total = 64 bits
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // Should work: both signals fit in DLC=8 (64 bits total)
        let result = Message::new(
            256,
            "TestMessage",
            8,
            "ECM",
            vec![signal1.clone(), signal2.clone()],
        );
        assert!(result.is_ok(), "Both signals should fit in DLC=8");

        // Should fail: signals don't fit in DLC=7 (56 bits)
        let result = Message::new(256, "TestMessage", 7, "ECM", vec![signal1, signal2]);
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
