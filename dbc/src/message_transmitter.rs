//! Message Transmitter definition (BO_TX_BU_)
//!
//! Defines multiple transmitters for a message (higher-layer protocols).
//! Note: This is not for CAN layer-2 communication, but for higher-layer protocol descriptions.

/// Message Transmitter definition (BO_TX_BU_)
///
/// Defines multiple transmitters for a message (higher-layer protocols).
/// Note: This is not for CAN layer-2 communication, but for higher-layer protocol descriptions.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct MessageTransmitter {
    message_id: u32,
    transmitters: std::vec::Vec<std::string::String>,
}

#[cfg(feature = "std")]
impl MessageTransmitter {
    /// Create a new MessageTransmitter
    pub(crate) fn new(message_id: u32, transmitters: std::vec::Vec<std::string::String>) -> Self {
        Self {
            message_id,
            transmitters,
        }
    }

    /// Get the message ID
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Get the list of transmitter node names
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn transmitters(&self) -> &[std::string::String] {
        &self.transmitters
    }
}
