use core::{convert::From, fmt, num::ParseIntError};

pub mod lang;
pub(crate) mod messages;

/// Error type for DBC parsing and validation operations.
///
/// This enum represents all possible errors that can occur when working with DBC files.
/// Most variants require the `alloc` feature to be enabled.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Invalid data error (e.g., parse failures, invalid formats).
    #[cfg(feature = "alloc")]
    InvalidData(String),

    /// Signal-related error (e.g., invalid signal definition).
    #[cfg(feature = "alloc")]
    Signal(String),

    /// Message-related error (e.g., invalid message definition).
    #[cfg(feature = "alloc")]
    Message(String),

    /// DBC file-level error (e.g., missing required sections).
    #[cfg(feature = "alloc")]
    Dbc(String),

    /// Version parsing error.
    #[cfg(feature = "alloc")]
    Version(String),

    /// Node-related error (e.g., duplicate node names).
    #[cfg(feature = "alloc")]
    Nodes(String),

    /// Low-level parse error (available in `no_std` builds).
    ParseError(ParseError),
}

/// Low-level parsing error that can occur during DBC file parsing.
///
/// This error type is available in both `std` and `no_std` builds.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseError {
    /// Unexpected end of input encountered.
    UnexpectedEof,

    /// Expected a specific token or value.
    Expected(&'static str),

    /// Invalid character encountered.
    InvalidChar(char),

    /// String length exceeds the maximum allowed length.
    MaxStrLength(u16),

    /// Version-related parse error.
    Version(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::Expected(msg) => write!(f, "Expected {}", msg),
            ParseError::InvalidChar(c) => write!(f, "Invalid character: {}", c),
            ParseError::MaxStrLength(max) => write!(f, "String length exceeds maximum: {}", max),
            ParseError::Version(msg) => write!(f, "Version error: {}", msg),
        }
    }
}

/// Result type alias for operations that can return an `Error`.
pub type Result<T> = core::result::Result<T, Error>;

/// Result type alias for low-level parsing operations that can return a `ParseError`.
pub type ParseResult<T> = core::result::Result<T, ParseError>;

#[cfg(feature = "alloc")]
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
            Error::ParseError(msg) => {
                write!(f, "Parse Error: {}", msg)
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::InvalidData(messages::parse_number_failed(err))
    }
}

#[cfg(not(feature = "alloc"))]
impl From<ParseIntError> for Error {
    fn from(_err: ParseIntError) -> Self {
        // In no_std, we can only return ParseError
        // ParseIntError conversion is not fully supported in no_std
        Error::ParseError(ParseError::Expected("Invalid number format"))
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseError(err)
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
    #![allow(clippy::float_cmp)]
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
                assert!(msg.contains(lang::FORMAT_PARSE_NUMBER_FAILED.split(':').next().unwrap()));
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
