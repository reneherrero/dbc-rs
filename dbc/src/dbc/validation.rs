#[cfg(feature = "std")]
use crate::ValueDescriptionsList;
use crate::{Error, Message, Nodes, Result};

/// Validation functions for DBC structures
pub(crate) struct Validation;

impl Validation {
    // Validate function for std feature (with value_descriptions)
    #[cfg(feature = "std")]
    pub(crate) fn validate(
        nodes: &Nodes,
        messages: &[Message],
        value_descriptions: Option<&ValueDescriptionsList>,
    ) -> Result<()> {
        Self::validate_common(nodes, messages)?;

        // Validate value descriptions if provided
        if let Some(value_descriptions) = value_descriptions {
            // Validate that all value descriptions reference existing messages and signals
            for ((message_id_opt, signal_name), _) in value_descriptions.iter() {
                // Check if message exists (for message-specific value descriptions)
                if let Some(message_id) = message_id_opt {
                    let message_exists = messages.iter().any(|msg| msg.id() == message_id);
                    if !message_exists {
                        return Err(Error::Validation(
                            Error::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND,
                        ));
                    }

                    // Check if signal exists in the message
                    let signal_exists = messages.iter().any(|msg| {
                        msg.id() == message_id && msg.signals().find(signal_name).is_some()
                    });
                    if !signal_exists {
                        return Err(Error::Validation(Error::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
                    }
                } else {
                    // For global value descriptions (message_id is None), check if signal exists in any message
                    let signal_exists =
                        messages.iter().any(|msg| msg.signals().find(signal_name).is_some());
                    if !signal_exists {
                        return Err(Error::Validation(Error::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
                    }
                }
            }
        }

        Ok(())
    }

    // Validate function for no_std mode (without value_descriptions)
    #[cfg(not(feature = "std"))]
    pub(crate) fn validate(nodes: &Nodes, messages: &[Message]) -> Result<()> {
        Self::validate_common(nodes, messages)
    }

    // Common validation logic shared by both versions
    fn validate_common(nodes: &Nodes, messages: &[Message]) -> Result<()> {
        // Check for duplicate message IDs
        for (i, msg1) in messages.iter().enumerate() {
            for msg2 in messages.iter().skip(i + 1) {
                if msg1.id() == msg2.id() {
                    return Err(Error::Validation(Error::DUPLICATE_MESSAGE_ID));
                }
            }
        }

        // Validate that all message senders are in the nodes list
        // Skip validation if nodes list is empty (empty nodes allowed per DBC spec)
        if !nodes.is_empty() {
            for msg in messages {
                if !nodes.contains(msg.sender()) {
                    return Err(Error::Validation(Error::SENDER_NOT_IN_NODES));
                }
            }
        }

        Ok(())
    }
}
