/// Byte order (endianness) for signal encoding.
///
/// Determines how multi-byte signals are encoded within the CAN message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Little-endian byte order (Intel format).
    LittleEndian = 0,
    /// Big-endian byte order (Motorola format).
    BigEndian = 1,
}
