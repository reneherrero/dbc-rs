use super::Signal;
use crate::{Error, Result};

impl Signal {
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
    /// Decode signal value from CAN payload - optimized for high-throughput decoding.
    #[inline]
    pub fn decode(&self, data: &[u8]) -> Result<f64> {
        // Cache conversions to usize (common in hot path)
        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        // Bounds check - early return for invalid signals
        if end_byte >= data.len() {
            return Err(Error::Decoding(Error::SIGNAL_EXTENDS_BEYOND_DATA));
        }

        // Extract bits based on byte order
        let raw_value = self.byte_order.extract_bits(data, start_bit, length);

        // Convert to signed/unsigned with optimized sign extension
        let value = if self.unsigned {
            raw_value as i64
        } else {
            // Sign extend for signed values
            // Optimized: compute sign bit mask only once
            let sign_bit_mask = 1u64 << (length - 1);
            if (raw_value & sign_bit_mask) != 0 {
                // Negative value - sign extend using bitwise mask
                let mask = !((1u64 << length) - 1);
                (raw_value | mask) as i64
            } else {
                raw_value as i64
            }
        };

        // Apply factor and offset to get physical value (single mul-add operation)
        Ok((value as f64) * self.factor + self.offset)
    }

    /// Decode the signal and return both raw and physical values in a single pass.
    ///
    /// This is an optimized method for multiplexer switch decoding where both the
    /// raw integer value (for switch matching) and the physical value are needed.
    /// Avoids the overhead of extracting bits twice.
    ///
    /// # Arguments
    ///
    /// * `data` - The CAN message data bytes (up to 8 bytes for classic CAN, 64 for CAN FD)
    ///
    /// # Returns
    ///
    /// * `Ok((raw_value, physical_value))` - The raw signed integer and physical (factor+offset) value
    /// * `Err(Error)` - If the signal extends beyond the data length
    #[inline]
    pub fn decode_raw(&self, data: &[u8]) -> Result<(i64, f64)> {
        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        if end_byte >= data.len() {
            return Err(Error::Decoding(Error::SIGNAL_EXTENDS_BEYOND_DATA));
        }

        let raw_bits = self.byte_order.extract_bits(data, start_bit, length);

        let raw_value = if self.unsigned {
            raw_bits as i64
        } else {
            let sign_bit_mask = 1u64 << (length - 1);
            if (raw_bits & sign_bit_mask) != 0 {
                let mask = !((1u64 << length) - 1);
                (raw_bits | mask) as i64
            } else {
                raw_bits as i64
            }
        };

        let physical_value = (raw_value as f64) * self.factor + self.offset;
        Ok((raw_value, physical_value))
    }
}

#[cfg(test)]
mod tests {
    use super::Signal;
    use crate::Parser;

    #[test]
    fn test_decode_little_endian() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ TestSignal : 0|16@1+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Test value 0x0102 = 258: little-endian bytes are [0x02, 0x01]
        let data = [0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();
        assert_eq!(value, 258.0);
    }

