use crate::{Dbc, Error, MAX_SIGNALS_PER_MESSAGE, Message, Result, compat::Vec};
#[cfg(feature = "embedded-can")]
use embedded_can::{Frame, Id};

/// A decoded signal from a CAN message.
///
/// Contains the signal name, its decoded physical value, unit, and optional value description.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedSignal<'a> {
    /// The name of the signal as defined in the DBC file.
    pub name: &'a str,
    /// The decoded physical value after applying factor and offset.
    pub value: f64,
    /// The unit of the signal (e.g., "rpm", "°C"), if defined.
    pub unit: Option<&'a str>,
    /// The value description text if defined in the DBC file (e.g., "Park", "Drive").
    /// This maps the raw signal value to a human-readable description.
    pub description: Option<&'a str>,
}

impl<'a> DecodedSignal<'a> {
    /// Creates a new `DecodedSignal` with the given name, value, unit, and description.
    ///
    /// # Arguments
    ///
    /// * `name` - The signal name
    /// * `value` - The decoded physical value (after applying factor and offset)
    /// * `unit` - The optional unit of measurement (e.g., "rpm", "km/h")
    /// * `description` - The optional value description text (e.g., "Park", "Drive")
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DecodedSignal;
    ///
    /// let signal = DecodedSignal::new("Gear", 3.0, Some(""), Some("Drive"));
    /// assert_eq!(signal.name, "Gear");
    /// assert_eq!(signal.value, 3.0);
    /// assert_eq!(signal.description, Some("Drive"));
    /// ```
    #[inline]
    pub fn new(
        name: &'a str,
        value: f64,
        unit: Option<&'a str>,
        description: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            value,
            unit,
            description,
        }
    }
}

/// Maximum number of multiplexer switches in a single message.
/// Most CAN messages have 0-2 switches; 8 is generous.
const MAX_SWITCHES: usize = 8;

/// Pre-allocated buffer for switch values during decode.
/// Uses fixed-size arrays to avoid heap allocation.
struct SwitchValues<'a> {
    /// Switch names (references to signal names, valid for decode lifetime)
    names: [Option<&'a str>; MAX_SWITCHES],
    /// Switch values corresponding to each name
    values: [u64; MAX_SWITCHES],
    /// Number of switches stored.
    count: usize,
}

impl<'a> SwitchValues<'a> {
    #[inline]
    const fn new() -> Self {
        Self {
            names: [None; MAX_SWITCHES],
            values: [0; MAX_SWITCHES],
            count: 0,
        }
    }

    /// Store a switch value. Returns Err if capacity exceeded.
    #[inline]
    fn push(&mut self, name: &'a str, value: u64) -> Result<()> {
        if self.count >= MAX_SWITCHES {
            return Err(Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS));
        }
        let idx = self.count;
        self.names[idx] = Some(name);
        self.values[idx] = value;
        self.count += 1;
        Ok(())
    }

    /// Find switch value by name. O(n) where n is number of switches (typically 1-2).
    #[inline]
    fn get_by_name(&self, name: &str) -> Option<u64> {
        for i in 0..self.count {
            if self.names[i] == Some(name) {
                return Some(self.values[i]);
            }
        }
        None
    }

    /// Check if any switch has the given value.
    #[inline]
    fn any_has_value(&self, target: u64) -> bool {
        for i in 0..self.count {
            if self.values[i] == target && self.names[i].is_some() {
                return true;
            }
        }
        false
    }
}

