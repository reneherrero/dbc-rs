//! Signal extended value type (SIG_VALTYPE_)
//!
//! Specifies how a signal's raw bits should be interpreted.
//! This is used for signals that contain IEEE 754 floating-point values
//! instead of integer values.
//!
//! # Specification (Section 10.6)
//!
//! - `0` = Signed or unsigned integer (default)
//! - `1` = 32-bit IEEE 754 float
//! - `2` = 64-bit IEEE 754 double
//! - `3` = Reserved

use crate::error::{Error, Result, lang};

/// Signal extended value type (SIG_VALTYPE_)
///
/// Specifies how a signal's raw bits should be interpreted.
/// This is used for signals that contain IEEE 754 floating-point values
/// instead of integer values.
///
/// # Specification (Section 10.6)
///
/// - `0` = Signed or unsigned integer (default)
/// - `1` = 32-bit IEEE 754 float
/// - `2` = 64-bit IEEE 754 double
/// - `3` = Reserved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalExtendedValueType {
    /// Integer (default) - signed or unsigned integer
    Integer,
    /// 32-bit IEEE 754 float
    Float32,
    /// 64-bit IEEE 754 double
    Float64,
}

impl SignalExtendedValueType {
    /// Parse value type from integer (0, 1, or 2)
    pub(crate) fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(SignalExtendedValueType::Integer),
            1 => Ok(SignalExtendedValueType::Float32),
            2 => Ok(SignalExtendedValueType::Float64),
            _ => Err(Error::Validation(lang::INVALID_SIGNAL_VALUE_TYPE)),
        }
    }

    /// Convert to integer representation
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            SignalExtendedValueType::Integer => 0,
            SignalExtendedValueType::Float32 => 1,
            SignalExtendedValueType::Float64 => 2,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_from_u8() {
        assert_eq!(SignalExtendedValueType::from_u8(0).unwrap(), SignalExtendedValueType::Integer);
        assert_eq!(SignalExtendedValueType::from_u8(1).unwrap(), SignalExtendedValueType::Float32);
        assert_eq!(SignalExtendedValueType::from_u8(2).unwrap(), SignalExtendedValueType::Float64);
        assert!(SignalExtendedValueType::from_u8(3).is_err());
    }

    #[test]
    fn test_to_u8() {
        assert_eq!(SignalExtendedValueType::Integer.to_u8(), 0);
        assert_eq!(SignalExtendedValueType::Float32.to_u8(), 1);
        assert_eq!(SignalExtendedValueType::Float64.to_u8(), 2);
    }
}

