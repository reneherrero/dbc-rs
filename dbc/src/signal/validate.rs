use super::Signal;
use crate::{Error, Result};

impl Signal {
    pub(crate) fn validate(name: &str, length: u16, min: f64, max: f64) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::Validation(Error::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        // Note: name is parsed before this validation, so we can include it in error messages
        if length == 0 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_SMALL));
        }
        if length > 512 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_LARGE));
        }

        // Note: start_bit validation (boundary checks and overlap detection) is done in
        // Message::validate, not here, because:
        // 1. The actual message size depends on DLC (1-64 bytes for CAN FD)
        // 2. Overlap detection requires comparing multiple signals
        // 3. This allows signals to be created independently and validated when added to a message

        // Validate min <= max
        if min > max {
            return Err(Error::Validation(Error::INVALID_RANGE));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_signal() {
        let result = Signal::validate("ValidSignal", 8, 0.0, 255.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let result = Signal::validate("", 8, 0.0, 255.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_whitespace_name() {
        let result = Signal::validate("   ", 8, 0.0, 255.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_length() {
        let result = Signal::validate("Signal", 0, 0.0, 255.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_length_one() {
        let result = Signal::validate("Signal", 1, 0.0, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_length_max() {
        // CAN FD maximum is 512 bits
        let result = Signal::validate("Signal", 512, 0.0, 1000.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_length_too_large() {
        let result = Signal::validate("Signal", 513, 0.0, 1000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_min_equals_max() {
        let result = Signal::validate("Signal", 8, 100.0, 100.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_min_greater_than_max() {
        let result = Signal::validate("Signal", 8, 200.0, 100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_negative_range() {
        let result = Signal::validate("Signal", 8, -100.0, -50.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_spanning_zero_range() {
        let result = Signal::validate("Signal", 16, -1000.0, 1000.0);
        assert!(result.is_ok());
    }
}
