// ============================================================================
// no_std error messages (used in Error::Validation, Error::Decoding, and other errors)
// ============================================================================

// Error messages
pub const EXPECTED_WHITESPACE: &str = "Expected whitespace";
pub const EXPECTED_PATTERN: &str = "Expected pattern";
pub const EXPECTED_KEYWORD: &str = "Expected keyword";
pub const EXPECTED_NUMBER: &str = "Expected number";
pub const EXPECTED_IDENTIFIER: &str = "Expected identifier";
pub const INVALID_UTF8: &str = "Invalid UTF-8";
pub const INVALID_NUMBER_FORMAT: &str = "Invalid number format";
pub const PARSE_NUMBER_FAILED: &str = "Failed to parse number";
pub const UNEXPECTED_EOF: &str = "Unexpected end of input";
pub const INVALID_CHARACTER: &str = "Invalid character";
pub const STRING_LENGTH_EXCEEDS_MAX: &str = "String length exceeds maximum";
pub const DECODING_ERROR_PREFIX: &str = "Decoding Error";
pub const VALIDATION_ERROR_PREFIX: &str = "Validation Error";
pub const VERSION_ERROR_PREFIX: &str = "Version error";
pub const MESSAGE_ERROR_PREFIX: &str = "Message error";
pub const RECEIVERS_ERROR_PREFIX: &str = "Receivers error";
pub const NODES_ERROR_PREFIX: &str = "Nodes error";
pub const SIGNAL_ERROR_PREFIX: &str = "Signal error";

// Signal error messages
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "Invalid start_bit";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "Invalid length";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "Invalid factor";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "Invalid offset";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "Invalid min";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "Invalid max";
pub const SIGNAL_PARSE_UNIT_TOO_LONG: &str = "Unit string exceeds maximum length of 256 characters";
pub const MAX_NAME_SIZE_EXCEEDED: &str = "Exceeded the maximum name length";

// Validation and decoding errors (available in no_std)
pub const NODES_DUPLICATE_NAME: &str = "Duplicate node name";
pub const NODES_TOO_MANY: &str = "Too many nodes: maximum allowed is 256";
pub const DUPLICATE_MESSAGE_ID: &str = "Duplicate message ID";
pub const SENDER_NOT_IN_NODES: &str = "Message sender not in nodes list";
pub const SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "Signal extends beyond message boundary";
pub const INVALID_RANGE: &str = "Invalid range: min > max";
pub const MESSAGE_TOO_MANY_SIGNALS: &str = "Too many signals: maximum allowed is 256 per message";
pub const SIGNAL_RECEIVERS_TOO_MANY: &str =
    "Too many receiver nodes: maximum allowed is 255 per signal";
pub const EXTENDED_MULTIPLEXING_TOO_MANY: &str =
    "Too many extended multiplexing entries: maximum allowed is 512 per DBC file";
pub const SIGNAL_NAME_EMPTY: &str = "Signal name cannot be empty";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "Signal length must be at least 1 bit";
pub const SIGNAL_LENGTH_TOO_LARGE: &str = "Signal length cannot exceed 512 bits (CAN FD maximum)";
pub const SIGNAL_OVERLAP: &str = "Signals overlap within message";
pub const SIGNAL_EXTENDS_BEYOND_DATA: &str = "Signal extends beyond message data";
pub const MESSAGE_NAME_EMPTY: &str = "Message name cannot be empty";
pub const MESSAGE_SENDER_EMPTY: &str = "Message sender cannot be empty";
pub const MESSAGE_DLC_TOO_SMALL: &str = "Message DLC must be at least 1 byte";
pub const MESSAGE_DLC_TOO_LARGE: &str = "Message DLC cannot exceed 64 bytes (CAN FD maximum)";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "Message ID out of valid range";
pub const MESSAGE_INVALID_ID: &str = "Invalid message ID";
pub const MESSAGE_INVALID_DLC: &str = "Invalid DLC";
pub const MESSAGE_NOT_FOUND: &str = "Message ID not found in database";
pub const PAYLOAD_LENGTH_MISMATCH: &str = "Payload length does not match message DLC";
pub const MULTIPLEXER_SWITCH_NEGATIVE: &str = "Multiplexer switch value cannot be negative";
