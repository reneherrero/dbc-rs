use core::fmt;

use super::Error;

impl Error {
    // ============================================================================
    // Constructor helpers (for creating errors without line info)
    // ============================================================================

    /// Create an UnexpectedEof error without line info.
    #[inline]
    pub fn unexpected_eof() -> Self {
        Error::UnexpectedEof { line: None }
    }

    /// Create an UnexpectedEof error with line info.
    #[inline]
    pub fn unexpected_eof_at(line: usize) -> Self {
        Error::UnexpectedEof { line: Some(line) }
    }

    /// Create an Expected error without line info.
    #[inline]
    pub fn expected(msg: &'static str) -> Self {
        Error::Expected { msg, line: None }
    }

    /// Create an Expected error with line info.
    #[inline]
    pub fn expected_at(msg: &'static str, line: usize) -> Self {
        Error::Expected {
            msg,
            line: Some(line),
        }
    }

    /// Create an InvalidChar error without line info.
    #[inline]
    pub fn invalid_char(char: char) -> Self {
        Error::InvalidChar { char, line: None }
    }

    /// Create an InvalidChar error with line info.
    #[inline]
    pub fn invalid_char_at(char: char, line: usize) -> Self {
        Error::InvalidChar {
            char,
            line: Some(line),
        }
    }

    /// Create a MaxStrLength error without line info.
    #[inline]
    pub fn max_str_length(max: usize) -> Self {
        Error::MaxStrLength { max, line: None }
    }

    /// Create a MaxStrLength error with line info.
    #[inline]
    pub fn max_str_length_at(max: usize, line: usize) -> Self {
        Error::MaxStrLength {
            max,
            line: Some(line),
        }
    }

    /// Create a Version error without line info.
    #[inline]
    pub fn version(msg: &'static str) -> Self {
        Error::Version { msg, line: None }
    }

    /// Create a Version error with line info.
    #[inline]
    pub fn version_at(msg: &'static str, line: usize) -> Self {
        Error::Version {
            msg,
            line: Some(line),
        }
    }

    /// Create a Message error without line info.
    #[inline]
    pub fn message(msg: &'static str) -> Self {
        Error::Message { msg, line: None }
    }

    /// Create a Message error with line info.
    #[inline]
    pub fn message_at(msg: &'static str, line: usize) -> Self {
        Error::Message {
            msg,
            line: Some(line),
        }
    }

    /// Create a Receivers error without line info.
    #[inline]
    pub fn receivers(msg: &'static str) -> Self {
        Error::Receivers { msg, line: None }
    }

    /// Create a Receivers error with line info.
    #[inline]
    pub fn receivers_at(msg: &'static str, line: usize) -> Self {
        Error::Receivers {
            msg,
            line: Some(line),
        }
    }

    /// Create a Nodes error without line info.
    #[inline]
    pub fn nodes(msg: &'static str) -> Self {
        Error::Nodes { msg, line: None }
    }

    /// Create a Nodes error with line info.
    #[inline]
    pub fn nodes_at(msg: &'static str, line: usize) -> Self {
        Error::Nodes {
            msg,
            line: Some(line),
        }
    }

    /// Create a Signal error without line info.
    #[inline]
    pub fn signal(msg: &'static str) -> Self {
        Error::Signal { msg, line: None }
    }

    /// Create a Signal error with line info.
    #[inline]
    pub fn signal_at(msg: &'static str, line: usize) -> Self {
        Error::Signal {
            msg,
            line: Some(line),
        }
    }

    // ============================================================================
    // Line number helpers
    // ============================================================================

    /// Get the line number associated with this error, if any.
    #[inline]
    pub fn line(&self) -> Option<usize> {
        match self {
            Error::UnexpectedEof { line } => *line,
            Error::Expected { line, .. } => *line,
            Error::InvalidChar { line, .. } => *line,
            Error::MaxStrLength { line, .. } => *line,
            Error::Version { line, .. } => *line,
            Error::Message { line, .. } => *line,
            Error::Receivers { line, .. } => *line,
            Error::Nodes { line, .. } => *line,
            Error::Signal { line, .. } => *line,
            Error::Decoding(_) | Error::Encoding(_) | Error::Validation(_) => None,
        }
    }

