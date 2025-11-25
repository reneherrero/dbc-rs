use alloc::string::String;
use core::fmt;
use core::num::ParseIntError;

pub mod lang;
pub(crate) mod messages;

/// Error type for DBC parsing and validation operations.
///
/// Errors are categorized by the component that generated them, making it
/// easier to identify where validation or parsing failed.
///
/// # Examples
///
/// ```rust
/// use dbc_rs::Error;
///
/// let error = Error::Signal("Signal name cannot be empty".to_string());
/// println!("{}", error);
/// ```
#[derive(Debug, PartialEq)]
pub enum Error {
    /// General data validation or parsing error.
    InvalidData(String),
    /// Signal-specific validation or parsing error.
    Signal(String),
    /// Message-specific validation or parsing error.
    Message(String),
    /// DBC file-level validation or parsing error.
    Dbc(String),
    /// Version parsing error.
    Version(String),
    /// Node-related validation or parsing error.
    Nodes(String),
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
            Error::Message(msg) => {
                write!(f, "{}", messages::format_message_error(msg))
            }
            Error::Dbc(msg) => {
                write!(f, "{}", messages::format_dbc_error(msg))
            }
            Error::Version(msg) => {
                write!(f, "{}", messages::format_version_error(msg))
            }
            Error::Nodes(msg) => {
                write!(f, "{}", messages::format_nodes_error(msg))
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
    use crate::error::lang;
    use alloc::string::ToString;

    #[test]
    fn test_from_parse_int_error() {
        // Create a ParseIntError by trying to parse an invalid string
        let parse_error = "invalid".parse::<u32>().unwrap_err();
        let error: Error = parse_error.into();

        match error {
            Error::InvalidData(msg) => {
                assert!(msg.contains(lang::FORMAT_PARSE_NUMBER_FAILED.split(':').next().unwrap()))
            }
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_display_invalid_data() {
        let error = Error::InvalidData("Test error message".to_string());
        let display = error.to_string();
        assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
        assert!(display.contains("Test error message"));
    }

    #[test]
    fn test_display_signal_error() {
        let error = Error::Signal(lang::SIGNAL_NAME_EMPTY.to_string());
        let display = error.to_string();
        assert!(display.starts_with(lang::SIGNAL_ERROR_CATEGORY));
        assert!(display.contains(lang::SIGNAL_NAME_EMPTY));
    }

    #[test]
    fn test_display_formatting() {
        // Test that Display properly formats complex error messages
        let error = Error::InvalidData(
            "Duplicate message ID: 256 (messages 'EngineData' and 'BrakeData')".to_string(),
        );
        let display = error.to_string();
        assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
        assert!(display.contains("256"));
        assert!(display.contains("EngineData"));
        assert!(display.contains("BrakeData"));
    }

    #[test]
    fn test_display_parse_error() {
        let parse_error = "not_a_number".parse::<u32>().unwrap_err();
        let error: Error = parse_error.into();
        let display = error.to_string();

        assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
        assert!(display.contains(lang::FORMAT_PARSE_NUMBER_FAILED.split(':').next().unwrap()));
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
