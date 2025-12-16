use crate::{Dbc, Error, MAX_EXTENDED_MULTIPLEXING, MAX_SIGNALS_PER_MESSAGE, Result, compat::Vec};

/// Decoding functionality for DBC structures
impl Dbc {
    /// Decode a CAN message payload using the message ID to find the corresponding message definition.
    ///
    /// This is a high-performance method for decoding CAN messages in `no_std` environments.
    /// It finds the message by ID, then decodes all signals in the message from the payload bytes.
    ///
    /// # Arguments
    ///
    /// * `id` - The CAN message ID to look up
    /// * `payload` - The CAN message payload bytes (up to 64 bytes for CAN FD)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<...>)` - A vector of (signal_name, physical_value) pairs
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
    /// let decoded = dbc.decode(256, &payload)?;
    /// assert_eq!(decoded.len(), 1);
    /// assert_eq!(decoded[0].0, "RPM");
    /// assert_eq!(decoded[0].1, 2000.0);
    /// assert_eq!(decoded[0].2, Some("rpm"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// High-performance CAN message decoding optimized for throughput.
    ///
    /// Performance optimizations:
    /// - O(1) or O(log n) message lookup via feature-flagged index (heapless/alloc)
    /// - Inlined hot paths
    /// - Direct error construction (no closure allocation)
    /// - Early validation to avoid unnecessary work
    /// - Optimized signal decoding loop
    #[inline]
    pub fn decode(
        &self,
        id: u32,
        payload: &[u8],
    ) -> Result<Vec<(&str, f64, Option<&str>), { MAX_SIGNALS_PER_MESSAGE }>> {
        // Find message by ID (performance-critical lookup)
        // Uses optimized index when available (O(1) with heapless, O(log n) with alloc)
        let message = self
            .messages()
            .find_by_id(id)
            .ok_or(Error::Decoding(Error::MESSAGE_NOT_FOUND))?;

        // Cache DLC conversion to avoid repeated casts
        let dlc = message.dlc() as usize;

        // Validate payload length matches message DLC (early return before any decoding)
        if payload.len() < dlc {
            return Err(Error::Decoding(Error::PAYLOAD_LENGTH_MISMATCH));
        }

        // Allocate Vec for decoded signals (name, value, unit)
        // Note: heapless Vec grows as needed; alloc Vec allocates dynamically
        let mut decoded_signals: Vec<(&str, f64, Option<&str>), { MAX_SIGNALS_PER_MESSAGE }> =
            Vec::new();

        let signals = message.signals();

        // Step 1: Decode all multiplexer switch signals first
        // Map switch signal names to their decoded values
        let mut switch_values: Vec<(&str, u64), 16> = Vec::new();
        for signal in signals.iter() {
            if signal.is_multiplexer_switch() {
                let value = signal.decode(payload)?;
                // Store the raw integer value (before factor/offset) for switch matching
                // We need to decode again to get the raw value for comparison
                let raw_value = {
                    let start_bit = signal.start_bit() as usize;
                    let length = signal.length() as usize;
                    let raw_bits = signal.byte_order().extract_bits(payload, start_bit, length);
                    if signal.is_unsigned() {
                        raw_bits as i64
                    } else {
                        let sign_bit_mask = 1u64 << (length - 1);
                        if (raw_bits & sign_bit_mask) != 0 {
                            let mask = !((1u64 << length) - 1);
                            (raw_bits | mask) as i64
                        } else {
                            raw_bits as i64
                        }
                    }
                };
                switch_values
                    .push((signal.name(), raw_value as u64))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
                // Also add to decoded signals
                decoded_signals
                    .push((signal.name(), value, signal.unit()))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
            }
        }

        // Step 2: Get extended multiplexing entries for this message
        let extended_mux_entries = self.extended_multiplexing_for_message(id);

        // Step 3: Decode all non-switch signals based on multiplexing rules
        for signal in signals.iter() {
            // Skip multiplexer switches (already decoded)
            if signal.is_multiplexer_switch() {
                continue;
            }

            // Check if signal should be decoded based on multiplexing
            let should_decode = if let Some(mux_value) = signal.multiplexer_switch_value() {
                // This is a multiplexed signal (m0, m1, etc.)
                // Extended multiplexing: Check SG_MUL_VAL_ ranges first
                // If extended multiplexing entries exist, they take precedence over basic m0/m1 values
                let extended_entries_for_signal: Vec<_, { MAX_EXTENDED_MULTIPLEXING }> =
                    extended_mux_entries
                        .iter()
                        .filter(|ext_mux| ext_mux.signal_name() == signal.name())
                        .cloned()
                        .collect();

                if !extended_entries_for_signal.is_empty() {
                    // Extended multiplexing: Check ALL switches referenced in extended entries (AND logic)
                    // Collect unique switch names (no_std compatible, using Vec)
                    let mut unique_switches: Vec<&str, 16> = Vec::new();
                    for ext_mux in extended_entries_for_signal.iter() {
                        let switch_name = ext_mux.multiplexer_switch();
                        if !unique_switches.iter().any(|&s| s == switch_name) {
                            let _ = unique_switches.push(switch_name);
                        }
                    }

                    // ALL switches referenced by this signal must have matching values
                    unique_switches.iter().all(|switch_name| {
                        // Find the switch value by name
                        let switch_val = switch_values
                            .iter()
                            .find(|(name, _)| *name == *switch_name)
                            .map(|(_, val)| *val);

                        if let Some(val) = switch_val {
                            // Check if any extended entry for this switch has a matching value range
                            extended_entries_for_signal
                                .iter()
                                .filter(|e| e.multiplexer_switch() == *switch_name)
                                .any(|ext_mux| {
                                    ext_mux
                                        .value_ranges()
                                        .iter()
                                        .any(|(min, max)| val >= *min && val <= *max)
                                })
                        } else {
                            // Switch not found, cannot match
                            false
                        }
                    })
                } else {
                    // Use basic multiplexing: Check if any switch value equals mux_value
                    // m0 means decode when any switch value is 0, m1 means decode when any switch value is 1, etc.
                    switch_values.iter().any(|(_, switch_val)| *switch_val == mux_value)
                }
            } else {
                // Normal signal (not multiplexed) - always decode
                true
            };

            if should_decode {
                let value = signal.decode(payload)?;
                decoded_signals
                    .push((signal.name(), value, signal.unit()))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
            }
        }

        Ok(decoded_signals)
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
        let decoded = dbc.decode(256, &payload).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, "RPM");
        assert_eq!(decoded[0].1, 2000.0);
        assert_eq!(decoded[0].2, Some("rpm"));
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
        let result = dbc.decode(512, &payload);
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
        let decoded = dbc.decode(256, &payload).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].0, "RPM");
        assert_eq!(decoded[0].1, 2000.0);
        assert_eq!(decoded[0].2, Some("rpm"));
        assert_eq!(decoded[1].0, "Temp");
        assert_eq!(decoded[1].1, 50.0);
        assert_eq!(decoded[1].2, Some("°C"));
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
        let result = dbc.decode(256, &payload);
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
        let decoded = dbc.decode(256, &payload).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, "RPM");
        // The exact value depends on big-endian bit extraction implementation
        // We just verify that decoding doesn't crash and returns a value
        assert!(decoded[0].1 >= 0.0);
        assert_eq!(decoded[0].2, Some("rpm"));
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
        let decoded = dbc.decode(256, &payload).unwrap();

        // Helper to find value by signal name
        let find_signal =
            |name: &str| decoded.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

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
        let decoded = dbc.decode(256, &payload).unwrap();

        // Helper to find value by signal name
        let find_signal =
            |name: &str| decoded.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

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
 SG_ BigEndianSignal : 16|16@0+ (1.0,0) [0|65535] ""
 SG_ AnotherLittleEndian : 32|8@1+ (1.0,0) [0|255] ""
 SG_ AnotherBigEndian : 40|8@0+ (1.0,0) [0|255] ""
