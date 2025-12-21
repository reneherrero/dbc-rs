use super::ValueDescriptionsMap;
use crate::{Error, ExtendedMultiplexing, Message, Nodes, Result, VECTOR_XXX};

/// Validation functions for DBC structures
pub(crate) struct Validate;

impl Validate {
    /// Validates DBC structures including messages, value descriptions, and extended multiplexing.
    pub fn validate(
        nodes: &Nodes,
        messages: &[Message],
        value_descriptions: Option<&ValueDescriptionsMap>,
        extended_multiplexing: Option<&[ExtendedMultiplexing]>,
    ) -> Result<()> {
        Self::validate_common(nodes, messages)?;

        // Validate value descriptions if provided
        if let Some(value_descriptions) = value_descriptions {
            Self::validate_value_descriptions(messages, value_descriptions)?;
        }

        // Validate extended multiplexing if provided
        if let Some(ext_mux_entries) = extended_multiplexing {
            Self::validate_extended_multiplexing(messages, ext_mux_entries)?;
        }

        Ok(())
    }

    // Common validation logic
    fn validate_common(nodes: &Nodes, messages: &[Message]) -> Result<()> {
        // Check for duplicate message IDs
        // Compare using the internal ID representation (with extended flag)
        // Standard and extended IDs with the same base value are NOT duplicates
        for (i, msg1) in messages.iter().enumerate() {
            for msg2 in messages.iter().skip(i + 1) {
                if msg1.id_with_flag() == msg2.id_with_flag() {
                    return Err(Error::Validation(Error::DUPLICATE_MESSAGE_ID));
                }
            }
        }

        // Validate that all message senders are in the nodes list
        // Skip validation if nodes list is empty (empty nodes allowed per DBC spec)
        // Per DBC spec Section 8.4: Vector__XXX means "no sender / unknown sender"
        // and doesn't need to be in the nodes list
        if !nodes.is_empty() {
            for msg in messages {
                let sender = msg.sender();
                if sender != VECTOR_XXX && !nodes.contains(sender) {
                    return Err(Error::Validation(Error::SENDER_NOT_IN_NODES));
                }
            }
        }

        Ok(())
    }

