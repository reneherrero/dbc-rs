use super::Signal;
use crate::{Error, Result};

/// Round to nearest integer (half away from zero).
/// Equivalent to libm::round but without the dependency.
#[inline]
fn round(x: f64) -> f64 {
    // Cast to i64 truncates towards zero, so we add/subtract 0.5 first
    if x >= 0.0 {
        (x + 0.5) as i64 as f64
    } else {
        (x - 0.5) as i64 as f64
    }
}

impl Signal {
    /// Encode a physical value to raw bits for this signal.
    ///
    /// This is the inverse of `decode_raw()`. It converts a physical value
    /// (after factor and offset have been applied) back to the raw integer
    /// value that can be inserted into a CAN message payload.
    ///
    /// # Arguments
    ///
    /// * `physical_value` - The physical value to encode (e.g., 2000.0 for RPM)
    ///
    /// # Returns
    ///
    /// * `Ok(raw_bits)` - The raw bits ready to be inserted into the payload
    /// * `Err(Error)` - If the value is outside the signal's min/max range
    ///
    /// # Formula
    ///
    /// ```text
    /// raw_value = (physical_value - offset) / factor
    /// ```
    #[inline]
    pub fn encode_raw(&self, physical_value: f64) -> Result<u64> {
        // Validate value is within min/max range
        if physical_value < self.min || physical_value > self.max {
            return Err(Error::Encoding(Error::ENCODING_VALUE_OUT_OF_RANGE));
        }

        // Reverse the decode formula: raw = (physical - offset) / factor
        // Handle factor == 0 to avoid division by zero (shouldn't happen in valid DBC)
        let raw_float = if self.factor != 0.0 {
            (physical_value - self.offset) / self.factor
        } else {
            // If factor is 0, physical == offset always, so raw can be 0
            0.0
        };

        // Round to nearest integer
        let raw_signed = round(raw_float) as i64;

        // Handle signed vs unsigned encoding
        let raw_bits = if self.unsigned {
            // Unsigned: ensure non-negative and fits in bit length
            if raw_signed < 0 {
                return Err(Error::Encoding(Error::ENCODING_VALUE_OVERFLOW));
            }
            let raw_unsigned = raw_signed as u64;
            let max_value = if self.length >= 64 {
                u64::MAX
            } else {
                (1u64 << self.length) - 1
            };
            if raw_unsigned > max_value {
                return Err(Error::Encoding(Error::ENCODING_VALUE_OVERFLOW));
            }
            raw_unsigned
        } else {
            // Signed: use two's complement encoding
            // Check if value fits in signed range for this bit length
            let half_range = 1i64 << (self.length - 1);
            let min_signed = -half_range;
            let max_signed = half_range - 1;

            if raw_signed < min_signed || raw_signed > max_signed {
                return Err(Error::Encoding(Error::ENCODING_VALUE_OVERFLOW));
            }

            // Convert to two's complement representation
            // For positive values, just cast. For negative, mask to bit length.
            if raw_signed >= 0 {
                raw_signed as u64
            } else {
                // Two's complement: mask to signal bit length
                let mask = if self.length >= 64 {
                    u64::MAX
                } else {
                    (1u64 << self.length) - 1
                };
                (raw_signed as u64) & mask
            }
        };

        Ok(raw_bits)
    }

