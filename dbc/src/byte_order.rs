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
    LittleEndian = 0,
    /// Big-endian byte order (Motorola format, `0` in DBC files).
    ///
    /// Bytes are ordered from most significant to least significant.
    BigEndian = 1,
}