    /// Validates value descriptions reference existing messages and signals
    fn validate_value_descriptions(
        messages: &[Message],
        value_descriptions: &ValueDescriptionsMap,
    ) -> Result<()> {
        for ((message_id_opt, signal_name), _) in value_descriptions.iter() {
            // Check if message exists (for message-specific value descriptions)
            // Use id_with_flag since message_id from DBC includes the extended flag
            if let Some(message_id) = message_id_opt {
                let message_exists = messages.iter().any(|msg| msg.id_with_flag() == message_id);
                if !message_exists {
                    return Err(Error::Validation(
                        Error::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND,
                    ));
                }

                // Check if signal exists in the message
                let signal_exists = messages.iter().any(|msg| {
                    msg.id_with_flag() == message_id && msg.signals().find(signal_name).is_some()
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

        Ok(())
    }

    /// Validates extended multiplexing entries against the messages
    fn validate_extended_multiplexing(
        messages: &[Message],
        ext_mux_entries: &[ExtendedMultiplexing],
    ) -> Result<()> {
        for ext_mux in ext_mux_entries {
            let message_id = ext_mux.message_id();
            let signal_name = ext_mux.signal_name();
            let multiplexer_switch = ext_mux.multiplexer_switch();

            // Find the message (use id_with_flag since message_id includes the extended flag)
            let message = messages
                .iter()
                .find(|msg| msg.id_with_flag() == message_id)
                .ok_or(Error::Validation(Error::EXT_MUX_MESSAGE_NOT_FOUND))?;

            // Check that the signal exists in the message
            if message.signals().find(signal_name).is_none() {
                return Err(Error::Validation(Error::EXT_MUX_SIGNAL_NOT_FOUND));
            }

            // Check that the multiplexer switch exists in the message
            if message.signals().find(multiplexer_switch).is_none() {
                return Err(Error::Validation(Error::EXT_MUX_SWITCH_NOT_FOUND));
            }

            // Validate value ranges (min <= max)
            for (min, max) in ext_mux.value_ranges() {
                if min > max {
                    return Err(Error::Validation(Error::EXT_MUX_INVALID_RANGE));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    /// Test that extended IDs (29-bit) with the 0x80000000 flag are handled correctly
    /// in duplicate detection. Standard and extended IDs with the same base value
    /// should NOT be considered duplicates.
    #[test]
    fn test_validate_standard_and_extended_same_base_id_not_duplicate() {
        // Standard ID 256 and Extended ID 256 (0x80000100) should be allowed
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 StandardMsg : 8 ECM
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] "" Vector__XXX

BO_ 2147483904 ExtendedMsg : 8 ECM
 SG_ Signal2 : 0|8@1+ (1,0) [0|255] "" Vector__XXX
"#,
        );
        // 2147483904 = 0x80000100 = extended ID 256
        assert!(
            dbc.is_ok(),
            "Standard and extended IDs with same base should not be duplicates"
        );

        let dbc = dbc.unwrap();
        assert_eq!(dbc.messages().len(), 2);

        // Verify the IDs are correct
        let std_msg = dbc.messages().find("StandardMsg").unwrap();
        let ext_msg = dbc.messages().find("ExtendedMsg").unwrap();

        assert_eq!(std_msg.id(), 256);
        assert!(!std_msg.is_extended());

        assert_eq!(ext_msg.id(), 256); // Raw ID without flag
        assert!(ext_msg.is_extended());
    }

    /// Test that actual duplicate extended IDs are detected
    #[test]
    fn test_validate_duplicate_extended_ids_rejected() {
        let result = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 2147483904 ExtendedMsg1 : 8 ECM
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] "" Vector__XXX

BO_ 2147483904 ExtendedMsg2 : 8 ECM
 SG_ Signal2 : 0|8@1+ (1,0) [0|255] "" Vector__XXX
"#,
        );
        // Both messages have extended ID 256 (0x80000100) - should fail
        assert!(result.is_err());
    }

    /// Test that value descriptions with extended message IDs work correctly
    #[test]
    fn test_validate_value_descriptions_extended_id() {
        // Extended ID: 2564485397 = 0x98DAF115 = 0x80000000 | 0x18DAF115
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_:

BO_ 2564485397 OBD2 : 8 Vector__XXX
 SG_ Mode : 0|8@1+ (1,0) [0|255] "" Vector__XXX

VAL_ 2564485397 Mode 1 "CurrentData" 2 "FreezeFrame" 3 "StoredDTCs" ;
"#,
        );

        assert!(
            dbc.is_ok(),
            "Value descriptions with extended ID should parse: {:?}",
            dbc.err()
        );
        let dbc = dbc.unwrap();

        // Verify the message
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.id(), 0x18DAF115); // Raw 29-bit ID
        assert!(msg.is_extended());
    }

    /// Test that extended multiplexing with extended message IDs work correctly
    #[test]
    fn test_validate_extended_multiplexing_extended_id() {
        // This is similar to the 29-bit OBD2 DBC file structure
        // Extended ID: 2564485397 = 0x98DAF115 = 0x80000000 | 0x18DAF115
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_:

BO_ 2564485397 OBD2 : 8 Vector__XXX
 SG_ Service M : 0|8@1+ (1,0) [0|255] "" Vector__XXX
 SG_ PID m1 : 8|8@1+ (1,0) [0|255] "" Vector__XXX
 SG_ Data m1 : 16|16@1+ (1,0) [0|65535] "" Vector__XXX

SG_MUL_VAL_ 2564485397 PID Service 1-1;
SG_MUL_VAL_ 2564485397 Data PID 12-12;
"#,
        );

        assert!(
            dbc.is_ok(),
            "Extended multiplexing with extended ID should parse: {:?}",
            dbc.err()
        );
        let dbc = dbc.unwrap();

        // Verify the message
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.id(), 0x18DAF115); // Raw 29-bit ID
        assert!(msg.is_extended());
        assert_eq!(msg.signals().len(), 3);
    }

    /// Test parsing the real 29-bit OBD2 DBC file pattern
    #[test]
    fn test_validate_real_obd2_extended_id_pattern() {
        // This mimics the structure of the 29-bit-OBD2-v4.0.dbc file
        let dbc = Dbc::parse(
            r#"VERSION ""

NS_ :

BS_:

BU_:

BO_ 2564485397 OBD2: 8 Vector__XXX
 SG_ S M : 15|8@0+ (1,0) [0|15] "" Vector__XXX
 SG_ S01PID m65M : 23|8@0+ (1,0) [0|255] "" Vector__XXX
 SG_ S01PID0D_VehicleSpeed m13 : 31|8@0+ (1,0) [0|255] "km/h" Vector__XXX

SG_MUL_VAL_ 2564485397 S01PID S 65-65;
SG_MUL_VAL_ 2564485397 S01PID0D_VehicleSpeed S01PID 13-13;
"#,
        );

        assert!(
            dbc.is_ok(),
            "Real OBD2 pattern should parse: {:?}",
            dbc.err()
        );
        let dbc = dbc.unwrap();

        let msg = dbc.messages().iter().next().unwrap();
        // 2564485397 = 0x98DAF115, raw ID = 0x18DAF115 = 417001749
        assert_eq!(msg.id(), 417001749);
        assert!(msg.is_extended());
    }
}
