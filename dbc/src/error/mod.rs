use core::{convert::From, fmt};

// ParseIntError is used in From<ParseIntError> implementations
// The warning appears when default features (std/alloc) are enabled with kernel
// because the kernel-specific implementation isn't compiled in that case
#[cfg(any(feature = "alloc", feature = "kernel"))]
#[allow(unused_imports)]
use core::num::ParseIntError;

// Type alias for String based on feature flags
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
use crate::kernel::alloc::string::String as ErrorString;
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
use alloc::string::String as ErrorString;

// Helper function to convert &str to ErrorString
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub(crate) fn str_to_error_string(s: &str) -> ErrorString {
    ErrorString::from_str(s)
}

#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub(crate) fn str_to_error_string(s: &str) -> ErrorString {
    ErrorString::from(s)
}

pub mod lang;
pub(crate) mod messages;

/// Error type for DBC parsing and validation operations.
///
/// This enum represents all possible errors that can occur when working with DBC files.
/// Most variants require the `alloc` feature to be enabled.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Invalid data error (e.g., parse failures, invalid formats).
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    InvalidData(ErrorString),

    /// Signal-related error (e.g., invalid signal definition).
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    Signal(ErrorString),

    /// Message-related error (e.g., invalid message definition).
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    Message(ErrorString),

    /// DBC file-level error (e.g., missing required sections).
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    Dbc(ErrorString),

    /// Version parsing error.
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    Version(ErrorString),

    /// Node-related error (e.g., duplicate node names).
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    Nodes(ErrorString),

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

// ============================================================================
// Helper function to convert String to ParseError::Version(&'static str)
// ============================================================================

/// Converts a String error message to a `ParseError::Version` with a static lifetime.
///
/// This function takes ownership of the String, boxes it, and leaks it to create
/// a `&'static str` that can be used in `ParseError::Version`.
///
/// # Safety
///
/// This function leaks memory. The leaked memory will live for the duration of
/// the program. This is acceptable for error messages that are typically displayed
/// once and then the program exits or handles the error.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::error::{messages, version_error_from_string};
///
/// let msg = messages::message_id_out_of_range(0x20000000);
/// return Err(version_error_from_string(msg));
/// ```
#[cfg(feature = "alloc")]
pub(crate) fn version_error_from_string(msg: ErrorString) -> ParseError {
    use alloc::boxed::Box;
    ParseError::Version(Box::leak(msg.into_boxed_str()))
}

// Separate Display implementations for alloc and kernel (Strategy 4)
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData(msg) => write!(f, "{}", messages::format_invalid_data(msg)),
            Error::Signal(msg) => write!(f, "{}", messages::format_signal_error(msg)),
            Error::Message(msg) => write!(f, "{}", messages::format_message_error(msg)),
            Error::Dbc(msg) => write!(f, "{}", messages::format_dbc_error(msg)),
            Error::Version(msg) => write!(f, "{}", messages::format_version_error(msg)),
            Error::Nodes(msg) => write!(f, "{}", messages::format_nodes_error(msg)),
            Error::ParseError(msg) => write!(f, "Parse Error: {}", msg),
        }
    }
}

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData(msg) => {
                let formatted = messages::format_invalid_data(msg);
                write!(f, "{}", formatted)
            }
            Error::Signal(msg) => {
                let formatted = messages::format_signal_error(msg);
                write!(f, "{}", formatted)
            }
            Error::Message(msg) => {
                let formatted = messages::format_message_error(msg);
                write!(f, "{}", formatted)
            }
            Error::Dbc(msg) => {
                let formatted = messages::format_dbc_error(msg);
                write!(f, "{}", formatted)
            }
            Error::Version(msg) => {
                let formatted = messages::format_version_error(msg);
                write!(f, "{}", formatted)
            }
            Error::Nodes(msg) => {
                let formatted = messages::format_nodes_error(msg);
                write!(f, "{}", formatted)
            }
            Error::ParseError(msg) => write!(f, "Parse Error: {}", msg),
        }
    }
}

// Separate From<ParseIntError> implementations for alloc vs kernel
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::InvalidData(messages::parse_number_failed(err))
    }
}

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        // In kernel mode, parse_number_failed returns String (infallible)
        Error::InvalidData(messages::parse_number_failed(err))
    }
}

#[cfg(not(any(feature = "alloc", feature = "kernel")))]
impl From<core::num::ParseIntError> for Error {
    fn from(_err: core::num::ParseIntError) -> Self {
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

// std::error::Error is only available with std feature (which requires alloc)
// Display is already implemented for alloc feature, so this should work
#[cfg(all(feature = "std", feature = "alloc", not(feature = "kernel")))]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    // Tests that require alloc or kernel feature (for Display/ToString)
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    mod tests_with_alloc {
        use crate::error::{Error, lang};

        #[test]
        fn test_from_parse_int_error() {
            // Create a ParseIntError by trying to parse an invalid string
            let parse_error = "invalid".parse::<u32>().unwrap_err();
            let error: Error = parse_error.into();

            match error {
                Error::InvalidData(msg) => {
                    assert!(
                        msg.contains(lang::FORMAT_PARSE_NUMBER_FAILED.split(':').next().unwrap())
                    );
                }
                _ => panic!("Expected InvalidData error"),
            }
        }

        #[test]
        fn test_display_invalid_data() {
            use crate::compat::{display_to_string, str_to_string};
            let error = Error::InvalidData(str_to_string("Test error message"));
            let display = display_to_string(error);
            assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
            assert!(display.contains("Test error message"));
        }

        #[test]
        fn test_display_signal_error() {
            use crate::compat::{display_to_string, str_to_string};
            let error = Error::Signal(str_to_string("Test signal error"));
            let display = display_to_string(error);
            assert!(display.starts_with(lang::SIGNAL_ERROR_CATEGORY));
            assert!(display.contains("Test signal error"));
        }

        #[test]
        fn test_display_formatting() {
            use crate::compat::{display_to_string, str_to_string};
            // Test that Display properly formats complex error messages
            let error = Error::InvalidData(str_to_string(
                "Duplicate message ID: 256 (messages 'EngineData' and 'BrakeData')",
            ));
            let display = display_to_string(error);
            assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
            assert!(display.contains("256"));
            assert!(display.contains("EngineData"));
            assert!(display.contains("BrakeData"));
        }

        #[test]
        fn test_display_parse_error() {
            use crate::compat::display_to_string;
            let parse_error = "not_a_number".parse::<u32>().unwrap_err();
            let error: Error = parse_error.into();
            let display = display_to_string(error);

            assert!(display.starts_with(lang::INVALID_DATA_CATEGORY));
            assert!(display.contains(lang::FORMAT_PARSE_NUMBER_FAILED.split(':').next().unwrap()));
        }
    }

    // Tests that require std feature (for std::error::Error trait)
    #[cfg(feature = "std")]
    mod tests_std {
        use crate::error::Error;
        use std::error::Error as StdError;

        #[test]
        fn test_std_error_trait() {
            use crate::compat::str_to_string;
            let error = Error::InvalidData(str_to_string("Test"));
            // Verify it implements std::error::Error
            let _: &dyn StdError = &error;

            // Verify source() returns None (no underlying error)
            assert!(error.source().is_none());
        }
    }
}