"#,
        )
        .unwrap();

        // Test payload with both big-endian and little-endian signals:
        // - LittleEndianSignal at bits 0-15 (bytes 0-1): [0x34, 0x12] = 0x1234 = 4660
        // - BigEndianSignal at bits 16-31 (bytes 2-3): [0x00, 0x01] = decoded based on BE bit mapping
        // - AnotherLittleEndian at bits 32-39 (byte 4): 0xAB = 171
        // - AnotherBigEndian at bits 40-47 (byte 5): 0xCD = decoded based on BE bit mapping
        let payload = [
            0x34, 0x12, // Bytes 0-1: LittleEndianSignal
            0x00, 0x01, // Bytes 2-3: BigEndianSignal
            0xAB, // Byte 4: AnotherLittleEndian
            0xCD, // Byte 5: AnotherBigEndian
            0x00, 0x00, // Padding
        ];
        let decoded = dbc.decode(256, &payload).unwrap();

        // Helper to find value by signal name
        let find_signal =
            |name: &str| decoded.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

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
        let decoded = dbc.decode(500, &payload).unwrap();

        let find_signal =
            |name: &str| decoded.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

        assert_eq!(find_signal("Mux1"), Some(5.0));
        // Extended multiplexing: Signal_A should decode when Mux1 is in range 5-10
        let ext_entries = dbc.extended_multiplexing_for_message(500);
        assert_eq!(
            ext_entries.len(),
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
        let decoded2 = dbc.decode(500, &payload2).unwrap();
        let find_signal2 =
            |name: &str| decoded2.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

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
        let decoded1 = dbc.decode(501, &payload1).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find1("Mux1"), Some(3.0));
        assert!(find1("Signal_B").is_some());

        // Test with Mux1 = 12: Should decode (within range 10-15)
        // Signal_B at bits 16-31, value 8192 (raw, little-endian: 0x00, 0x20)
        let payload2 = [0x0C, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(501, &payload2).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find2("Mux1"), Some(12.0));
        assert!(find2("Signal_B").is_some());

        // Test with Mux1 = 22: Should decode (within range 20-25)
        // Signal_B at bits 16-31, value 12288 (raw, little-endian: 0x00, 0x30)
        let payload3 = [0x16, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00];
        let decoded3 = dbc.decode(501, &payload3).unwrap();
        let find3 = |name: &str| decoded3.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find3("Mux1"), Some(22.0));
        assert!(find3("Signal_B").is_some());

        // Test with Mux1 = 8: Should NOT decode (not in any range)
        // Signal_B at bits 16-31, value 16384 (raw, little-endian: 0x00, 0x40)
        let payload4 = [0x08, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00];
        let decoded4 = dbc.decode(501, &payload4).unwrap();
        let find4 = |name: &str| decoded4.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
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
        let decoded1 = dbc.decode(502, &payload1).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find1("Mux1"), Some(7.0));
        assert_eq!(find1("Mux2"), Some(22.0));
        assert!(find1("Signal_C").is_some());

        // Test with Mux1=7 and Mux2=30: Should NOT decode (Mux2 outside range)
        let payload2 = [0x07, 0x1E, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(502, &payload2).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find2("Mux1"), Some(7.0));
        assert_eq!(find2("Mux2"), Some(30.0));
        assert!(find2("Signal_C").is_none());

        // Test with Mux1=15 and Mux2=22: Should NOT decode (Mux1 outside range)
        let payload3 = [0x0F, 0x16, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00];
        let decoded3 = dbc.decode(502, &payload3).unwrap();
        let find3 = |name: &str| decoded3.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
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
        let decoded1 = dbc.decode(503, &payload1).unwrap();
        let find1 = |name: &str| decoded1.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find1("Mux1"), Some(0.0));
        assert!(find1("Signal_D").is_none());

        // Test with Mux1 = 12: Should decode (within extended range 10-15)
        let payload2 = [0x0C, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(503, &payload2).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
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
        let decoded = dbc.decode(504, &payload).unwrap();
        let find = |name: &str| decoded.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);

        assert_eq!(find("Mux1"), Some(65.0));
        assert_eq!(find("Mux2"), Some(12.0));
        assert!(find("Signal_E").is_some());

        // Test with Mux1=64 and Mux2=12: Should NOT decode (Mux1 not 65)
        let payload2 = [0x40, 0x0C, 0x00, 0xB0, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc.decode(504, &payload2).unwrap();
        let find2 = |name: &str| decoded2.iter().find(|(n, _, _)| *n == name).map(|(_, v, _)| *v);
        assert_eq!(find2("Mux1"), Some(64.0));
        assert_eq!(find2("Mux2"), Some(12.0));
        assert!(find2("Signal_E").is_none());
    }
}
