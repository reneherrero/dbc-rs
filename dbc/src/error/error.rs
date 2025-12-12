use core::fmt;

use crate::error::lang;

/// Error type for DBC operations.
///
/// This enum represents all possible errors that can occur when working with DBC files.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// Unexpected end of input encountered.
    UnexpectedEof,
    /// Expected a specific token or value.
    Expected(&'static str),
    /// Invalid character encountered.
    InvalidChar(char),
    /// String length exceeds the maximum allowed length.
    MaxStrLength(usize),
    /// Version-related error.
    Version(&'static str),
    /// Message-related error.
    Message(&'static str),
    /// Receivers-related error.
    Receivers(&'static str),
    /// Nodes-related error.
    Nodes(&'static str),
    /// Signal-related error.
    Signal(&'static str),
    /// Decoding-related error.
    Decoding(&'static str),
    /// Validation-related error.
    Validation(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEof => write!(f, "{}", lang::UNEXPECTED_EOF),
            Error::Expected(msg) => write!(f, "Expected {}", msg),
            Error::InvalidChar(c) => write!(f, "{}: {}", lang::INVALID_CHARACTER, c),
            Error::MaxStrLength(max) => {
                write!(f, "{}: {}", lang::STRING_LENGTH_EXCEEDS_MAX, max)
            }
            Error::Version(msg) => write!(f, "{}: {}", lang::VERSION_ERROR_PREFIX, msg),
            Error::Message(msg) => write!(f, "{}: {}", lang::MESSAGE_ERROR_PREFIX, msg),
            Error::Receivers(msg) => {
                write!(f, "{}: {}", lang::RECEIVERS_ERROR_PREFIX, msg)
            }
            Error::Nodes(msg) => write!(f, "{}: {}", lang::NODES_ERROR_PREFIX, msg),
            Error::Signal(msg) => write!(f, "{}: {}", lang::SIGNAL_ERROR_PREFIX, msg),
            Error::Decoding(msg) => write!(f, "{}: {}", lang::DECODING_ERROR_PREFIX, msg),
            Error::Validation(msg) => write!(f, "{}: {}", lang::VALIDATION_ERROR_PREFIX, msg),
        }
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(_err: core::num::ParseIntError) -> Self {
        Error::Expected(lang::PARSE_NUMBER_FAILED)
    }
}

// std::error::Error is only available with std feature
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
        use super::super::Error;
        use crate::error::lang;

        #[test]
        fn test_from_parse_int_error() {
            // Create a ParseIntError by trying to parse an invalid string
            let parse_error = "invalid".parse::<u32>().unwrap_err();
            let error: Error = parse_error.into();

            match error {
                Error::Expected(msg) => {
                    assert_eq!(msg, lang::PARSE_NUMBER_FAILED);
                }
                _ => panic!("Expected Error::Expected(lang::PARSE_NUMBER_FAILED)"),
            }
        }

        #[test]
        fn test_display_decoding_error() {
            let error = Error::Decoding("Test error message");
            let display = error.to_string();
            assert!(display.starts_with(lang::DECODING_ERROR_PREFIX));
            assert!(display.contains("Test error message"));
        }

        #[test]
        fn test_display_signal_error() {
            let error = Error::Signal("Test signal error");
            let display = error.to_string();
            assert!(display.starts_with(lang::SIGNAL_ERROR_PREFIX));
            assert!(display.contains("Test signal error"));
        }

        #[test]
        fn test_display_formatting() {
            // Test that Display properly formats complex error messages
            let error = Error::Decoding(
                "Duplicate message ID: 256 (messages 'EngineData' and 'BrakeData')",
            );
            let display = error.to_string();
            assert!(display.starts_with(lang::DECODING_ERROR_PREFIX));
            assert!(display.contains("256"));
            assert!(display.contains("EngineData"));
            assert!(display.contains("BrakeData"));
        }

        #[test]
        fn test_display_from_parse_int_error() {
            let int_error = "not_a_number".parse::<u32>().unwrap_err();
            let error: Error = int_error.into();
            let display = error.to_string();

            assert!(display.contains(lang::PARSE_NUMBER_FAILED));
        }
    }

    // Tests that require std feature (for std::error::Error trait)
    // Only available when std is enabled
    #[cfg(feature = "std")]
    mod tests_std {
        use super::super::Error;
        use std::error::Error as StdError;

        #[test]
        fn test_std_error_trait() {
            let error = Error::Decoding("Test");
            // Verify it implements std::error::Error
            let _: &dyn StdError = &error;

            // Verify source() returns None (no underlying error)
            assert!(error.source().is_none());
        }
    }
}