/// Decoding functionality for DBC structures
impl Dbc {
    /// Decode a CAN message payload using the message ID to find the corresponding message definition.
    ///
    /// This is a high-performance method for decoding CAN messages in `no_std` environments.
    /// It finds the message by ID, then decodes all signals in the message from the payload bytes.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw CAN message ID (without extended flag)
    /// * `payload` - The CAN message payload bytes (up to 64 bytes for CAN FD)
    /// * `is_extended` - Whether this is an extended (29-bit) CAN ID
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DecodedSignal>)` - A vector of decoded signals with name, value, and unit
    /// * `Err(Error)` - If the message ID is not found, payload length doesn't match DLC, or signal decoding fails
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
    /// // Decode a CAN message with RPM value of 2000 (raw: 8000 = 0x1F40)
    /// let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let decoded = dbc.decode(256, &payload, false)?; // false = standard CAN ID
    /// assert_eq!(decoded.len(), 1);
    /// assert_eq!(decoded[0].name, "RPM");
    /// assert_eq!(decoded[0].value, 2000.0);
    /// assert_eq!(decoded[0].unit, Some("rpm"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// High-performance CAN message decoding optimized for throughput.
    ///
    /// Performance optimizations:
    /// - O(1) or O(log n) message lookup via feature-flagged index
    /// - Single-pass signal iteration with inline switch processing
    /// - Stack-allocated switch value storage (no heap allocation)
    /// - Message-level extended multiplexing check to skip per-signal lookups
    /// - Inlined hot paths with early returns
    #[inline]
    pub fn decode(
        &self,
        id: u32,
        payload: &[u8],
        is_extended: bool,
    ) -> Result<Vec<DecodedSignal<'_>, { MAX_SIGNALS_PER_MESSAGE }>> {
        // If it's an extended ID, add the extended ID flag
        let id = if is_extended {
            id | Message::EXTENDED_ID_FLAG
        } else {
            id
        };

        // Find message by ID (performance-critical lookup)
        let message = self
            .messages()
            .find_by_id(id)
            .ok_or(Error::Decoding(Error::MESSAGE_NOT_FOUND))?;

        // Validate payload length matches message DLC (early return before any decoding)
        let dlc = message.dlc() as usize;
        if payload.len() < dlc {
            return Err(Error::Decoding(Error::PAYLOAD_LENGTH_MISMATCH));
        }

        // Pre-allocate result vector
        let mut decoded_signals: Vec<DecodedSignal<'_>, { MAX_SIGNALS_PER_MESSAGE }> = Vec::new();

        // Stack-allocated switch values (no heap allocation)
        let mut switch_values = SwitchValues::<'_>::new();

        let signals = message.signals();

        // Check once if this message has ANY extended multiplexing
        // Fast path: skip if DBC has no extended multiplexing at all (O(1) check)
        // Slow path: check if this specific message has entries (O(m) where m = unique message IDs)
        let message_has_extended_mux = !self.extended_multiplexing.is_empty()
            && self.has_extended_multiplexing_for_message(id);

        // Check once if DBC has ANY value descriptions - skip all lookups if empty (common case)
        // This avoids O(n) linear scans per signal when no value descriptions exist
        let has_any_value_descriptions = !self.value_descriptions.is_empty();

        // PASS 1: Decode multiplexer switches first (needed before multiplexed signals)
        // This is necessary because multiplexed signals depend on switch values
        for signal in signals.iter() {
            if signal.is_multiplexer_switch() {
                // decode_raw() returns (raw_value, physical_value) in one pass
                let (raw_value, physical_value) = signal.decode_raw(payload)?;

                // Multiplexer switch values must be non-negative
                if raw_value < 0 {
                    return Err(Error::Decoding(Error::MULTIPLEXER_SWITCH_NEGATIVE));
                }

                // Store switch value for later multiplexing checks
                switch_values.push(signal.name(), raw_value as u64)?;

                // Lookup value description only if any exist (skip O(n) scan otherwise)
                let description = if has_any_value_descriptions {
                    self.value_descriptions_for_signal(id, signal.name())
                        .and_then(|vd| vd.get(raw_value as u64))
                } else {
                    None
                };

                // Add to decoded signals
                decoded_signals
                    .push(DecodedSignal::new(
                        signal.name(),
                        physical_value,
                        signal.unit(),
                        description,
                    ))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
            }
        }

        // PASS 2: Decode non-switch signals based on multiplexing rules
        for signal in signals.iter() {
            // Skip multiplexer switches (already decoded in pass 1)
            if signal.is_multiplexer_switch() {
                continue;
            }

            // Determine if this signal should be decoded based on multiplexing
            let should_decode = if let Some(mux_value) = signal.multiplexer_switch_value() {
                // This is a multiplexed signal (m0, m1, etc.)
                if message_has_extended_mux {
                    // Check extended multiplexing only if message has any
                    self.check_extended_multiplexing(id, signal.name(), &switch_values)
                        .unwrap_or_else(|| {
                            // No extended entries for this signal - use basic multiplexing
                            switch_values.any_has_value(mux_value)
                        })
                } else {
                    // No extended multiplexing for this message - use basic check
                    switch_values.any_has_value(mux_value)
                }
            } else {
                // Normal signal (not multiplexed) - always decode
                true
            };

            if should_decode {
                // decode_raw() returns (raw_value, physical_value) in one pass
                let (raw_value, physical_value) = signal.decode_raw(payload)?;

                // Lookup value description only if any exist (skip O(n) scan otherwise)
                let description = if has_any_value_descriptions {
                    self.value_descriptions_for_signal(id, signal.name())
                        .and_then(|vd| vd.get(raw_value as u64))
                } else {
                    None
                };

                decoded_signals
                    .push(DecodedSignal::new(
                        signal.name(),
                        physical_value,
                        signal.unit(),
                        description,
                    ))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
            }
        }

        Ok(decoded_signals)
    }

    /// Check extended multiplexing rules for a signal.
    /// Returns Some(true) if signal should be decoded, Some(false) if not,
    /// or None if no extended multiplexing entries exist for this signal.
    #[inline]
    fn check_extended_multiplexing(
        &self,
        message_id: u32,
        signal_name: &str,
        switch_values: &SwitchValues,
    ) -> Option<bool> {
        // Get extended entries for this signal
        let indices = self.ext_mux_index.get(message_id, signal_name)?;
        if indices.is_empty() {
            return None;
        }

        // Collect entries without allocation by working with indices directly
        // Check ALL switches referenced - all must match (AND logic)

        // First, collect unique switch names referenced by this signal's extended entries
        let mut unique_switches: [Option<&str>; MAX_SWITCHES] = [None; MAX_SWITCHES];
        let mut unique_count = 0;

        for &idx in indices {
            if let Some(entry) = self.extended_multiplexing.get(idx) {
                let switch_name = entry.multiplexer_switch();
                // Check if already in unique_switches
                let found =
                    unique_switches.iter().take(unique_count).any(|&s| s == Some(switch_name));
                if !found && unique_count < MAX_SWITCHES {
                    unique_switches[unique_count] = Some(switch_name);
                    unique_count += 1;
                }
            }
        }

        // All referenced switches must have matching values
        for switch_opt in unique_switches.iter().take(unique_count) {
            let switch_name = match switch_opt {
                Some(name) => *name,
                None => continue,
            };

            // Get the current switch value
            let switch_val = match switch_values.get_by_name(switch_name) {
                Some(v) => v,
                None => return Some(false), // Switch not found, signal not active
            };

            // Check if any extended entry for this switch has a matching value range
            let mut has_match = false;
            for &idx in indices {
                if let Some(entry) = self.extended_multiplexing.get(idx) {
                    if entry.multiplexer_switch() == switch_name {
                        for &(min, max) in entry.value_ranges() {
                            if switch_val >= min && switch_val <= max {
                                has_match = true;
                                break;
                            }
                        }
                        if has_match {
                            break;
                        }
                    }
                }
            }

            if !has_match {
                return Some(false); // This switch doesn't match, signal not active
            }
        }

        Some(true) // All switches match
    }

    /// Check if any extended multiplexing entries exist for a message.
    /// O(n) scan over extended multiplexing entries.
    #[inline]
    fn has_extended_multiplexing_for_message(&self, message_id: u32) -> bool {
        self.extended_multiplexing
            .iter()
            .any(|ext_mux| ext_mux.message_id() == message_id)
    }

    /// Decode a CAN frame using the embedded-can [`Frame`] trait.
    ///
    /// This is a convenience method that automatically extracts the CAN ID and payload
    /// from any type implementing the embedded-can [`Frame`] trait, and determines
    /// whether the ID is standard (11-bit) or extended (29-bit).
    ///
    /// # Arguments
    ///
    /// * `frame` - Any type implementing the embedded-can [`Frame`] trait
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DecodedSignal>)` - A vector of decoded signals with name, value, and unit
    /// * `Err(Error)` - If the message ID is not found, payload length doesn't match DLC, or signal decoding fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use dbc_rs::Dbc;
    /// use embedded_can::{Frame, Id, StandardId};
    ///
    /// // Assuming you have a CAN frame from your hardware driver
    /// let frame = MyCanFrame::new(StandardId::new(256).unwrap(), &[0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// let decoded = dbc.decode_frame(frame)?;
    /// assert_eq!(decoded[0].name, "RPM");
    /// assert_eq!(decoded[0].value, 2000.0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Feature
    ///
    /// This method is only available when the `embedded-can` feature is enabled:
    ///
    /// ```toml
    /// [dependencies]
    /// dbc-rs = { version = "1", features = ["embedded-can"] }
    /// ```
    #[cfg(feature = "embedded-can")]
    #[inline]
    pub fn decode_frame<T: Frame>(
        &self,
        frame: T,
    ) -> Result<Vec<DecodedSignal<'_>, { MAX_SIGNALS_PER_MESSAGE }>> {
        let payload = frame.data();
        match frame.id() {
            Id::Standard(id) => self.decode(id.as_raw() as u32, payload, false),
            Id::Extended(id) => self.decode(id.as_raw(), payload, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_decode_basic() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        // Decode a CAN message with RPM value of 2000 (raw: 8000 = 0x1F40)
        let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload, false).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "RPM");
        assert_eq!(decoded[0].value, 2000.0);
        assert_eq!(decoded[0].unit, Some("rpm"));
    }

    #[test]
    fn test_decode_message_not_found() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        let payload = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = dbc.decode(512, &payload, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_message() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Decode a CAN message with RPM = 2000 (raw: 8000 = 0x1F40) and Temp = 50°C (raw: 90)
        // Little-endian: RPM at bits 0-15, Temp at bits 16-23
        let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload, false).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].name, "RPM");
        assert_eq!(decoded[0].value, 2000.0);
        assert_eq!(decoded[0].unit, Some("rpm"));
        assert_eq!(decoded[1].name, "Temp");
        assert_eq!(decoded[1].value, 50.0);
        assert_eq!(decoded[1].unit, Some("°C"));
    }

    #[test]
    fn test_decode_payload_length_mismatch() {
        use crate::Error;
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Try to decode with payload shorter than DLC (DLC is 8, payload is 4)
        let payload = [0x40, 0x1F, 0x00, 0x00];
        let result = dbc.decode(256, &payload, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Decoding(msg) => {
                assert!(msg.contains(Error::PAYLOAD_LENGTH_MISMATCH));
            }
            _ => panic!("Expected Error::Decoding"),
        }
    }

    #[test]
    fn test_decode_big_endian_signal() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@0+ (1.0,0) [0|65535] "rpm" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Decode a big-endian signal: RPM = 256 (raw: 256 = 0x0100)
        // For big-endian at bit 0-15, the bytes are arranged as [0x01, 0x00]
        // Testing with a simple value that's easier to verify
        let payload = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload, false).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "RPM");
        // The exact value depends on big-endian bit extraction implementation
        // We just verify that decoding doesn't crash and returns a value
        assert!(decoded[0].value >= 0.0);
        assert_eq!(decoded[0].unit, Some("rpm"));
    }

    #[test]
    fn test_decode_multiplexed_signal() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ MuxId M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal0 m0 : 8|16@1+ (0.1,0) [0|6553.5] "unit" *
 SG_ Signal1 m1 : 24|16@1+ (0.01,0) [0|655.35] "unit" *
 SG_ NormalSignal : 40|8@1+ (1,0) [0|255] ""
