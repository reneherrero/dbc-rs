use super::{Message, Signals};
use crate::{Error, MAX_NAME_SIZE, Parser, Result, Signal, compat};

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
        Message::validate_internal(id, name, dlc, sender, signals).map_err(|e| {
            crate::error::map_val_error(e, crate::error::Error::Message, || {
                crate::error::Error::Message(crate::error::Error::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Convert to owned types (validation already done, so unwrap is safe)
        let name_str: compat::String<{ MAX_NAME_SIZE }> = compat::validate_name(name)?;
        let sender_str: compat::String<{ MAX_NAME_SIZE }> = compat::validate_name(sender)?;
        let signals_collection = Signals::from_slice(signals);

        Ok(Message::new(
            id,
            name_str,
            dlc,
            sender_str,
            signals_collection,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Signal};

    #[test]
    fn test_message_parse_valid() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 0);
    }

    #[test]
    fn test_message_parse_invalid_id() {
        let data = b"BO_ invalid EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let result = Message::parse(&mut parser, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(_) => {
                // Expected
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_message_parse_empty_name() {
        let data = b"BO_ 256  : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let result = Message::parse(&mut parser, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(_) => {
                // Expected
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_message_parse_invalid_dlc() {
        let data = b"BO_ 256 EngineData : invalid ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let result = Message::parse(&mut parser, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(_) => {
                // Expected
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_message_parse_empty_sender() {
        let data = b"BO_ 256 EngineData : 8 ";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let result = Message::parse(&mut parser, signals);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(_) => {
                // Expected
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_message_parse_with_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Create test signals
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 2);
    }
}
