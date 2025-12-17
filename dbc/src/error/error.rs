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

impl Error {
    // Error message constants
    pub const UNEXPECTED_EOF: &'static str = lang::UNEXPECTED_EOF;
    pub const EXPECTED_WHITESPACE: &'static str = lang::EXPECTED_WHITESPACE;
    pub const EXPECTED_PATTERN: &'static str = lang::EXPECTED_PATTERN;
    pub const EXPECTED_KEYWORD: &'static str = lang::EXPECTED_KEYWORD;
    pub const EXPECTED_NUMBER: &'static str = lang::EXPECTED_NUMBER;
    pub const EXPECTED_IDENTIFIER: &'static str = lang::EXPECTED_IDENTIFIER;
    pub const INVALID_UTF8: &'static str = lang::INVALID_UTF8;
    pub const INVALID_NUMBER_FORMAT: &'static str = lang::INVALID_NUMBER_FORMAT;
    pub const PARSE_NUMBER_FAILED: &'static str = lang::PARSE_NUMBER_FAILED;
    pub const INVALID_CHARACTER: &'static str = lang::INVALID_CHARACTER;
    pub const STRING_LENGTH_EXCEEDS_MAX: &'static str = lang::STRING_LENGTH_EXCEEDS_MAX;
    pub const MAX_NAME_SIZE_EXCEEDED: &'static str = lang::MAX_NAME_SIZE_EXCEEDED;

    // Error prefix constants (used in Display impl)
    pub const DECODING_ERROR_PREFIX: &'static str = lang::DECODING_ERROR_PREFIX;
    pub const VALIDATION_ERROR_PREFIX: &'static str = lang::VALIDATION_ERROR_PREFIX;
    pub const VERSION_ERROR_PREFIX: &'static str = lang::VERSION_ERROR_PREFIX;
    pub const MESSAGE_ERROR_PREFIX: &'static str = lang::MESSAGE_ERROR_PREFIX;
    pub const RECEIVERS_ERROR_PREFIX: &'static str = lang::RECEIVERS_ERROR_PREFIX;
    pub const NODES_ERROR_PREFIX: &'static str = lang::NODES_ERROR_PREFIX;
    pub const SIGNAL_ERROR_PREFIX: &'static str = lang::SIGNAL_ERROR_PREFIX;

    // Signal error constants
    pub const SIGNAL_PARSE_INVALID_START_BIT: &'static str = lang::SIGNAL_PARSE_INVALID_START_BIT;
    pub const SIGNAL_PARSE_INVALID_LENGTH: &'static str = lang::SIGNAL_PARSE_INVALID_LENGTH;
    pub const SIGNAL_PARSE_INVALID_FACTOR: &'static str = lang::SIGNAL_PARSE_INVALID_FACTOR;
    pub const SIGNAL_PARSE_INVALID_OFFSET: &'static str = lang::SIGNAL_PARSE_INVALID_OFFSET;
    pub const SIGNAL_PARSE_INVALID_MIN: &'static str = lang::SIGNAL_PARSE_INVALID_MIN;
    pub const SIGNAL_PARSE_INVALID_MAX: &'static str = lang::SIGNAL_PARSE_INVALID_MAX;
    pub const SIGNAL_PARSE_UNIT_TOO_LONG: &'static str = lang::SIGNAL_PARSE_UNIT_TOO_LONG;
    pub const SIGNAL_NAME_EMPTY: &'static str = lang::SIGNAL_NAME_EMPTY;
    pub const SIGNAL_LENGTH_TOO_SMALL: &'static str = lang::SIGNAL_LENGTH_TOO_SMALL;
    pub const SIGNAL_LENGTH_TOO_LARGE: &'static str = lang::SIGNAL_LENGTH_TOO_LARGE;
    #[cfg(feature = "std")]
    pub const SIGNAL_LENGTH_REQUIRED: &'static str = lang::SIGNAL_LENGTH_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_START_BIT_REQUIRED: &'static str = lang::SIGNAL_START_BIT_REQUIRED;
    pub const SIGNAL_OVERLAP: &'static str = lang::SIGNAL_OVERLAP;
    pub const SIGNAL_EXTENDS_BEYOND_MESSAGE: &'static str = lang::SIGNAL_EXTENDS_BEYOND_MESSAGE;
    pub const SIGNAL_EXTENDS_BEYOND_DATA: &'static str = lang::SIGNAL_EXTENDS_BEYOND_DATA;
    pub const SIGNAL_RECEIVERS_TOO_MANY: &'static str = lang::SIGNAL_RECEIVERS_TOO_MANY;