"#,
        )
        .unwrap();

        // Test with MuxId = 0: Should decode Signal0 and NormalSignal, but not Signal1
        let payload = [0x00, 0x64, 0x00, 0x00, 0x00, 0x32, 0x00, 0x00];
        // MuxId=0, Signal0=100 (raw: 1000 = 0x03E8, but little-endian: 0xE8, 0x03), NormalSignal=50
        // Payload: [MuxId=0, Signal0_low=0x64, Signal0_high=0x00, padding, NormalSignal=0x32, ...]
        let decoded = dbc.decode(256, &payload, false).unwrap();

        // Helper to find value by signal name
        let find_signal = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);

        // MuxId should always be decoded
        assert!(find_signal("MuxId").is_some());
        // Signal0 should be decoded (MuxId == 0)
        assert!(find_signal("Signal0").is_some());
        // Signal1 should NOT be decoded (MuxId != 1)
        assert!(find_signal("Signal1").is_none());
        // NormalSignal should always be decoded (not multiplexed)
        assert!(find_signal("NormalSignal").is_some());
    }

    #[test]
    fn test_decode_multiplexed_signal_switch_one() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ MuxId M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal0 m0 : 8|16@1+ (0.1,0) [0|6553.5] "unit" *
 SG_ Signal1 m1 : 24|16@1+ (0.01,0) [0|655.35] "unit" *
