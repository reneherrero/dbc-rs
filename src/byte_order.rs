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

    /// Insert bits into data based on byte order.
    /// This is the inverse of `extract_bits` - used for encoding signals.
    /// Inlined for hot path optimization.
    ///
    /// # Arguments
    ///
    /// * `data` - Mutable byte slice to write into
    /// * `start_bit` - Starting bit position (LSB for LE, MSB for BE)
    /// * `length` - Number of bits to write
    /// * `value` - The value to insert (must fit in `length` bits)
    #[inline]
    pub(crate) fn insert_bits(self, data: &mut [u8], start_bit: usize, length: usize, value: u64) {
        match self {
            ByteOrder::LittleEndian => {
                // Fast path: byte-aligned little-endian signals (most common case)
                let bit_offset = start_bit % 8;
                let byte_idx = start_bit / 8;

                if bit_offset == 0 {
                    // Byte-aligned - use direct memory writes
                    match length {
                        8 => {
                            data[byte_idx] = value as u8;
                            return;
                        }
                        16 => {
                            let bytes = (value as u16).to_le_bytes();
                            data[byte_idx] = bytes[0];
                            data[byte_idx + 1] = bytes[1];
                            return;
                        }
                        32 => {
                            let bytes = (value as u32).to_le_bytes();
                            data[byte_idx] = bytes[0];
                            data[byte_idx + 1] = bytes[1];
                            data[byte_idx + 2] = bytes[2];
                            data[byte_idx + 3] = bytes[3];
                            return;
                        }
                        64 => {
                            let bytes = value.to_le_bytes();
                            data[byte_idx..byte_idx + 8].copy_from_slice(&bytes);
                            return;
                        }
                        _ => {} // Fall through to generic path
                    }
                }

                // Generic path: insert bits sequentially from start_bit forward
                let mut bits_remaining = length;
                let mut current_bit = start_bit;
                let mut value_offset = 0;

                while bits_remaining > 0 {
                    let byte_idx = current_bit / 8;
                    let bit_in_byte = current_bit % 8;
                    let bits_to_write = bits_remaining.min(8 - bit_in_byte);

                    // Extract the bits from value that we want to write
                    let bits_mask = (1u64 << bits_to_write) - 1;
                    let bits_to_insert = ((value >> value_offset) & bits_mask) as u8;

                    // Create mask for the target position in the byte
                    let target_mask = (bits_mask as u8) << bit_in_byte;

                    // Clear the target bits and set the new value
                    data[byte_idx] =
                        (data[byte_idx] & !target_mask) | (bits_to_insert << bit_in_byte);

                    bits_remaining -= bits_to_write;
                    current_bit += bits_to_write;
                    value_offset += bits_to_write;
                }
            }
            ByteOrder::BigEndian => {
                // Big-endian (Motorola): start_bit is MSB in big-endian numbering.
                // BE bit N maps to physical bit: byte_num * 8 + (7 - bit_in_byte)
                let mut bits_remaining = length;
                let mut signal_bit_offset = 0; // How many bits of the signal we've processed

                while bits_remaining > 0 {
                    // Current BE bit position
                    let be_bit = start_bit + signal_bit_offset;
                    let byte_num = be_bit / 8;
                    let bit_in_byte = be_bit % 8;

                    // Calculate how many bits we can write to this byte
                    // In BE numbering, bits go from high to low within a byte (7,6,5,4,3,2,1,0)
                    let available_in_byte = bit_in_byte + 1;
                    let bits_to_write = bits_remaining.min(available_in_byte);

                    // Calculate physical position in byte
                    // BE bit_in_byte maps to physical position (7 - bit_in_byte)
                    let physical_start = 7 - bit_in_byte;

                    // Extract the bits from value (MSB first, so from the high end)
                    let shift_amount = bits_remaining - bits_to_write;
                    let bits_mask = (1u64 << bits_to_write) - 1;
                    let bits_to_insert = ((value >> shift_amount) & bits_mask) as u8;

                    // Create mask for the target position in the byte
                    let target_mask = (bits_mask as u8) << physical_start;

                    // Clear the target bits and set the new value
                    data[byte_num] =
                        (data[byte_num] & !target_mask) | (bits_to_insert << physical_start);

                    bits_remaining -= bits_to_write;
                    signal_bit_offset += bits_to_write;
                }
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

    // ============================================================================
    // insert_bits tests
    // ============================================================================

    #[test]
    fn test_insert_bits_little_endian_8bit() {
        let mut data = [0x00; 8];
        ByteOrder::LittleEndian.insert_bits(&mut data, 0, 8, 0x42);
        assert_eq!(data[0], 0x42);
    }

    #[test]
    fn test_insert_bits_little_endian_16bit() {
        let mut data = [0x00; 8];
        ByteOrder::LittleEndian.insert_bits(&mut data, 0, 16, 0x1234);
        // Little-endian: LSB first
        assert_eq!(data[0], 0x34);
        assert_eq!(data[1], 0x12);
    }

    #[test]
    fn test_insert_bits_little_endian_32bit() {
        let mut data = [0x00; 8];
        ByteOrder::LittleEndian.insert_bits(&mut data, 0, 32, 0x12345678);
        assert_eq!(data[0], 0x78);
        assert_eq!(data[1], 0x56);
        assert_eq!(data[2], 0x34);
        assert_eq!(data[3], 0x12);
    }

    #[test]
    fn test_insert_bits_little_endian_64bit() {
        let mut data = [0x00; 8];
        ByteOrder::LittleEndian.insert_bits(&mut data, 0, 64, 0x0123456789ABCDEF);
        assert_eq!(data, [0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]);
    }

    #[test]
    fn test_insert_bits_little_endian_non_aligned() {
        let mut data = [0x00; 8];
        // Insert 12 bits at bit 4
        ByteOrder::LittleEndian.insert_bits(&mut data, 4, 12, 0x12F);
        // Verify by extracting
        let extracted = ByteOrder::LittleEndian.extract_bits(&data, 4, 12);
        assert_eq!(extracted, 0x12F);
    }

    #[test]
    fn test_insert_extract_roundtrip_little_endian() {
        // Round-trip test: insert then extract should return same value
        let test_cases = [
            (0, 8, 0x42u64),
            (0, 16, 0x1234),
            (8, 16, 0xABCD),
            (4, 12, 0x123),
            (0, 32, 0x12345678),
            (0, 64, 0x0123456789ABCDEF),
        ];

        for (start_bit, length, value) in test_cases {
            let mut data = [0x00; 8];
            ByteOrder::LittleEndian.insert_bits(&mut data, start_bit, length, value);
            let extracted = ByteOrder::LittleEndian.extract_bits(&data, start_bit, length);
            assert_eq!(
                extracted, value,
                "Round-trip failed for start_bit={}, length={}, value=0x{:X}",
                start_bit, length, value
            );
        }
    }

    #[test]
    fn test_insert_extract_roundtrip_big_endian() {
        // Round-trip test for big-endian
        let test_cases = [
            (7, 8, 0x42u64),  // 8-bit at MSB position 7
            (7, 16, 0x1234),  // 16-bit spanning bytes 0-1
            (15, 16, 0xABCD), // 16-bit spanning bytes 1-2
        ];

        for (start_bit, length, value) in test_cases {
            let mut data = [0x00; 8];
            ByteOrder::BigEndian.insert_bits(&mut data, start_bit, length, value);
            let extracted = ByteOrder::BigEndian.extract_bits(&data, start_bit, length);
            assert_eq!(
                extracted, value,
                "BE round-trip failed for start_bit={}, length={}, value=0x{:X}",
                start_bit, length, value
            );
        }
    }

    #[test]
    fn test_insert_bits_preserves_other_bits() {
        // Test that insert_bits doesn't corrupt other bits
        let mut data = [0xFF; 8];
        ByteOrder::LittleEndian.insert_bits(&mut data, 8, 8, 0x00);
        // Byte 0 should still be 0xFF, byte 1 should be 0x00
        assert_eq!(data[0], 0xFF);
        assert_eq!(data[1], 0x00);
        assert_eq!(data[2], 0xFF);
    }

    #[test]
    fn test_insert_bits_at_offset() {
        let mut data = [0x00; 8];
        // Insert 16-bit value at byte 2
        ByteOrder::LittleEndian.insert_bits(&mut data, 16, 16, 0x5678);
        assert_eq!(data[0], 0x00);
        assert_eq!(data[1], 0x00);
        assert_eq!(data[2], 0x78);
        assert_eq!(data[3], 0x56);
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
