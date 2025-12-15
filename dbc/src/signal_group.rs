//! Signal Group definition (SIG_GROUP_)
//!
//! Represents a group of related signals within a message.

/// Signal Group definition (SIG_GROUP_)
///
/// Represents a group of related signals within a message.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct SignalGroup {
    message_id: u32,
    signal_group_name: std::string::String,
    repetitions: u32,
    signal_names: std::vec::Vec<std::string::String>,
}

#[cfg(feature = "std")]
impl SignalGroup {
    /// Create a new SignalGroup
    pub(crate) fn new(
        message_id: u32,
        signal_group_name: std::string::String,
        repetitions: u32,
        signal_names: std::vec::Vec<std::string::String>,
    ) -> Self {
        Self {
            message_id,
            signal_group_name,
            repetitions,
            signal_names,
        }
    }

    /// Get the message ID
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Get the signal group name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn signal_group_name(&self) -> &str {
        &self.signal_group_name
    }

    /// Get the repetitions value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn repetitions(&self) -> u32 {
        self.repetitions
    }

    /// Get the list of signal names
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn signal_names(&self) -> &[std::string::String] {
        &self.signal_names
    }
}
