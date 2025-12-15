//! Bit Timing definition (BS_)
//!
//! Represents the bit timing configuration for the CAN bus.
//! Typically empty or obsolete in modern CAN systems.

/// Bit Timing definition (BS_)
///
/// Represents the bit timing configuration for the CAN bus.
/// Typically empty or obsolete in modern CAN systems.
#[derive(Debug, Clone, PartialEq)]
pub struct BitTiming {
    baudrate: Option<u32>,
    btr1: Option<u32>,
    btr2: Option<u32>,
}

impl BitTiming {
    /// Create a new BitTiming with optional values
    pub(crate) fn new(baudrate: Option<u32>, btr1: Option<u32>, btr2: Option<u32>) -> Self {
        Self {
            baudrate,
            btr1,
            btr2,
        }
    }

    /// Get the baudrate (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn baudrate(&self) -> Option<u32> {
        self.baudrate
    }

    /// Get BTR1 (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn btr1(&self) -> Option<u32> {
        self.btr1
    }

    /// Get BTR2 (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn btr2(&self) -> Option<u32> {
        self.btr2
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_bit_timing_creation() {
        let bt = BitTiming::new(Some(500), Some(12), Some(34));
        assert_eq!(bt.baudrate(), Some(500));
        assert_eq!(bt.btr1(), Some(12));
        assert_eq!(bt.btr2(), Some(34));
    }

    #[test]
    fn test_bit_timing_empty() {
        let bt = BitTiming::new(None, None, None);
        assert_eq!(bt.baudrate(), None);
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);
    }

    // Integration tests using Dbc::parse
    #[test]
    fn test_parse_bs_empty() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_:
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse empty BS_");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_bs_with_baudrate() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_: 500
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse BS_ with baudrate");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_bs_with_btr() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_: 500 : 12,34
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse BS_ with BTR values");
        assert_eq!(dbc.messages().len(), 0);
    }
}
