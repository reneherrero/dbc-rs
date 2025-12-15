use crate::{Error, Parser, Result, Signal};

use super::Message;

impl Message {
    pub(crate) fn parse(parser: &mut Parser, signals: &[Signal]) -> Result<Self> {
        // Message parsing must always start with "BO_" keyword
        parser
            .expect(crate::BO_.as_bytes())
            .map_err(|_| Error::Expected("Expected BO_ keyword"))?;

        // Skip whitespace
        let _ = parser.skip_whitespace();

        // Parse message ID
        let id = parser
            .parse_u32_with_error(|| Error::Message(crate::error::Error::MESSAGE_INVALID_ID))?;

        // Skip whitespace
        parser.skip_whitespace().map_err(|_| Error::Expected("Expected whitespace"))?;

        // Parse message name (identifier)
        let name = parser.parse_identifier_with_error(|| {
            Error::Message(crate::error::Error::MESSAGE_NAME_EMPTY)
        })?;

        // Skip whitespace (optional before colon)
        let _ = parser.skip_whitespace();

        // Expect colon
        parser.expect_with_msg(b":", "Expected colon")?;

        // Skip whitespace after colon
        let _ = parser.skip_whitespace();

        // Parse DLC
        let dlc = parser
            .parse_u8_with_error(|| Error::Message(crate::error::Error::MESSAGE_INVALID_DLC))?;

        // Skip whitespace (required)
        parser.skip_whitespace().map_err(|_| Error::Expected("Expected whitespace"))?;

        // Parse sender (identifier, until end of line or whitespace)
        let sender = parser.parse_identifier_with_error(|| {
            Error::Message(crate::error::Error::MESSAGE_SENDER_EMPTY)
        })?;

        // Check for extra content after sender (invalid format)
        parser.skip_newlines_and_spaces();
        if !parser.is_empty() {
            return Err(Error::Expected("Unexpected content after message sender"));
        }

        // Validate before construction
        Self::validate_internal(id, name, dlc, sender, signals).map_err(|e| {
            crate::error::map_val_error(e, crate::error::Error::Message, || {
                crate::error::Error::Message(crate::error::Error::MESSAGE_ERROR_PREFIX)
            })
        })?;
        // Construct directly (validation already done)
        Ok(Self::new_from_signals(id, name, dlc, sender, signals))
    }
}
