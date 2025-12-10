pub mod lang;

use core::{convert::From, fmt};

/// Error type for DBC parsing and validation operations.
///
/// This enum represents all possible errors that can occur when working with DBC files.
/// Most variants require the `std` feature to be enabled.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Invalid data error (e.g., parse failures, invalid formats).
    #[cfg(feature = "std")]
    InvalidData(String),

    /// Signal-related error (e.g., invalid signal definition).
    #[cfg(feature = "std")]
    Signal(String),

    /// Message-related error (e.g., invalid message definition).
    #[cfg(feature = "std")]
    Message(String),

    /// DBC file-level error (e.g., missing required sections).
    #[cfg(feature = "std")]
    Dbc(String),

    /// Version parsing error.
    #[cfg(feature = "std")]
    Version(String),

    /// Node-related error (e.g., duplicate node names).
    #[cfg(feature = "std")]
    Nodes(String),

    /// Decoding-related parse error.
    Decoding(&'static str),

    /// Validation-related parse error.
    Validation(&'static str),

    /// Low-level parse error.
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

    /// Message-related parse error.
    Message(&'static str),

    /// Receivers-related parse error.
    Receivers(&'static str),

    /// Nodes-related parse error.
    Nodes(&'static str),

    /// Signal-related parse error.
    Signal(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::Expected(msg) => write!(f, "Expected {}", msg),
            ParseError::InvalidChar(c) => write!(f, "Invalid character: {}", c),
            ParseError::MaxStrLength(max) => write!(f, "String length exceeds maximum: {}", max),
            ParseError::Version(msg) => write!(f, "Version error: {}", msg),
            ParseError::Message(msg) => write!(f, "Message error: {}", msg),
            ParseError::Receivers(msg) => write!(f, "Receivers error: {}", msg),
            ParseError::Nodes(msg) => write!(f, "Nodes error: {}", msg),
            ParseError::Signal(msg) => write!(f, "Signal error: {}", msg),
        }
    }
}

/// Result type alias for operations that can return an `Error`.
pub type Result<T> = core::result::Result<T, Error>;

/// Result type alias for low-level parsing operations that can return a `ParseError`.
pub type ParseResult<T> = core::result::Result<T, ParseError>;

// Unified Display implementation for alloc
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Error::InvalidData(msg) => write!(f, "{}: {}", lang::INVALID_DATA_CATEGORY, msg),
            #[cfg(feature = "std")]
            Error::Signal(msg) => write!(f, "{}: {}", lang::SIGNAL_ERROR_CATEGORY, msg),
            #[cfg(feature = "std")]
            Error::Message(msg) => write!(f, "{}: {}", lang::MESSAGE_ERROR_CATEGORY, msg),
            #[cfg(feature = "std")]
            Error::Dbc(msg) => write!(f, "{}: {}", lang::DBC_ERROR_CATEGORY, msg),
            #[cfg(feature = "std")]
            Error::Version(msg) => write!(f, "{}: {}", lang::VERSION_ERROR_CATEGORY, msg),
            #[cfg(feature = "std")]
            Error::Nodes(msg) => write!(f, "{}: {}", lang::NODES_ERROR_CATEGORY, msg),
            Error::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            Error::Decoding(msg) => write!(f, "Decoding Error: {}", msg),
            Error::Validation(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(_err: core::num::ParseIntError) -> Self {
        Error::ParseError(ParseError::Expected(lang::PARSE_NUMBER_FAILED))
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseError(err)
    }
}

// std::error::Error is only available with std feature
// Display is already implemented for std feature, so this should work
#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    // Tests that require std feature (for Display/ToString)
    #[cfg(feature = "std")]
    mod tests_with_std {
        use crate::{
            ParseError,
            error::{Error, lang},
        };

        #[test]
        fn test_from_parse_int_error() {
            // Create a ParseIntError by trying to parse an invalid string
            let parse_error = "invalid".parse::<u32>().unwrap_err();
            let error: Error = parse_error.into();

            match error {
                Error::ParseError(ParseError::Expected(msg)) => {
                    assert_eq!(msg, lang::PARSE_NUMBER_FAILED);
                }
                _ => panic!("Expected ParseError::Expected(lang::PARSE_NUMBER_FAILED)"),
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
            let error = Error::Signal("Test signal error".to_string());
            let display = error.to_string();
            assert!(display.starts_with(lang::SIGNAL_ERROR_CATEGORY));
            assert!(display.contains("Test signal error"));
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

            assert!(display.contains(lang::PARSE_NUMBER_FAILED));
        }
    }

    // Tests that require std feature (for std::error::Error trait)
    // Only available when std is enabled
    #[cfg(feature = "std")]
    mod tests_std {
        use crate::error::Error;
        use std::error::Error as StdError;

        #[test]
        fn test_std_error_trait() {
            let error = Error::InvalidData("Test".to_string());
            // Verify it implements std::error::Error
            let _: &dyn StdError = &error;

            // Verify source() returns None (no underlying error)
            assert!(error.source().is_none());
        }
    }
}