    /// Add line number information to an error.
    /// If the error already has line info, it is preserved.
    #[inline]
    pub fn with_line(self, line: usize) -> Self {
        match self {
            Error::UnexpectedEof { line: None } => Error::UnexpectedEof { line: Some(line) },
            Error::Expected { msg, line: None } => Error::Expected {
                msg,
                line: Some(line),
            },
            Error::InvalidChar { char, line: None } => Error::InvalidChar {
                char,
                line: Some(line),
            },
            Error::MaxStrLength { max, line: None } => Error::MaxStrLength {
                max,
                line: Some(line),
            },
            Error::Version { msg, line: None } => Error::Version {
                msg,
                line: Some(line),
            },
            Error::Message { msg, line: None } => Error::Message {
                msg,
                line: Some(line),
            },
            Error::Receivers { msg, line: None } => Error::Receivers {
                msg,
                line: Some(line),
            },
            Error::Nodes { msg, line: None } => Error::Nodes {
                msg,
                line: Some(line),
            },
            Error::Signal { msg, line: None } => Error::Signal {
                msg,
                line: Some(line),
            },
            // Already has line info or doesn't support it - return unchanged
            other => other,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEof { line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}", line, Error::UNEXPECTED_EOF)
                } else {
                    write!(f, "{}", Error::UNEXPECTED_EOF)
                }
            }
            Error::Expected { msg, line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}", line, msg)
                } else {
                    write!(f, "{}", msg)
                }
            }
            Error::InvalidChar { char, line } => {
                if let Some(line) = line {
                    write!(
                        f,
                        "line {}: {}: {}",
                        line,
                        Error::INVALID_CHARACTER,
                        char.escape_debug()
                    )
                } else {
                    write!(f, "{}: {}", Error::INVALID_CHARACTER, char.escape_debug())
                }
            }
            Error::MaxStrLength { max, line } => {
                if let Some(line) = line {
                    write!(
                        f,
                        "line {}: {}: {}",
                        line,
                        Error::STRING_LENGTH_EXCEEDS_MAX,
                        max
                    )
                } else {
                    write!(f, "{}: {}", Error::STRING_LENGTH_EXCEEDS_MAX, max)
                }
            }
            Error::Version { msg, line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}: {}", line, Error::VERSION_ERROR_PREFIX, msg)
                } else {
                    write!(f, "{}: {}", Error::VERSION_ERROR_PREFIX, msg)
                }
            }
            Error::Message { msg, line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}: {}", line, Error::MESSAGE_ERROR_PREFIX, msg)
                } else {
                    write!(f, "{}: {}", Error::MESSAGE_ERROR_PREFIX, msg)
                }
            }
            Error::Receivers { msg, line } => {
                if let Some(line) = line {
                    write!(
                        f,
                        "line {}: {}: {}",
                        line,
                        Error::RECEIVERS_ERROR_PREFIX,
                        msg
                    )
                } else {
                    write!(f, "{}: {}", Error::RECEIVERS_ERROR_PREFIX, msg)
                }
            }
            Error::Nodes { msg, line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}: {}", line, Error::NODES_ERROR_PREFIX, msg)
                } else {
                    write!(f, "{}: {}", Error::NODES_ERROR_PREFIX, msg)
                }
            }
            Error::Signal { msg, line } => {
                if let Some(line) = line {
                    write!(f, "line {}: {}: {}", line, Error::SIGNAL_ERROR_PREFIX, msg)
                } else {
                    write!(f, "{}: {}", Error::SIGNAL_ERROR_PREFIX, msg)
                }
            }
            Error::Decoding(msg) => {
                write!(f, "{}: {}", Error::DECODING_ERROR_PREFIX, msg)
            }
            Error::Encoding(msg) => {
                write!(f, "{}: {}", Error::ENCODING_ERROR_PREFIX, msg)
            }
            Error::Validation(msg) => {
                write!(f, "{}: {}", Error::VALIDATION_ERROR_PREFIX, msg)
            }
        }
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(_err: core::num::ParseIntError) -> Self {
        Error::expected(Error::PARSE_NUMBER_FAILED)
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
                Error::Expected { msg, line } => {
                    assert_eq!(msg, Error::PARSE_NUMBER_FAILED);
                    assert_eq!(line, None);
                }
                _ => panic!("Expected Error::Expected"),
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
            let error = Error::signal("Test signal error");
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

        #[test]
        fn test_error_with_line_number() {
            let error = Error::expected_at("Expected identifier", 42);
            let display = error.to_string();
            assert!(display.contains("line 42"));
            assert!(display.contains("Expected identifier"));
        }

        #[test]
        fn test_error_without_line_number() {
            let error = Error::expected("Expected identifier");
            let display = error.to_string();
            assert!(!display.contains("line"));
            assert!(display.contains("Expected identifier"));
        }

        #[test]
        fn test_with_line_adds_line_info() {
            let error = Error::expected("Expected whitespace");
            assert_eq!(error.line(), None);

            let error_with_line = error.with_line(10);
            assert_eq!(error_with_line.line(), Some(10));
        }

        #[test]
        fn test_with_line_preserves_existing_line() {
            let error = Error::expected_at("Expected whitespace", 5);
            let error_with_line = error.with_line(10);
            // Should preserve the original line (5), not overwrite with 10
            assert_eq!(error_with_line.line(), Some(5));
        }

        #[test]
        fn test_invalid_char_display() {
            let error = Error::invalid_char_at('\t', 15);
            let display = error.to_string();
            assert!(display.contains("line 15"));
            assert!(display.contains("\\t"));
        }

        #[test]
        fn test_max_str_length_display() {
            let error = Error::max_str_length_at(256, 20);
            let display = error.to_string();
            assert!(display.contains("line 20"));
            assert!(display.contains("256"));
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
