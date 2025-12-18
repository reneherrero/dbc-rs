mod helpers;
mod impls;
mod lang;

pub(crate) use helpers::{check_max_limit, map_val_error, map_val_error_with_line};

/// Error type for DBC operations.
///
/// This enum represents all possible errors that can occur when working with DBC files.
/// Parsing errors include line number information to help locate issues in the source file.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// Unexpected end of input encountered.
    UnexpectedEof { line: Option<usize> },
    /// Expected a specific token or value.
    Expected {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Invalid character encountered.
    InvalidChar { char: char, line: Option<usize> },
    /// String length exceeds the maximum allowed length.
    MaxStrLength { max: usize, line: Option<usize> },
    /// Version-related error.
    Version {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Message-related error.
    Message {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Receivers-related error.
    Receivers {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Nodes-related error.
    Nodes {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Signal-related error.
    Signal {
        msg: &'static str,
        line: Option<usize>,
    },
    /// Decoding-related error (runtime, no line info).
    Decoding(&'static str),
    /// Validation-related error (post-parse, no line info).
    Validation(&'static str),
}

impl Error {
    // ============================================================================
    // Error message constants
    // ============================================================================

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
    #[cfg(feature = "std")]
    pub const SIGNAL_BYTE_ORDER_REQUIRED: &'static str = lang::SIGNAL_BYTE_ORDER_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_UNSIGNED_REQUIRED: &'static str = lang::SIGNAL_UNSIGNED_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_FACTOR_REQUIRED: &'static str = lang::SIGNAL_FACTOR_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_OFFSET_REQUIRED: &'static str = lang::SIGNAL_OFFSET_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_MIN_REQUIRED: &'static str = lang::SIGNAL_MIN_REQUIRED;
    #[cfg(feature = "std")]
    pub const SIGNAL_MAX_REQUIRED: &'static str = lang::SIGNAL_MAX_REQUIRED;
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

    // Value description error constants (no_std)
    pub const VALUE_DESCRIPTION_MESSAGE_NOT_FOUND: &'static str =
        lang::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND;
    pub const VALUE_DESCRIPTION_SIGNAL_NOT_FOUND: &'static str =
        lang::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND;
    pub const VALUE_DESCRIPTIONS_TOO_MANY: &'static str = lang::VALUE_DESCRIPTIONS_TOO_MANY;

    // Extended multiplexing error constants (no_std)
    pub const EXT_MUX_MESSAGE_NOT_FOUND: &'static str = lang::EXT_MUX_MESSAGE_NOT_FOUND;
    pub const EXT_MUX_SIGNAL_NOT_FOUND: &'static str = lang::EXT_MUX_SIGNAL_NOT_FOUND;
    pub const EXT_MUX_SWITCH_NOT_FOUND: &'static str = lang::EXT_MUX_SWITCH_NOT_FOUND;
    pub const EXT_MUX_INVALID_RANGE: &'static str = lang::EXT_MUX_INVALID_RANGE;
}

/// Result type alias for operations that can return an `Error`.
pub type Result<T> = core::result::Result<T, Error>;