    /// Encode a physical value and insert it into a payload buffer.
    ///
    /// This is a convenience method that combines `encode_raw()` with
    /// `ByteOrder::insert_bits()` to directly write the encoded value
    /// into a CAN message payload.
    ///
    /// # Arguments
    ///
    /// * `physical_value` - The physical value to encode
    /// * `payload` - The mutable payload buffer to write into
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Value was successfully encoded and written
    /// * `Err(Error)` - If encoding failed or signal extends beyond payload
    #[inline]
    pub fn encode_to(&self, physical_value: f64, payload: &mut [u8]) -> Result<()> {
        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        if end_byte >= payload.len() {
            return Err(Error::Encoding(Error::SIGNAL_EXTENDS_BEYOND_DATA));
        }

        let raw_bits = self.encode_raw(physical_value)?;
        self.byte_order.insert_bits(payload, start_bit, length, raw_bits);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Signal;
    use crate::Parser;

    #[test]
    fn test_encode_raw_unsigned() {
        // Signal: 16-bit unsigned, factor=0.25, offset=0
        // Physical 2000.0 -> raw = 2000 / 0.25 = 8000
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();

        let raw = signal.encode_raw(2000.0).unwrap();
        assert_eq!(raw, 8000);
    }

    #[test]
    fn test_encode_raw_with_offset() {
        // Signal: 8-bit signed, factor=1, offset=-40
        // Physical 50.0 -> raw = (50 - (-40)) / 1 = 90
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Temp : 16|8@1- (1,-40) [-40|87] \"\"").unwrap())
                .unwrap();

        let raw = signal.encode_raw(50.0).unwrap();
        assert_eq!(raw, 90);
    }

    #[test]
    fn test_encode_raw_signed_negative() {
        // Signal: 16-bit signed, factor=0.01, offset=0
        // Physical -10.0 -> raw = -10 / 0.01 = -1000
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Torque : 0|16@1- (0.01,0) [-327.68|327.67] \"Nm\"").unwrap(),
        )
        .unwrap();

        let raw = signal.encode_raw(-10.0).unwrap();
        // -1000 in 16-bit two's complement = 0xFC18
        assert_eq!(raw, 0xFC18);
    }

    #[test]
    fn test_encode_raw_out_of_range() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();

        // Value above max
        let result = signal.encode_raw(9000.0);
        assert!(result.is_err());

        // Value below min
        let result = signal.encode_raw(-100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        // Test that encode(decode(x)) == x for the raw value
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] \"km/h\"").unwrap(),
        )
        .unwrap();

        // Encode physical value 100.0
        let raw = signal.encode_raw(100.0).unwrap();
        assert_eq!(raw, 1000); // 100 / 0.1 = 1000

        // Decode the raw value back
        let data = [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 1000 in LE
        let (decoded_raw, decoded_physical) = signal.decode_raw(&data).unwrap();
        assert_eq!(decoded_raw, 1000);
        assert!((decoded_physical - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_encode_to_little_endian() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] \"km/h\"").unwrap(),
        )
        .unwrap();

        let mut payload = [0x00; 8];
        signal.encode_to(100.0, &mut payload).unwrap();

        // 100.0 / 0.1 = 1000 = 0x03E8, little-endian: [0xE8, 0x03]
        assert_eq!(payload[0], 0xE8);
        assert_eq!(payload[1], 0x03);
    }

    #[test]
    fn test_encode_to_big_endian() {
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ Pressure : 7|16@0+ (0.01,0) [0|655.35] \"kPa\"").unwrap(),
        )
        .unwrap();

        let mut payload = [0x00; 8];
        signal.encode_to(10.0, &mut payload).unwrap();

        // 10.0 / 0.01 = 1000 = 0x03E8, big-endian at bit 7
        assert_eq!(payload[0], 0x03);
        assert_eq!(payload[1], 0xE8);
    }

    #[test]
    fn test_encode_to_at_offset() {
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Throttle : 24|8@1+ (1,0) [0|100] \"%\"").unwrap())
                .unwrap();

        let mut payload = [0x00; 8];
        signal.encode_to(75.0, &mut payload).unwrap();

        // 75 should be at byte 3 (bit 24)
        assert_eq!(payload[3], 75);
    }

    #[test]
    fn test_encode_to_preserves_other_bits() {
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Gear : 8|8@1+ (1,0) [0|5] \"\"").unwrap())
                .unwrap();

        let mut payload = [0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        signal.encode_to(3.0, &mut payload).unwrap();

        // Byte 0 and 2+ should be preserved
        assert_eq!(payload[0], 0xFF);
        assert_eq!(payload[1], 3);
        assert_eq!(payload[2], 0xFF);
    }
}