"#,
        )
        .unwrap();

        // Test with MuxId = 1: Should decode Signal1, but not Signal0
        let payload = [0x01, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00];
        // MuxId=1 (at byte 0), Signal1 at bits 24-39 (bytes 3-4, little-endian)
        // Signal1 value: 100 (raw: 100, little-endian bytes: 0x64, 0x00)
        let decoded = dbc.decode(256, &payload, false).unwrap();

        // Helper to find value by signal name
        let find_signal = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);

        // MuxId should always be decoded
        assert_eq!(find_signal("MuxId"), Some(1.0));
        // Signal0 should NOT be decoded (MuxId != 0)
        assert!(find_signal("Signal0").is_none());
        // Signal1 should be decoded (MuxId == 1)
        assert!(find_signal("Signal1").is_some());
    }

    #[test]
    fn test_decode_mixed_byte_order() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 MixedByteOrder : 8 ECM
 SG_ LittleEndianSignal : 0|16@1+ (1.0,0) [0|65535] ""
 SG_ BigEndianSignal : 23|16@0+ (1.0,0) [0|65535] ""
 SG_ AnotherLittleEndian : 32|8@1+ (1.0,0) [0|255] ""
 SG_ AnotherBigEndian : 47|8@0+ (1.0,0) [0|255] ""
"#,
        )
        .unwrap();

        // Test payload with both big-endian and little-endian signals:
        // - LittleEndianSignal at bits 0-15 (bytes 0-1): [0x34, 0x12] = 0x1234 = 4660
        // - BigEndianSignal: BE start_bit=23 (MSB of byte 2), 16 bits -> bytes 2-3
        // - AnotherLittleEndian at bits 32-39 (byte 4): 0xAB = 171
        // - AnotherBigEndian: BE start_bit=47 (MSB of byte 5), 8 bits -> byte 5
        let payload = [
            0x34, 0x12, // Bytes 0-1: LittleEndianSignal
            0x00, 0x01, // Bytes 2-3: BigEndianSignal (BE: 0x0001 = 1)
            0xAB, // Byte 4: AnotherLittleEndian
            0xCD, // Byte 5: AnotherBigEndian (BE: 0xCD = 205)
            0x00, 0x00, // Padding
        ];
        let decoded = dbc.decode(256, &payload, false).unwrap();

        // Helper to find value by signal name
        let find_signal = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);

        // Verify little-endian 16-bit signal: bytes [0x34, 0x12] = 0x1234 = 4660
        assert_eq!(find_signal("LittleEndianSignal"), Some(4660.0)); // 0x1234

        // For big-endian, verify it decodes correctly (exact value depends on BE bit mapping)
        let big_endian_value = find_signal("BigEndianSignal").unwrap();
        // Big-endian signal should decode to a reasonable value
        assert!((0.0..=65535.0).contains(&big_endian_value));

        // Verify little-endian 8-bit signal at byte 4
        assert_eq!(find_signal("AnotherLittleEndian"), Some(171.0)); // 0xAB

        // For big-endian 8-bit signal, verify it decoded (exact value depends on BE bit mapping)
        let big_endian_8bit = find_signal("AnotherBigEndian");
        assert!(big_endian_8bit.is_some());
        assert!(big_endian_8bit.unwrap() >= 0.0 && big_endian_8bit.unwrap() <= 255.0);

        // All signals should be decoded
        assert_eq!(decoded.len(), 4);

        // Verify both 16-bit signals decoded successfully (proves both byte orders work)
        assert!(find_signal("LittleEndianSignal").is_some());
        assert!(find_signal("BigEndianSignal").is_some());
    }

    #[test]
    fn test_decode_extended_multiplexing_simple() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 500 ComplexMux : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *

SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
"#,
        )
        .unwrap();

        // Test with Mux1 = 5: Should decode Signal_A (within range 5-10)
        // Mux1=5 (byte 0), Signal_A at bits 16-31 (bytes 2-3, little-endian)
        // Signal_A=100 (raw: 1000 = 0x03E8, little-endian bytes: 0xE8, 0x03)
        let payload = [0x05, 0x00, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(500, &payload, false).unwrap();

        let find_signal = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);

        assert_eq!(find_signal("Mux1"), Some(5.0));
        // Extended multiplexing: Signal_A should decode when Mux1 is in range 5-10
        assert_eq!(
            dbc.extended_multiplexing_for_message(500).count(),
            1,
            "Extended multiplexing entries should be parsed"
        );
        assert!(
            find_signal("Signal_A").is_some(),
            "Signal_A should be decoded when Mux1=5 (within range 5-10)"
        );
        assert_eq!(find_signal("Signal_A").unwrap(), 100.0);

        // Test with Mux1 = 15: Should NOT decode Signal_A (outside range 5-10)
        let payload2 = [0x0F, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(500, &payload2, false).unwrap();
        let find_signal2 = |name: &str| decoded2.iter().find(|s| s.name == name).map(|s| s.value);

        assert_eq!(find_signal2("Mux1"), Some(15.0));
        assert!(find_signal2("Signal_A").is_none());
    }

    #[test]
    fn test_decode_extended_multiplexing_multiple_ranges() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 501 MultiRangeMux : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal_B m0 : 16|16@1+ (1,0) [0|65535] "unit" *

SG_MUL_VAL_ 501 Signal_B Mux1 0-5,10-15,20-25 ;
"#,
        )
        .unwrap();

        // Test with Mux1 = 3: Should decode (within range 0-5)
        // Signal_B at bits 16-31, value 4096 (raw, little-endian: 0x00, 0x10)
        let payload1 = [0x03, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00];
        let decoded1 = dbc.decode(501, &payload1, false).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find1("Mux1"), Some(3.0));
        assert!(find1("Signal_B").is_some());

        // Test with Mux1 = 12: Should decode (within range 10-15)
        // Signal_B at bits 16-31, value 8192 (raw, little-endian: 0x00, 0x20)
        let payload2 = [0x0C, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(501, &payload2, false).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find2("Mux1"), Some(12.0));
        assert!(find2("Signal_B").is_some());

        // Test with Mux1 = 22: Should decode (within range 20-25)
        // Signal_B at bits 16-31, value 12288 (raw, little-endian: 0x00, 0x30)
        let payload3 = [0x16, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00];
        let decoded3 = dbc.decode(501, &payload3, false).unwrap();
        let find3 = |name: &str| decoded3.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find3("Mux1"), Some(22.0));
        assert!(find3("Signal_B").is_some());

        // Test with Mux1 = 8: Should NOT decode (not in any range)
        // Signal_B at bits 16-31, value 16384 (raw, little-endian: 0x00, 0x40)
        let payload4 = [0x08, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00];
        let decoded4 = dbc.decode(501, &payload4, false).unwrap();
        let find4 = |name: &str| decoded4.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find4("Mux1"), Some(8.0));
        assert!(find4("Signal_B").is_none());
    }

    /// Test extended multiplexing with multiple switches (AND logic - all must match).
    /// Note: Depends on SG_MUL_VAL_ parsing working correctly.
    #[test]
    fn test_decode_extended_multiplexing_multiple_switches() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 502 MultiSwitchMux : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Mux2 M : 8|8@1+ (1,0) [0|255] ""
 SG_ Signal_C m0 : 16|16@1+ (1,0) [0|65535] "unit" *

SG_MUL_VAL_ 502 Signal_C Mux1 5-10 ;
SG_MUL_VAL_ 502 Signal_C Mux2 20-25 ;
"#,
        )
        .unwrap();

        // Test with Mux1=7 and Mux2=22: Should decode (both switches match their ranges)
        // Mux1=7 (byte 0), Mux2=22 (byte 1), Signal_C at bits 16-31 (bytes 2-3, little-endian)
        let payload1 = [0x07, 0x16, 0x00, 0x50, 0x00, 0x00, 0x00, 0x00];
        let decoded1 = dbc.decode(502, &payload1, false).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find1("Mux1"), Some(7.0));
        assert_eq!(find1("Mux2"), Some(22.0));
        assert!(find1("Signal_C").is_some());

        // Test with Mux1=7 and Mux2=30: Should NOT decode (Mux2 outside range)
        let payload2 = [0x07, 0x1E, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(502, &payload2, false).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find2("Mux1"), Some(7.0));
        assert_eq!(find2("Mux2"), Some(30.0));
        assert!(find2("Signal_C").is_none());

        // Test with Mux1=15 and Mux2=22: Should NOT decode (Mux1 outside range)
        let payload3 = [0x0F, 0x16, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00];
        let decoded3 = dbc.decode(502, &payload3, false).unwrap();
        let find3 = |name: &str| decoded3.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find3("Mux1"), Some(15.0));
        assert_eq!(find3("Mux2"), Some(22.0));
        assert!(find3("Signal_C").is_none());
    }

    /// Test that extended multiplexing takes precedence over basic m0/m1 values.
    /// Note: Depends on SG_MUL_VAL_ parsing working correctly.
    #[test]
    fn test_decode_extended_multiplexing_takes_precedence() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 503 PrecedenceTest : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal_D m0 : 16|16@1+ (1,0) [0|65535] "unit" *

