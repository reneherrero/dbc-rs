use super::Dbc;
use crate::compat::Vec;
use crate::{Error, MAX_SIGNALS_PER_MESSAGE, Result};

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
        // Also build a mapping from switch index (0, 1, 2, ...) to switch name
        #[cfg(feature = "std")]
        let mut switch_values: std::collections::BTreeMap<&str, u64> =
            std::collections::BTreeMap::new();
        #[cfg(feature = "std")]
        let mut switch_index_to_name: std::vec::Vec<&str> = std::vec::Vec::new();
        #[cfg(not(feature = "std"))]
        let mut switch_values: crate::compat::Vec<(&str, u64), 16> = crate::compat::Vec::new();
        #[cfg(not(feature = "std"))]
        let mut switch_index_to_name: crate::compat::Vec<&str, 16> = crate::compat::Vec::new();
        for signal in signals.iter() {
            if signal.is_multiplexer_switch() {
                let value = signal.decode(payload)?;
                // Store the raw integer value (before factor/offset) for switch matching
                // We need to decode again to get the raw value for comparison
                // Actually, we can compute raw from physical: (physical - offset) / factor
                // But to avoid precision issues, let's decode the raw bits directly
                let raw_value = {
                    let start_bit = signal.start_bit() as usize;
                    let length = signal.length() as usize;
                    let raw_bits = match signal.byte_order() {
                        crate::ByteOrder::LittleEndian => {
                            crate::signal::Signal::extract_bits_little_endian_helper(
                                payload, start_bit, length,
                            )
                        }
                        crate::ByteOrder::BigEndian => {
                            crate::signal::Signal::extract_bits_big_endian_helper(
                                payload, start_bit, length,
                            )
                        }
                    };
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
                #[cfg(feature = "std")]
                {
                    switch_values.insert(signal.name(), raw_value as u64);
                    switch_index_to_name.push(signal.name());
                }
                #[cfg(not(feature = "std"))]
                {
                    switch_values.push((signal.name(), raw_value as u64)).ok();
                    switch_index_to_name.push(signal.name()).ok();
                }
                // Also add to decoded signals
                decoded_signals
                    .push((signal.name(), value, signal.unit()))
                    .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
            }
        }

        // Step 2: Get extended multiplexing entries for this message (if std feature enabled)
        #[cfg(feature = "std")]
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
                #[cfg(feature = "std")]
                {
                    // Extended multiplexing: Check SG_MUL_VAL_ ranges first
                    // If extended multiplexing entries exist, they take precedence over basic m0/m1 values
                    let extended_entries_for_signal: std::vec::Vec<_> = extended_mux_entries
                        .iter()
                        .filter(|ext_mux| ext_mux.signal_name() == signal.name())
                        .collect();

                    if !extended_entries_for_signal.is_empty() {
                        // Extended multiplexing: Check ALL switches referenced in extended entries (AND logic)
                        use std::collections::HashSet;
                        let unique_switches: HashSet<&str> = extended_entries_for_signal
                            .iter()
                            .map(|e| e.multiplexer_switch())
                            .collect();

                        // ALL switches referenced by this signal must have matching values
                        let all_switches_match = unique_switches.iter().all(|switch_name| {
                            extended_entries_for_signal
                                .iter()
                                .filter(|e| e.multiplexer_switch() == *switch_name)
                                .any(|ext_mux| {
                                    switch_values
                                        .get(ext_mux.multiplexer_switch())
                                        .map(|&switch_val| {
                                            ext_mux.value_ranges().iter().any(|(min, max)| {
                                                switch_val >= *min && switch_val <= *max
                                            })
                                        })
                                        .unwrap_or(false)
                                })
                        });

                        if !all_switches_match {
                            false
                        } else {
                            // When signals share bit positions and both match, check if there are
                            // other signals that also match. If so, only decode the one with the
                            // highest m0/m1 index. This requires checking all signals, so for now,
                            // we'll use a simpler heuristic: if the primary switch is in extended
                            // entries and its value equals the m0/m1 index, decode. Otherwise,
                            // check if there's a conflicting signal with higher m0/m1 index.
                            if (mux_value as usize) < switch_index_to_name.len() {
                                let primary_switch_name = switch_index_to_name[mux_value as usize];
                                if unique_switches.contains(primary_switch_name) {
                                    // Check other signals that share the same bit positions
                                    let signal_start_bit = signal.start_bit();
                                    let signal_length = signal.length();
                                    let conflicting_signals: std::vec::Vec<_> = signals
                                        .iter()
                                        .filter(|s| {
                                            !s.is_multiplexer_switch()
                                                && s.start_bit() == signal_start_bit
                                                && s.length() == signal_length
                                                && s.name() != signal.name()
                                        })
                                        .collect();

                                    // Check if any conflicting signal also matches and has higher m0/m1 index
                                    let has_higher_conflict = conflicting_signals.iter().any(|conflict_sig| {
                                        if let Some(conflict_mux_val) = conflict_sig.multiplexer_switch_value() {
                                            if conflict_mux_val > mux_value {
                                                // Check if this conflicting signal also matches
                                                let conflict_extended: std::vec::Vec<_> = extended_mux_entries
                                                    .iter()
                                                    .filter(|ext_mux| ext_mux.signal_name() == conflict_sig.name())
                                                    .collect();
                                                if !conflict_extended.is_empty() {
                                                    let conflict_switches: HashSet<&str> = conflict_extended
                                                        .iter()
                                                        .map(|e| e.multiplexer_switch())
                                                        .collect();
                                                    conflict_switches.iter().all(|switch_name| {
                                                        conflict_extended
                                                            .iter()
                                                            .filter(|e| e.multiplexer_switch() == *switch_name)
                                                            .any(|ext_mux| {
                                                                switch_values
                                                                    .get(ext_mux.multiplexer_switch())
                                                                    .map(|&switch_val| {
                                                                        ext_mux.value_ranges().iter().any(|(min, max)| {
                                                                            switch_val >= *min && switch_val <= *max
                                                                        })
                                                                    })
                                                                    .unwrap_or(false)
                                                            })
                                                    })
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    });

                                    // If there's a conflicting signal with higher m0/m1 index that also matches,
                                    // don't decode this signal
                                    !has_higher_conflict
                                } else {
                                    all_switches_match
                                }
                            } else {
                                all_switches_match
                            }
                        }
                    } else {
                        // Use basic multiplexing: Check if the switch at index mux_value matches
                        // m0 refers to switch index 0, m1 to index 1, etc.
                        if (mux_value as usize) < switch_index_to_name.len() {
                            let switch_name = switch_index_to_name[mux_value as usize];
                            switch_values
                                .get(switch_name)
                                .map(|&switch_val| switch_val == mux_value)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std, only support basic multiplexing
                    // Check if the switch at index mux_value has value mux_value
                    if (mux_value as usize) < switch_index_to_name.len() {
                        let switch_name = switch_index_to_name[mux_value as usize];
                        switch_values
                            .iter()
                            .find(|(name, _)| *name == switch_name)
                            .map(|(_, switch_val)| *switch_val == mux_value)
                            .unwrap_or(false)
                    } else {
                        false
                    }
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
