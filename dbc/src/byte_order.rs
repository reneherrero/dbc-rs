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