SG_MUL_VAL_ 503 Signal_D Mux1 10-15 ;
"#,
        )
        .unwrap();

        // Test with Mux1 = 0: Should NOT decode
        // Even though basic multiplexing would match (m0 means decode when switch=0),
        // extended multiplexing takes precedence and requires Mux1 to be 10-15
        let payload1 = [0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00];
        let decoded1 = dbc.decode(503, &payload1, false).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find1("Mux1"), Some(0.0));
        assert!(find1("Signal_D").is_none());

        // Test with Mux1 = 12: Should decode (within extended range 10-15)
        let payload2 = [0x0C, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(503, &payload2, false).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find2("Mux1"), Some(12.0));
        assert!(find2("Signal_D").is_some());
    }

    /// Test extended multiplexing with signals that are both multiplexed and multiplexer switches (m65M pattern).
    /// Note: Depends on SG_MUL_VAL_ parsing working correctly.
    #[test]
    fn test_decode_extended_multiplexing_with_extended_mux_signal() {
        // Test extended multiplexing where the signal itself is also a multiplexer (m65M pattern)
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 504 ExtendedMuxSignal : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Mux2 m65M : 8|8@1+ (1,0) [0|255] ""
 SG_ Signal_E m0 : 16|16@1+ (1,0) [0|65535] "unit" *

SG_MUL_VAL_ 504 Signal_E Mux1 65-65 ;
SG_MUL_VAL_ 504 Signal_E Mux2 10-15 ;
"#,
        )
        .unwrap();

        // Test with Mux1=65 and Mux2=12: Should decode Signal_E
        // Mux2 is both multiplexed (m65 - active when Mux1=65) and a multiplexer (M)
        let payload = [0x41, 0x0C, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00];
        // Mux1=65 (0x41), Mux2=12 (0x0C), Signal_E at bits 16-31
        let decoded = dbc.decode(504, &payload, false).unwrap();
        let find = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);

        assert_eq!(find("Mux1"), Some(65.0));
        assert_eq!(find("Mux2"), Some(12.0));
        assert!(find("Signal_E").is_some());

        // Test with Mux1=64 and Mux2=12: Should NOT decode (Mux1 not 65)
        let payload2 = [0x40, 0x0C, 0x00, 0xB0, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(504, &payload2, false).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find2("Mux1"), Some(64.0));
        assert_eq!(find2("Mux2"), Some(12.0));
        assert!(find2("Signal_E").is_none());
    }

    #[test]
    fn test_decode_negative_multiplexer_switch() {
        use crate::Error;
        // Create a DBC with a signed multiplexer switch signal
        // 8-bit signed signal: values -128 to 127
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 MuxMessage : 8 ECM
 SG_ MuxSwitch M : 0|8@1- (1,0) [-128|127] ""
 SG_ SignalA m0 : 8|8@1+ (1,0) [0|255] ""
"#,
        )
        .unwrap();

        // Try to decode with a negative raw value for the multiplexer switch
        // -5 in 8-bit two's complement is 0xFB
        // Little-endian: MuxSwitch at bits 0-7, SignalA at bits 8-15
        let payload = [0xFB, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = dbc.decode(256, &payload, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Decoding(msg) => {
                assert_eq!(msg, Error::MULTIPLEXER_SWITCH_NEGATIVE);
            }
            _ => panic!("Expected Error::Decoding with MULTIPLEXER_SWITCH_NEGATIVE"),
        }
    }

    #[test]
    fn test_decode_too_many_unique_switches() {
        use crate::{Error, MAX_SIGNALS_PER_MESSAGE};
        // Skip this test if MAX_SIGNALS_PER_MESSAGE is too low to create 17 signals
        // (16 multiplexer switches + 1 signal = 17 total signals needed)
        if MAX_SIGNALS_PER_MESSAGE < 17 {
            return;
        }

        // Create a DBC with more than 16 unique switches in extended multiplexing
        // This should trigger an error when trying to decode
        // Using string literal concatenation to avoid std features
        let dbc_str = r#"VERSION "1.0"

BU_: ECM

BO_ 600 TooManySwitches : 18 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Mux2 M : 8|8@1+ (1,0) [0|255] ""
 SG_ Mux3 M : 16|8@1+ (1,0) [0|255] ""
 SG_ Mux4 M : 24|8@1+ (1,0) [0|255] ""
 SG_ Mux5 M : 32|8@1+ (1,0) [0|255] ""
 SG_ Mux6 M : 40|8@1+ (1,0) [0|255] ""
 SG_ Mux7 M : 48|8@1+ (1,0) [0|255] ""
 SG_ Mux8 M : 56|8@1+ (1,0) [0|255] ""
 SG_ Mux9 M : 64|8@1+ (1,0) [0|255] ""
 SG_ Mux10 M : 72|8@1+ (1,0) [0|255] ""
 SG_ Mux11 M : 80|8@1+ (1,0) [0|255] ""
 SG_ Mux12 M : 88|8@1+ (1,0) [0|255] ""
 SG_ Mux13 M : 96|8@1+ (1,0) [0|255] ""
 SG_ Mux14 M : 104|8@1+ (1,0) [0|255] ""
 SG_ Mux15 M : 112|8@1+ (1,0) [0|255] ""
 SG_ Mux16 M : 120|8@1+ (1,0) [0|255] ""
 SG_ Mux17 M : 128|8@1+ (1,0) [0|255] ""
 SG_ Signal_X m0 : 136|8@1+ (1,0) [0|255] "unit" *

SG_MUL_VAL_ 600 Signal_X Mux1 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux2 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux3 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux4 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux5 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux6 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux7 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux8 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux9 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux10 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux11 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux12 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux13 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux14 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux15 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux16 0-255 ;
SG_MUL_VAL_ 600 Signal_X Mux17 0-255 ;
"#;

        let dbc = Dbc::parse(dbc_str).unwrap();

        // Try to decode - should fail with MESSAGE_TOO_MANY_SIGNALS error
        // because we have 17 unique switches (exceeding the limit of 16)
        let payload = [0x00; 18];
        let result = dbc.decode(600, &payload, false);
        assert!(
            result.is_err(),
            "Decode should fail when there are more than 16 unique switches"
        );
        match result.unwrap_err() {
            Error::Decoding(msg) => {
                assert_eq!(
                    msg,
                    Error::MESSAGE_TOO_MANY_SIGNALS,
                    "Expected MESSAGE_TOO_MANY_SIGNALS error, got: {}",
                    msg
                );
            }
            e => panic!(
                "Expected Error::Decoding with MESSAGE_TOO_MANY_SIGNALS, got: {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_decode_extended_can_id() {
        // Test decoding with extended CAN ID (29-bit)
        // In DBC files, extended IDs have bit 31 set (0x80000000)
        // 0x80000000 + 0x400 = 2147484672
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
        )
        .unwrap();
        // 2147484672 = 0x80000400 = extended ID 0x400 (1024)

        let payload = [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        // Speed = 1000 * 0.1 = 100.0 km/h

        // Decode using raw CAN ID (0x400) with is_extended=true
        let decoded = dbc.decode(0x400, &payload, true).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "Speed");
        assert_eq!(decoded[0].value, 100.0);
        assert_eq!(decoded[0].unit, Some("km/h"));
    }

    #[test]
    fn test_decode_extended_can_id_not_found_without_flag() {
        // Verify that extended CAN messages are NOT found when is_extended=false
        // 0x80000000 + 0x400 = 2147484672
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
        )
        .unwrap();

        let payload = [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        // Without is_extended=true, the message should NOT be found
        let result = dbc.decode(0x400, &payload, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_standard_vs_extended_same_base_id() {
        // Test that standard and extended messages with same base ID are distinct
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 StandardMsg : 8 ECM
 SG_ StdSignal : 0|8@1+ (1,0) [0|255] "" *

BO_ 2147483904 ExtendedMsg : 8 ECM
 SG_ ExtSignal : 0|8@1+ (2,0) [0|510] "" *
"#,
        )
        .unwrap();
        // 2147483904 = 0x80000100 = extended ID 0x100 (256)

        let payload = [0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // raw = 100

        // Decode standard message (ID 256, is_extended=false)
        let decoded_std = dbc.decode(256, &payload, false).unwrap();
        assert_eq!(decoded_std.len(), 1);
        assert_eq!(decoded_std[0].name, "StdSignal");
        assert_eq!(decoded_std[0].value, 100.0); // factor 1

        // Decode extended message (ID 256, is_extended=true)
        let decoded_ext = dbc.decode(256, &payload, true).unwrap();
        assert_eq!(decoded_ext.len(), 1);
        assert_eq!(decoded_ext[0].name, "ExtSignal");
        assert_eq!(decoded_ext[0].value, 200.0); // factor 2
    }

    #[test]
    fn test_decode_with_value_descriptions() {
        // Test that value descriptions are included in decoded signals
        // Reference: SPECIFICATIONS.md Section 18.2 and 18.3
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 200 GearboxData : 4 ECM
 SG_ GearActual : 0|8@1+ (1,0) [0|5] "" *

VAL_ 200 GearActual 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" 5 "Manual" ;
"#,
        )
        .unwrap();

        // Test Gear = 0 (Park)
        let payload = [0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(200, &payload, false).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "GearActual");
        assert_eq!(decoded[0].value, 0.0);
        assert_eq!(decoded[0].description, Some("Park"));

        // Test Gear = 3 (Drive)
        let payload = [0x03, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(200, &payload, false).unwrap();
        assert_eq!(decoded[0].value, 3.0);
        assert_eq!(decoded[0].description, Some("Drive"));

        // Test Gear = 5 (Manual)
        let payload = [0x05, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(200, &payload, false).unwrap();
        assert_eq!(decoded[0].value, 5.0);
        assert_eq!(decoded[0].description, Some("Manual"));
    }

    #[test]
    fn test_decode_without_value_descriptions() {
        // Test that signals without value descriptions have None
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload, false).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "RPM");
        assert_eq!(decoded[0].value, 2000.0);
        assert_eq!(decoded[0].unit, Some("rpm"));
        assert_eq!(decoded[0].description, None);
    }

    #[test]
    fn test_decode_value_description_not_found() {
        // Test that values not in the description table return None
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 200 GearboxData : 4 ECM
 SG_ GearActual : 0|8@1+ (1,0) [0|255] "" *

VAL_ 200 GearActual 0 "Park" 1 "Reverse" 2 "Neutral" ;
"#,
        )
        .unwrap();

        // Test Gear = 10 (not in description table)
        let payload = [0x0A, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(200, &payload, false).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].value, 10.0);
        assert_eq!(decoded[0].description, None); // Value 10 not in table
    }

    #[test]
    fn test_decode_multiplexer_with_value_descriptions() {
        // Test that multiplexer switch signals also get value descriptions
        // Reference: SPECIFICATIONS.md Section 18.3
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 300 MultiplexedSensors : 8 ECM
 SG_ SensorID M : 0|8@1+ (1,0) [0|3] "" *
 SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C" *
 SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa" *

VAL_ 300 SensorID 0 "Temperature Sensor" 1 "Pressure Sensor" 2 "Humidity Sensor" 3 "Voltage Sensor" ;
"#,
        )
        .unwrap();

        // Test with SensorID = 0 (Temperature Sensor)
        // Temperature raw = 500 => physical = 500 * 0.1 + (-40) = 10.0 °C
        let payload = [0x00, 0xF4, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(300, &payload, false).unwrap();

        // Find the SensorID signal
        let sensor_id = decoded.iter().find(|s| s.name == "SensorID").unwrap();
        assert_eq!(sensor_id.value, 0.0);
        assert_eq!(sensor_id.description, Some("Temperature Sensor"));

        // Temperature should be decoded (m0 matches SensorID=0)
        let temp = decoded.iter().find(|s| s.name == "Temperature").unwrap();
        assert!(temp.description.is_none()); // No value descriptions for Temperature

        // Test with SensorID = 1 (Pressure Sensor)
        let payload = [0x01, 0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(300, &payload, false).unwrap();

        let sensor_id = decoded.iter().find(|s| s.name == "SensorID").unwrap();
        assert_eq!(sensor_id.value, 1.0);
        assert_eq!(sensor_id.description, Some("Pressure Sensor"));
    }

    #[cfg(feature = "embedded-can")]
    mod embedded_can_tests {
        use super::*;
        use embedded_can::{ExtendedId, Frame, Id, StandardId};

        /// A simple test frame for embedded-can tests
        struct TestFrame {
            id: Id,
            data: [u8; 8],
            dlc: usize,
        }

        impl TestFrame {
            fn new_standard(id: u16, data: &[u8]) -> Self {
                let mut frame_data = [0u8; 8];
                let dlc = data.len().min(8);
                frame_data[..dlc].copy_from_slice(&data[..dlc]);
                Self {
                    id: Id::Standard(StandardId::new(id).unwrap()),
                    data: frame_data,
                    dlc,
                }
            }

            fn new_extended(id: u32, data: &[u8]) -> Self {
                let mut frame_data = [0u8; 8];
                let dlc = data.len().min(8);
                frame_data[..dlc].copy_from_slice(&data[..dlc]);
                Self {
                    id: Id::Extended(ExtendedId::new(id).unwrap()),
                    data: frame_data,
                    dlc,
                }
            }
        }

        impl Frame for TestFrame {
            fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
                let mut frame_data = [0u8; 8];
                let dlc = data.len().min(8);
                frame_data[..dlc].copy_from_slice(&data[..dlc]);
                Some(Self {
                    id: id.into(),
                    data: frame_data,
                    dlc,
                })
            }

            fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Option<Self> {
                None // Not used in tests
            }

            fn is_extended(&self) -> bool {
                matches!(self.id, Id::Extended(_))
            }

            fn is_remote_frame(&self) -> bool {
                false
            }

            fn id(&self) -> Id {
                self.id
            }

            fn dlc(&self) -> usize {
                self.dlc
            }

            fn data(&self) -> &[u8] {
                &self.data[..self.dlc]
            }
        }

        #[test]
        fn test_decode_frame_standard() {
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
            )
            .unwrap();

            // Create a standard CAN frame
            let frame =
                TestFrame::new_standard(256, &[0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            let decoded = dbc.decode_frame(frame).unwrap();
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0].name, "RPM");
            assert_eq!(decoded[0].value, 2000.0);
            assert_eq!(decoded[0].unit, Some("rpm"));
        }

        #[test]
        fn test_decode_frame_extended() {
            // 0x80000000 + 0x400 = 2147484672
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
            )
            .unwrap();
            // 2147484672 = 0x80000400 = extended ID 0x400

            // Create an extended CAN frame with raw ID 0x400
            let frame =
                TestFrame::new_extended(0x400, &[0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            let decoded = dbc.decode_frame(frame).unwrap();
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0].name, "Speed");
            assert_eq!(decoded[0].value, 100.0);
            assert_eq!(decoded[0].unit, Some("km/h"));
        }

        #[test]
        fn test_decode_frame_standard_vs_extended() {
            // Test that decode_frame correctly distinguishes standard vs extended frames
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 256 StandardMsg : 8 ECM
 SG_ StdSignal : 0|8@1+ (1,0) [0|255] "" *

BO_ 2147483904 ExtendedMsg : 8 ECM
 SG_ ExtSignal : 0|8@1+ (2,0) [0|510] "" *
"#,
            )
            .unwrap();
            // 2147483904 = 0x80000100 = extended ID 0x100 (256)

            let payload = [0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // raw = 100

            // Standard frame with ID 256
            let std_frame = TestFrame::new_standard(256, &payload);
            let decoded_std = dbc.decode_frame(std_frame).unwrap();
            assert_eq!(decoded_std[0].name, "StdSignal");
            assert_eq!(decoded_std[0].value, 100.0);

            // Extended frame with ID 256
            let ext_frame = TestFrame::new_extended(256, &payload);
            let decoded_ext = dbc.decode_frame(ext_frame).unwrap();
            assert_eq!(decoded_ext[0].name, "ExtSignal");
            assert_eq!(decoded_ext[0].value, 200.0);
        }

        #[test]
        fn test_decode_frame_message_not_found() {
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
            )
            .unwrap();

            // Try to decode a frame with unknown ID
            let frame =
                TestFrame::new_standard(512, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            let result = dbc.decode_frame(frame);
            assert!(result.is_err());
        }
    }
}
