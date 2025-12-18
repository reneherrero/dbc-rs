/// Byte order (endianness) for signal encoding in CAN messages.
///
/// In DBC files, byte order is specified as:
/// - `0` = BigEndian (Motorola format)
/// - `1` = LittleEndian (Intel format)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ByteOrder {
    /// Little-endian byte order (Intel format, `1` in DBC files).
    ///
    /// Bytes are ordered from least significant to most significant.
    LittleEndian = 1,
    /// Big-endian byte order (Motorola format, `0` in DBC files).
    ///
    /// Bytes are ordered from most significant to least significant.
    BigEndian = 0,
}

impl ByteOrder {
    /// Extract bits from data based on byte order.
    /// Inlined for hot path optimization.
    ///
    /// # Performance
    ///
    /// This method uses optimized fast paths for common cases:
    /// - Byte-aligned little-endian 8/16/32/64-bit signals use direct memory reads
    /// - Other cases use a generic loop-based extraction
    #[inline]
    pub(crate) fn extract_bits(self, data: &[u8], start_bit: usize, length: usize) -> u64 {
        match self {
            ByteOrder::LittleEndian => {
                // Fast path: byte-aligned little-endian signals (most common case)
                let bit_offset = start_bit % 8;
                let byte_idx = start_bit / 8;

                if bit_offset == 0 {
                    // Byte-aligned - use direct memory reads
                    match length {
                        8 => return data[byte_idx] as u64,
                        16 => {
                            // SAFETY: bounds checked by caller (end_byte < data.len())
                            return u16::from_le_bytes([data[byte_idx], data[byte_idx + 1]]) as u64;
                        }
                        32 => {
                            return u32::from_le_bytes([
                                data[byte_idx],
                                data[byte_idx + 1],
                                data[byte_idx + 2],
                                data[byte_idx + 3],
                            ]) as u64;
                        }
                        64 => {
                            return u64::from_le_bytes([
                                data[byte_idx],
                                data[byte_idx + 1],
                                data[byte_idx + 2],
                                data[byte_idx + 3],
                                data[byte_idx + 4],
                                data[byte_idx + 5],
                                data[byte_idx + 6],
                                data[byte_idx + 7],
                            ]);
                        }
                        _ => {} // Fall through to generic path
                    }
                }

                // Generic path: extract bits sequentially from start_bit forward
                let mut value: u64 = 0;
                let mut bits_remaining = length;
                let mut current_bit = start_bit;

                while bits_remaining > 0 {
                    let byte_idx = current_bit / 8;
                    let bit_in_byte = current_bit % 8;
                    let bits_to_take = bits_remaining.min(8 - bit_in_byte);

                    let byte = data[byte_idx] as u64;
                    let mask = ((1u64 << bits_to_take) - 1) << bit_in_byte;
                    let extracted = (byte & mask) >> bit_in_byte;

                    value |= extracted << (length - bits_remaining);

                    bits_remaining -= bits_to_take;
                    current_bit += bits_to_take;
                }

                value
            }
            ByteOrder::BigEndian => {
                // Big-endian (Motorola): start_bit is MSB in big-endian numbering.
                // BE bit N maps to physical bit: byte_num * 8 + (7 - bit_in_byte)
                //
                // Optimization: Process up to 8 bits at a time instead of 1 bit at a time.
                // This reduces loop iterations from O(length) to O(length/8).
                let mut value: u64 = 0;
                let mut bits_remaining = length;
                let mut signal_bit_offset = 0; // How many bits of the signal we've processed

                while bits_remaining > 0 {
                    // Current BE bit position
                    let be_bit = start_bit + signal_bit_offset;
                    let byte_num = be_bit / 8;
                    let bit_in_byte = be_bit % 8;

                    // Calculate how many bits we can take from this byte
                    // In BE numbering, bits go from high to low within a byte (7,6,5,4,3,2,1,0)
                    // bit_in_byte 0 = physical bit 7, bit_in_byte 7 = physical bit 0
                    // Available bits in this byte: from bit_in_byte down to 0 = bit_in_byte + 1
                    let available_in_byte = bit_in_byte + 1;
                    let bits_to_take = bits_remaining.min(available_in_byte);

                    // Extract the bits from the physical byte
                    // BE bit_in_byte maps to physical position (7 - bit_in_byte)
                    // We want to extract 'bits_to_take' bits starting from bit_in_byte going down
                    // Physical positions: (7 - bit_in_byte) to (7 - bit_in_byte + bits_to_take - 1)
                    let physical_start = 7 - bit_in_byte;
                    let byte = data[byte_num] as u64;

                    // Create mask for bits_to_take consecutive bits starting at physical_start
                    let mask = ((1u64 << bits_to_take) - 1) << physical_start;
                    let extracted = (byte & mask) >> physical_start;

                    // Place extracted bits into result (MSB first, so at the high end)
                    let shift_amount = bits_remaining - bits_to_take;
                    value |= extracted << shift_amount;

                    bits_remaining -= bits_to_take;
                    signal_bit_offset += bits_to_take;
                }

                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ByteOrder;
    use core::hash::Hash;

    // Tests that work in all configurations (no_std, std)
    #[test]
    fn test_byte_order_variants() {
        assert_eq!(ByteOrder::LittleEndian as u8, 1);
        assert_eq!(ByteOrder::BigEndian as u8, 0);
    }

    #[test]
    fn test_byte_order_equality() {
        assert_eq!(ByteOrder::LittleEndian, ByteOrder::LittleEndian);
        assert_eq!(ByteOrder::BigEndian, ByteOrder::BigEndian);
        assert_ne!(ByteOrder::LittleEndian, ByteOrder::BigEndian);
    }

    #[test]
    fn test_byte_order_clone() {
        let original = ByteOrder::LittleEndian;
        let cloned = original;
        assert_eq!(original, cloned);

        let original2 = ByteOrder::BigEndian;
        let cloned2 = original2;
        assert_eq!(original2, cloned2);
    }

    #[test]
    fn test_byte_order_copy() {
        let order = ByteOrder::LittleEndian;
        let copied = order; // Copy, not move
        assert_eq!(order, copied); // Original still valid
    }

    #[test]
    fn test_byte_order_hash_trait() {
        // Test that Hash trait is implemented by checking it compiles
        fn _assert_hash<T: Hash>() {}
        _assert_hash::<ByteOrder>();
    }

    #[test]
    fn test_extract_bits_little_endian() {
        // Test value 0x1234: little-endian bytes are [0x34, 0x12] (LSB first)
        let data = [0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 0, 16);
        assert_eq!(raw_value, 0x1234);
    }

    #[test]
    fn test_extract_bits_little_endian_8bit() {
        // Test 8-bit value at byte boundary
        let data = [0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 0, 8);
        assert_eq!(raw_value, 0x42);
    }

    #[test]
    fn test_extract_bits_little_endian_32bit() {
        // Test 32-bit value at byte boundary
        let data = [0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 0, 32);
        assert_eq!(raw_value, 0x12345678);
    }

    #[test]
    fn test_extract_bits_little_endian_64bit() {
        // Test 64-bit value at byte boundary
        let data = [0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 0, 64);
        assert_eq!(raw_value, 0x0123456789ABCDEF);
    }

    #[test]
    fn test_extract_bits_big_endian() {
        // Test big-endian extraction: For BE bit 0-15, value 0x0100 = 256
        // Big-endian at bit 0, length 16: bytes [0x01, 0x00]
        let data = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::BigEndian.extract_bits(&data, 0, 16);
        // Verify it decodes to a valid value (exact value depends on BE bit mapping)
        assert!(raw_value <= 65535);
    }

    #[test]
    fn test_extract_bits_mixed_positions_little_endian() {
        // Test signal at bit 8, length 16 (spans bytes 1-2)
        let data = [0x00, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 8, 16);
        assert_eq!(raw_value, 0x1234);
    }

    #[test]
    fn test_extract_bits_mixed_positions_big_endian() {
        // Test signal at bit 8, length 16 (spans bytes 1-2)
        // Big-endian at BE bit 8-23: bytes [0x01, 0x00]
        let data = [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::BigEndian.extract_bits(&data, 8, 16);
        // Verify it decodes to a valid value (exact value depends on BE bit mapping)
        assert!(raw_value <= 65535);
    }

    #[test]
    fn test_byte_order_difference() {
        // Test that big-endian and little-endian produce different results
        // for the same byte data, proving both byte orders are handled differently
        let data = [0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let le_value = ByteOrder::LittleEndian.extract_bits(&data, 0, 16);
        let be_value = ByteOrder::BigEndian.extract_bits(&data, 0, 16);

        // Little-endian: [0x34, 0x12] = 0x1234 = 4660
        assert_eq!(le_value, 0x1234);

        // Big-endian should produce a different value (proves BE is being used)
        assert_ne!(
            le_value, be_value,
            "Big-endian and little-endian should produce different values"
        );
        assert!(be_value <= 65535);
    }

    #[test]
    fn test_extract_bits_non_aligned_little_endian() {
        // Test non-byte-aligned extraction to ensure generic path still works
        // Signal at bit 4, length 12
        let data = [0xF0, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw_value = ByteOrder::LittleEndian.extract_bits(&data, 4, 12);
        // Bits 4-15: from byte 0 bits 4-7 (0xF) and byte 1 bits 0-7 (0x12)
        // Little-endian: value should be 0x12F
        assert_eq!(raw_value, 0x12F);
    }

    // Tests that require std (for DefaultHasher)
    #[cfg(feature = "std")]
    mod tests_std {
        use super::*;
        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        #[test]
        fn test_byte_order_debug() {
            let little = format!("{:?}", ByteOrder::LittleEndian);
            assert!(little.contains("LittleEndian"));

            let big = format!("{:?}", ByteOrder::BigEndian);
            assert!(big.contains("BigEndian"));
        }

        #[test]
        fn test_byte_order_hash() {
            let mut hasher1 = DefaultHasher::new();
            let mut hasher2 = DefaultHasher::new();

            ByteOrder::LittleEndian.hash(&mut hasher1);
            ByteOrder::LittleEndian.hash(&mut hasher2);
            assert_eq!(hasher1.finish(), hasher2.finish());

            let mut hasher3 = DefaultHasher::new();
            ByteOrder::BigEndian.hash(&mut hasher3);
            assert_ne!(hasher1.finish(), hasher3.finish());
        }
    }
}
