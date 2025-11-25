#![allow(dead_code)] // Constants are conditionally used based on language feature selection

// ============================================================================
// Category labels
// ============================================================================

pub const INVALID_DATA_CATEGORY: &str = "Data Error";
pub const SIGNAL_ERROR_CATEGORY: &str = "Signal Error";
pub const MESSAGE_ERROR_CATEGORY: &str = "Message Error";
pub const DBC_ERROR_CATEGORY: &str = "DBC Error";
pub const VERSION_ERROR_CATEGORY: &str = "Version Error";
pub const NODES_ERROR_CATEGORY: &str = "Nodes Error";

// ============================================================================
// Version-related error messages
// ============================================================================

pub const VERSION_EMPTY: &str = "Empty version string";
pub const VERSION_INVALID: &str = "Invalid version string";
pub const VERSION_MAJOR_REQUIRED: &str = "major version is required";
pub const VERSION_PATCH_REQUIRES_MINOR: &str = "Patch version requires minor version";

// ============================================================================
// DBC file-related error messages
// ============================================================================

pub const DBC_EMPTY_FILE: &str = "Empty DBC file";
pub const DBC_VERSION_REQUIRED: &str = "version is required";
pub const DBC_NODES_REQUIRED: &str = "nodes is required";
pub const DBC_NODES_NOT_DEFINED: &str = "Nodes (BU_) are not defined";
pub const NODES_DUPLICATE_NAME: &str = "Duplicate node name";

// ============================================================================
// Message-related error messages
// ============================================================================

pub const MESSAGE_NAME_EMPTY: &str = "Message name cannot be empty";
pub const MESSAGE_ID_REQUIRED: &str = "id is required";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc is required";
pub const MESSAGE_SENDER_EMPTY: &str = "Message sender cannot be empty";
pub const MESSAGE_DLC_TOO_SMALL: &str = "Message DLC must be at least 1 byte";
pub const MESSAGE_DLC_TOO_LARGE: &str = "Message DLC cannot exceed 8 bytes";
pub const MESSAGE_INVALID_FORMAT: &str = "Invalid message format";
pub const MESSAGE_INVALID_ID: &str = "Invalid message ID";
pub const MESSAGE_INVALID_DLC: &str = "Invalid DLC";
pub const MESSAGE_ID_OUT_OF_RANGE: &str = "Message ID out of valid range";

// ============================================================================
// Signal-related error messages
// ============================================================================

pub const SIGNAL_NAME_EMPTY: &str = "Signal name cannot be empty";
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit is required";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length is required";
pub const SIGNAL_LENGTH_TOO_SMALL: &str = "Signal length must be at least 1 bit";
pub const SIGNAL_LENGTH_TOO_LARGE: &str = "Signal length cannot exceed 64 bits";
pub const SIGNAL_OVERLAP: &str = "Signals overlap within message";

// ============================================================================
// Signal parsing error messages
// ============================================================================

pub const SIGNAL_PARSE_EXPECTED_SG: &str = "Expected 'SG_' at line start";
pub const SIGNAL_PARSE_MISSING_COLON: &str = "Missing ':' in signal definition";
pub const SIGNAL_PARSE_MISSING_POSITION: &str = "Missing position spec";
pub const SIGNAL_PARSE_MISSING_REST: &str = "Missing rest after position spec";
pub const SIGNAL_PARSE_EXPECTED_AT: &str = "Expected '@' in startbit|length@...";
pub const SIGNAL_PARSE_EXPECTED_PIPE: &str = "Expected '|' in startbit|length";
pub const SIGNAL_PARSE_INVALID_START_BIT: &str = "Invalid start_bit";
pub const SIGNAL_PARSE_INVALID_LENGTH: &str = "Invalid length";
pub const SIGNAL_PARSE_MISSING_BYTE_ORDER: &str = "Missing byte order";
pub const SIGNAL_PARSE_MISSING_SIGN: &str = "Missing sign";
pub const SIGNAL_PARSE_MISSING_CLOSING_PAREN: &str = "Missing closing ')' for factor,offset";
pub const SIGNAL_PARSE_MISSING_OPENING_PAREN: &str = "Missing opening '(' for factor,offset";
pub const SIGNAL_PARSE_MISSING_COMMA: &str = "Missing ',' in factor,offset";
pub const SIGNAL_PARSE_INVALID_FACTOR: &str = "Invalid factor";
pub const SIGNAL_PARSE_INVALID_OFFSET: &str = "Invalid offset";
pub const SIGNAL_PARSE_MISSING_CLOSING_BRACKET: &str = "Missing closing ']' for min|max";
pub const SIGNAL_PARSE_MISSING_OPENING_BRACKET: &str = "Missing opening '[' for min|max";
pub const SIGNAL_PARSE_MISSING_PIPE_IN_RANGE: &str = "Missing '|' in min|max";
pub const SIGNAL_PARSE_INVALID_MIN: &str = "Invalid min";
pub const SIGNAL_PARSE_INVALID_MAX: &str = "Invalid max";
pub const SIGNAL_PARSE_EXPECTED_UNIT_QUOTE: &str = "Expected beginning of 'unit' string '\"'";

// ============================================================================
// Formatted error message templates
// ============================================================================

pub const FORMAT_DUPLICATE_MESSAGE_ID: &str = "Duplicate message ID: {} (messages '{}' and '{}')";
pub const FORMAT_DUPLICATE_NODE_NAME: &str = "Duplicate node name: '{}'";
pub const FORMAT_SENDER_NOT_IN_NODES: &str =
    "Message '{}' has sender '{}' which is not in the nodes list";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_MESSAGE: &str = "Signal '{}' extends beyond message boundary: start_bit {} + length {} = {} > {} (DLC {} bytes)";
pub const FORMAT_SIGNAL_EXTENDS_BEYOND_CAN: &str =
    "Signal extends beyond CAN message boundary: start_bit {} + length {} = {} > 64";
pub const FORMAT_INVALID_RANGE: &str = "Invalid range: min {} > max {}";
pub const FORMAT_UNKNOWN_BYTE_ORDER: &str = "Unknown byte order '{}'";
pub const FORMAT_UNKNOWN_SIGN: &str = "Unknown sign '{}'";
pub const FORMAT_PARSE_NUMBER_FAILED: &str = "Failed to parse number: {}";
pub const FORMAT_INVALID_UTF8: &str = "Invalid UTF-8: {}";
pub const FORMAT_READ_FAILED: &str = "Failed to read: {}";
pub const FORMAT_MESSAGE_ID_OUT_OF_RANGE: &str = "Message ID {} is out of valid range (standard 11-bit: 0-2047, extended 29-bit: 2048-536870911)";
pub const FORMAT_SIGNAL_OVERLAP: &str = "Signals '{}' and '{}' overlap in message '{}'";
