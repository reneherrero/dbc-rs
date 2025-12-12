// ============================================================================
// std error messages (only used in std code)
// ============================================================================

// DBC file-related error messages
pub const VALUE_DESCRIPTION_MESSAGE_NOT_FOUND: &str =
    "Value description references non-existent message";
pub const VALUE_DESCRIPTION_SIGNAL_NOT_FOUND: &str =
    "Value description references non-existent signal";
pub const VALUE_DESCRIPTIONS_TOO_MANY: &str = "Too many value descriptions: maximum allowed is 64";

// Message-related error messages (only used in std contexts)
pub const MESSAGE_ID_REQUIRED: &str = "id is required";
pub const MESSAGE_DLC_REQUIRED: &str = "dlc is required";

// Signal-related error messages (only used in std contexts)
pub const SIGNAL_START_BIT_REQUIRED: &str = "start_bit is required";
pub const SIGNAL_LENGTH_REQUIRED: &str = "length is required";

pub const RECEIVERS_DUPLICATE_NAME: &str = "Duplicate Receiver name";
