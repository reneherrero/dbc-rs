// ============================================================================
// std error messages (only used in std code)
// ============================================================================

// Category labels (used in Display impl which requires std)
pub const INVALID_DATA_CATEGORY: &str = "Data Error";
pub const SIGNAL_ERROR_CATEGORY: &str = "Signal Error";
pub const MESSAGE_ERROR_CATEGORY: &str = "Message Error";
pub const DBC_ERROR_CATEGORY: &str = "DBC Error";
pub const VERSION_ERROR_CATEGORY: &str = "Version Error";
pub const NODES_ERROR_CATEGORY: &str = "Nodes Error";

// Version-related error messages
pub const VERSION_EMPTY: &str = "Empty version string";

// DBC file-related error messages
pub const INVALID_UTF8: &str = "Invalid UTF-8";
pub const VALUE_DESCRIPTION_MESSAGE_NOT_FOUND: &str =
    "Value description references non-existent message";
pub const VALUE_DESCRIPTION_SIGNAL_NOT_FOUND: &str =
    "Value description references non-existent signal";

// Message-related error messages (only used in std contexts)
pub const MESSAGE_ID_REQUIRED: &str = "id is required";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc is required";

// Signal-related error messages (only used in std contexts)
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit is required";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length is required";

pub const RECEIVERS_DUPLICATE_NAME: &str = "Duplicate Receiver name";