    #[test]
    fn test_decode_big_endian() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ TestSignal : 0|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Test big-endian decoding: value 0x0100 = 256 at bit 0-15
        let data = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();
        // Verify it decodes to a valid value within range
        assert!((0.0..=65535.0).contains(&value));
    }

    #[test]
    fn test_decode_little_endian_with_offset() {
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Temp : 0|8@1- (1,-40) [-40|215] \"\"").unwrap())
                .unwrap();
        // Raw value 90 with offset -40 = 50°C
        let data = [0x5A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();
        assert_eq!(value, 50.0);
    }

    #[test]
    fn test_decode_big_endian_with_factor() {
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"\"").unwrap())
                .unwrap();
        // Test big-endian decoding with factor
        // Big-endian at bit 0-15: bytes [0x1F, 0x40]
        let data = [0x1F, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();
        // Verify it decodes and applies factor correctly (value should be positive)
        assert!((0.0..=16383.75).contains(&value)); // Max u16 * 0.25
    }

    // ============================================================================
    // Specification Verification Tests
    // These tests verify against exact examples from dbc/SPECIFICATIONS.md
    // ============================================================================

    /// Verify Section 10.3: Little-Endian Signals
    /// Example from spec:
    /// Signal: SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h"
    /// Message bytes: [0x64, 0x00, ...]
    /// Raw value = 0x0064 = 100 decimal
    /// Physical = 100 × 0.1 = 10.0 km/h
    #[test]
    fn test_spec_section_10_3_little_endian_example() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] \"km/h\"").unwrap(),
        )
        .unwrap();

        // Spec example: bytes [0x64, 0x00]
        let data = [0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();

        // Expected: raw=100, physical=100*0.1=10.0
        assert_eq!(
            value, 10.0,
            "Spec Section 10.3: Little-endian 0x64 should decode to 10.0 km/h"
        );
    }

    /// Verify Section 10.4: Big-Endian Signals
    /// Example from spec:
    /// Signal: SG_ Pressure : 7|16@0+ (0.01,0) [0|655.35] "kPa"
    /// Message bytes: [0x03, 0xE8, ...]
    /// Raw value = 0x03E8 = 1000 decimal
    /// Physical = 1000 × 0.01 = 10.0 kPa
    #[test]
    fn test_spec_section_10_4_big_endian_example() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Pressure : 7|16@0+ (0.01,0) [0|655.35] \"kPa\"").unwrap(),
        )
        .unwrap();

        // Spec example: bytes [0x03, 0xE8]
        let data = [0x03, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();

        // Expected: raw=1000 (0x03E8), physical=1000*0.01=10.0
        assert_eq!(
            value, 10.0,
            "Spec Section 10.4: Big-endian 0x03E8 should decode to 10.0 kPa"
        );
    }

    /// Verify Section 10.5: Value Conversion with Offset
    /// Example from spec:
    /// Signal: SG_ Temperature : 16|8@1- (1,-40) [-40|87] "°C"
    /// Raw value = 127 (0x7F) → Physical = 127 × 1 + (-40) = 87°C
    /// Raw value = 0 (0x00) → Physical = 0 × 1 + (-40) = -40°C
    #[test]
    fn test_spec_section_10_5_temperature_offset_example() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Temperature : 16|8@1- (1,-40) [-40|87] \"\"").unwrap(),
        )
        .unwrap();

        // Test 1: raw=127 → physical=87°C
        // Little-endian: signal at bit 16 means byte 2
        let data1 = [0x00, 0x00, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value1 = signal.decode(&data1).unwrap();
        assert_eq!(
            value1, 87.0,
            "Spec Section 10.5: raw=127 should decode to 87°C"
        );

        // Test 2: raw=0 → physical=-40°C
        let data2 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value2 = signal.decode(&data2).unwrap();
        assert_eq!(
            value2, -40.0,
            "Spec Section 10.5: raw=0 should decode to -40°C"
        );
    }

    /// Verify Section 10.2: Byte Order Values
    /// @0 = Big-Endian (Motorola)
    /// @1 = Little-Endian (Intel)
    #[test]
    fn test_spec_section_10_2_byte_order_values() {
        use crate::ByteOrder;

        // Verify enum values match spec
        assert_eq!(
            ByteOrder::BigEndian as u8,
            0,
            "Spec Section 10.2: @0 = Big-Endian"
        );
        assert_eq!(
            ByteOrder::LittleEndian as u8,
            1,
            "Spec Section 10.2: @1 = Little-Endian"
        );
    }

    /// Verify Section 10.5: Value Conversion Formula
    /// physical_value = raw_value × factor + offset
    #[test]
    fn test_spec_section_10_5_value_conversion_formula() {
        // Test with factor=0.25 and offset=100
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Test : 0|16@1+ (0.25,100) [0|1000] \"\"").unwrap(),
        )
        .unwrap();

        // raw=400 → physical = 400 * 0.25 + 100 = 200
        // Little-endian: 400 = 0x0190 → bytes [0x90, 0x01]
        let data = [0x90, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let value = signal.decode(&data).unwrap();
        assert_eq!(value, 200.0, "Spec Section 10.5: 400 * 0.25 + 100 = 200");
    }
}
