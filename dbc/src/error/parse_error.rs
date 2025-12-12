use core::fmt;

use crate::error::lang;

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
    MaxStrLength(usize),

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
            ParseError::UnexpectedEof => write!(f, "{}", lang::UNEXPECTED_EOF),
            ParseError::Expected(msg) => write!(f, "Expected {}", msg),
            ParseError::InvalidChar(c) => write!(f, "{}: {}", lang::INVALID_CHARACTER, c),
            ParseError::MaxStrLength(max) => {
                write!(f, "{}: {}", lang::STRING_LENGTH_EXCEEDS_MAX, max)
            }
            ParseError::Version(msg) => write!(f, "{}: {}", lang::VERSION_ERROR_PREFIX, msg),
            ParseError::Message(msg) => write!(f, "{}: {}", lang::MESSAGE_ERROR_PREFIX, msg),
            ParseError::Receivers(msg) => {
                write!(f, "{}: {}", lang::RECEIVERS_ERROR_PREFIX, msg)
            }
            ParseError::Nodes(msg) => write!(f, "{}: {}", lang::NODES_ERROR_PREFIX, msg),
            ParseError::Signal(msg) => write!(f, "{}: {}", lang::SIGNAL_ERROR_PREFIX, msg),
        }
    }
}