    // Validation and decoding error constants
    pub const NODES_DUPLICATE_NAME: &'static str = lang::NODES_DUPLICATE_NAME;
    pub const NODES_TOO_MANY: &'static str = lang::NODES_TOO_MANY;
    pub const DUPLICATE_MESSAGE_ID: &'static str = lang::DUPLICATE_MESSAGE_ID;
    pub const SENDER_NOT_IN_NODES: &'static str = lang::SENDER_NOT_IN_NODES;
    pub const INVALID_RANGE: &'static str = lang::INVALID_RANGE;
    pub const MESSAGE_TOO_MANY_SIGNALS: &'static str = lang::MESSAGE_TOO_MANY_SIGNALS;
    pub const EXTENDED_MULTIPLEXING_TOO_MANY: &'static str = lang::EXTENDED_MULTIPLEXING_TOO_MANY;
    pub const MESSAGE_NAME_EMPTY: &'static str = lang::MESSAGE_NAME_EMPTY;
    pub const MESSAGE_SENDER_EMPTY: &'static str = lang::MESSAGE_SENDER_EMPTY;
    pub const MESSAGE_DLC_TOO_SMALL: &'static str = lang::MESSAGE_DLC_TOO_SMALL;
    pub const MESSAGE_DLC_TOO_LARGE: &'static str = lang::MESSAGE_DLC_TOO_LARGE;
    #[cfg(feature = "std")]
    pub const MESSAGE_DLC_REQUIRED: &'static str = lang::MESSAGE_DLC_REQUIRED;
    pub const MESSAGE_ID_OUT_OF_RANGE: &'static str = lang::MESSAGE_ID_OUT_OF_RANGE;
    #[cfg(feature = "std")]
    pub const MESSAGE_ID_REQUIRED: &'static str = lang::MESSAGE_ID_REQUIRED;
    pub const MESSAGE_INVALID_ID: &'static str = lang::MESSAGE_INVALID_ID;
    pub const MESSAGE_INVALID_DLC: &'static str = lang::MESSAGE_INVALID_DLC;
    pub const MESSAGE_NOT_FOUND: &'static str = lang::MESSAGE_NOT_FOUND;
    pub const PAYLOAD_LENGTH_MISMATCH: &'static str = lang::PAYLOAD_LENGTH_MISMATCH;
    pub const MULTIPLEXER_SWITCH_NEGATIVE: &'static str = lang::MULTIPLEXER_SWITCH_NEGATIVE;
    #[cfg(feature = "std")]
    pub const RECEIVERS_DUPLICATE_NAME: &'static str = lang::RECEIVERS_DUPLICATE_NAME;
    #[cfg(feature = "std")]
    pub const VALUE_DESCRIPTION_MESSAGE_NOT_FOUND: &'static str =
        lang::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND;
    #[cfg(feature = "std")]
    pub const VALUE_DESCRIPTION_SIGNAL_NOT_FOUND: &'static str =
        lang::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND;
    #[cfg(feature = "std")]
    pub const VALUE_DESCRIPTIONS_TOO_MANY: &'static str = lang::VALUE_DESCRIPTIONS_TOO_MANY;
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEof => write!(f, "{}", Error::UNEXPECTED_EOF),
            Error::Expected(msg) => write!(f, "Expected {}", msg),
            Error::InvalidChar(c) => write!(f, "{}: {}", Error::INVALID_CHARACTER, c),
            Error::MaxStrLength(max) => {
                write!(f, "{}: {}", Error::STRING_LENGTH_EXCEEDS_MAX, max)
            }
            Error::Version(msg) => write!(f, "{}: {}", Error::VERSION_ERROR_PREFIX, msg),
            Error::Message(msg) => write!(f, "{}: {}", Error::MESSAGE_ERROR_PREFIX, msg),
            Error::Receivers(msg) => {
                write!(f, "{}: {}", Error::RECEIVERS_ERROR_PREFIX, msg)
            }
            Error::Nodes(msg) => write!(f, "{}: {}", Error::NODES_ERROR_PREFIX, msg),
            Error::Signal(msg) => write!(f, "{}: {}", Error::SIGNAL_ERROR_PREFIX, msg),
            Error::Decoding(msg) => write!(f, "{}: {}", Error::DECODING_ERROR_PREFIX, msg),
            Error::Validation(msg) => write!(f, "{}: {}", Error::VALIDATION_ERROR_PREFIX, msg),
        }
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(_err: core::num::ParseIntError) -> Self {
        Error::Expected(Error::PARSE_NUMBER_FAILED)
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
        use crate::Error;

        #[test]
        fn test_from_parse_int_error() {
            // Create a ParseIntError by trying to parse an invalid string
            let parse_error = "invalid".parse::<u32>().unwrap_err();
            let error: Error = parse_error.into();

            match error {
                Error::Expected(msg) => {
                    assert_eq!(msg, Error::PARSE_NUMBER_FAILED);
                }
                _ => panic!("Expected Error::Expected(Error::PARSE_NUMBER_FAILED)"),
            }
        }

        #[test]
        fn test_display_decoding_error() {
            let error = Error::Decoding("Test error message");
            let display = error.to_string();
            assert!(display.starts_with(Error::DECODING_ERROR_PREFIX));
            assert!(display.contains("Test error message"));
        }

        #[test]
        fn test_display_signal_error() {
            let error = Error::Signal("Test signal error");
            let display = error.to_string();
            assert!(display.starts_with(Error::SIGNAL_ERROR_PREFIX));
            assert!(display.contains("Test signal error"));
        }

        #[test]
        fn test_display_formatting() {
            // Test that Display properly formats complex error messages
            let error = Error::Decoding(
                "Duplicate message ID: 256 (messages 'EngineData' and 'BrakeData')",
            );
            let display = error.to_string();
            assert!(display.starts_with(Error::DECODING_ERROR_PREFIX));
            assert!(display.contains("256"));
            assert!(display.contains("EngineData"));
            assert!(display.contains("BrakeData"));
        }

        #[test]
        fn test_display_from_parse_int_error() {
            let int_error = "not_a_number".parse::<u32>().unwrap_err();
            let error: Error = int_error.into();
            let display = error.to_string();

            assert!(display.contains(Error::PARSE_NUMBER_FAILED));
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
