use alloc::string::String;
use core::fmt;
use core::num::ParseIntError;

mod lang;
pub(crate) mod messages;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidData(String),
    Signal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData(msg) => {
                // Display the message with category prefix for better readability
                write!(f, "{}", messages::format_invalid_data(msg))
            }
            Error::Signal(msg) => {
                // Display the message with category prefix for better readability
                write!(f, "{}", messages::format_signal_error(msg))
            }
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::InvalidData(messages::parse_number_failed(err))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use alloc::string::ToString;

    #[test]
    fn test_from_parse_int_error() {
        // Create a ParseIntError by trying to parse an invalid string
        let parse_error = "invalid".parse::<u32>().unwrap_err();
        let error: Error = parse_error.into();

        match error {
            Error::InvalidData(msg) => assert!(msg.contains("Failed to parse number")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_display_invalid_data() {
        let error = Error::InvalidData("Test error message".to_string());
        let display = error.to_string();
        assert!(display.starts_with("Data Error:"));
        assert!(display.contains("Test error message"));
    }

    #[test]
    fn test_display_signal_error() {
        let error = Error::Signal("Signal name cannot be empty".to_string());
        let display = error.to_string();
        assert!(display.starts_with("Signal Error:"));
        assert!(display.contains("Signal name cannot be empty"));
    }

    #[test]
    fn test_display_formatting() {
        // Test that Display properly formats complex error messages
        let error = Error::InvalidData(
            "Duplicate message ID: 256 (messages 'EngineData' and 'BrakeData')".to_string(),
        );
        let display = error.to_string();
        assert!(display.starts_with("Data Error:"));
        assert!(display.contains("Duplicate message ID"));
        assert!(display.contains("256"));
        assert!(display.contains("EngineData"));
        assert!(display.contains("BrakeData"));
    }

    #[test]
    fn test_display_parse_error() {
        let parse_error = "not_a_number".parse::<u32>().unwrap_err();
        let error: Error = parse_error.into();
        let display = error.to_string();

        assert!(display.starts_with("Data Error:"));
        assert!(display.contains("Failed to parse number"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_std_error_trait() {
        use std::error::Error as StdError;

        let error = Error::InvalidData("Test".to_string());
        // Verify it implements std::error::Error
        let _: &dyn StdError = &error;

        // Verify source() returns None (no underlying error)
        assert!(error.source().is_none());
    }
}
