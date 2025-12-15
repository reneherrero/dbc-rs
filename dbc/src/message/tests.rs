#![allow(clippy::float_cmp)]
use super::*;
use crate::{Error, Parser, Signal};

// Note: Builder tests have been moved to message_builder.rs
// This module only tests Message parsing and direct API usage

// Note: All test_message_new_* tests have been removed - they belong in message_builder.rs
// This module only tests Message parsing and direct API usage
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
    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
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

#[test]
fn test_message_signals_iterator() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    // Create test signals
    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();
    let signal2 = Signal::parse(
        &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
    )
    .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
    let mut signals_iter = message.signals().iter();
    assert_eq!(signals_iter.next().unwrap().name(), "RPM");
    assert_eq!(signals_iter.next().unwrap().name(), "Temp");
    assert!(signals_iter.next().is_none());
}

#[test]
fn test_message_signal_count() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signals: &[Signal] = &[];
    let message = Message::parse(&mut parser, signals).unwrap();
    assert_eq!(message.signals().len(), 0);

    // Create a new parser for the second parse since the first one consumed the input
    let data2 = b"BO_ 256 EngineData : 8 ECM";
    let mut parser2 = Parser::new(data2).unwrap();
    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();
    let message = Message::parse(&mut parser2, &[signal1]).unwrap();
    assert_eq!(message.signals().len(), 1);
}

#[test]
fn test_message_signal_at() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();
    let signal2 = Signal::parse(
        &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
    )
    .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
    assert_eq!(message.signals().at(0).unwrap().name(), "RPM");
    assert_eq!(message.signals().at(1).unwrap().name(), "Temp");
    assert!(message.signals().at(2).is_none());
}

#[test]
fn test_message_find_signal() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();
    let signal2 = Signal::parse(
        &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
    )
    .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
    assert_eq!(message.signals().find("RPM").unwrap().name(), "RPM");
    assert_eq!(message.signals().find("Temp").unwrap().name(), "Temp");
    assert!(message.signals().find("Nonexistent").is_none());
}

#[test]
fn test_message_multiple_signals_boundary_validation() {
    // Test that signals at message boundaries are validated correctly
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    // Create signals that exactly fill the message (8 bytes = 64 bits)
    // Signal 1: bits 0-15 (16 bits)
    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|16@0+ (1,0) [0|65535] \"\"").unwrap())
            .unwrap();
    // Signal 2: bits 16-31 (16 bits)
    let signal2 =
        Signal::parse(&mut Parser::new(b"SG_ Signal2 : 16|16@0+ (1,0) [0|65535] \"\"").unwrap())
            .unwrap();
    // Signal 3: bits 32-47 (16 bits)
    let signal3 =
        Signal::parse(&mut Parser::new(b"SG_ Signal3 : 32|16@0+ (1,0) [0|65535] \"\"").unwrap())
            .unwrap();
    // Signal 4: bits 48-63 (16 bits) - exactly at boundary
    let signal4 =
        Signal::parse(&mut Parser::new(b"SG_ Signal4 : 48|16@0+ (1,0) [0|65535] \"\"").unwrap())
            .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2, signal3, signal4]).unwrap();
    assert_eq!(message.signals().len(), 4);
}

#[test]
fn test_message_big_endian_bit_range_calculation() {
    // Test big-endian bit range calculation
    // BE bit 0 -> physical bit 7
    // BE bit 7 -> physical bit 0
    // BE bit 8 -> physical bit 15
    // BE bit 15 -> physical bit 8
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    // Signal starting at BE bit 0, length 8 -> should map to physical bits 0-7
    let signal =
        Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@1+ (1,0) [0|255] \"\"").unwrap())
            .unwrap();

    let message = Message::parse(&mut parser, &[signal]).unwrap();
    // The signal should be valid and fit within the message
    assert_eq!(message.signals().len(), 1);
}

#[test]
fn test_message_little_endian_bit_range_calculation() {
    // Test little-endian bit range calculation
    // LE bit N -> physical bit N (straightforward mapping)
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    // Signal starting at LE bit 0, length 8 -> should map to physical bits 0-7
    let signal =
        Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
            .unwrap();

    let message = Message::parse(&mut parser, &[signal]).unwrap();
    // The signal should be valid and fit within the message
    assert_eq!(message.signals().len(), 1);
}

// Note: Big-endian signal overlap and boundary tests have been moved to message_builder.rs

#[test]
fn test_message_signals_is_empty() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();
    let signals: &[Signal] = &[];
    let message = Message::parse(&mut parser, signals).unwrap();
    assert!(message.signals().is_empty());
    assert_eq!(message.signals().len(), 0);
}

#[test]
fn test_message_signals_at_out_of_bounds() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signal =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();

    let message = Message::parse(&mut parser, &[signal]).unwrap();

    // Valid index
    assert!(message.signals().at(0).is_some());
    assert_eq!(message.signals().at(0).unwrap().name(), "RPM");

    // Out of bounds
    assert!(message.signals().at(1).is_none());
    assert!(message.signals().at(100).is_none());
    assert!(message.signals().at(usize::MAX).is_none());
}

#[test]
fn test_message_signals_find_case_sensitive() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap())
            .unwrap();
    let signal2 = Signal::parse(
        &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
    )
    .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();

    // Exact match
    assert!(message.signals().find("RPM").is_some());
    assert_eq!(message.signals().find("RPM").unwrap().name(), "RPM");

    // Case sensitive - should not find
    assert!(message.signals().find("rpm").is_none());
    assert!(message.signals().find("Rpm").is_none());

    // Find second signal
    assert!(message.signals().find("Temp").is_some());
    assert_eq!(message.signals().find("Temp").unwrap().name(), "Temp");

    // Not found
    assert!(message.signals().find("Nonexistent").is_none());
    assert!(message.signals().find("").is_none());
}

#[test]
fn test_message_signals_find_empty_collection() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();
    let signals: &[Signal] = &[];
    let message = Message::parse(&mut parser, signals).unwrap();

    assert!(message.signals().find("RPM").is_none());
    assert!(message.signals().find("").is_none());
}

#[test]
fn test_message_getters_edge_cases() {
    // Test with minimum values
    let data = b"BO_ 0 A : 1 B";
    let mut parser = Parser::new(data).unwrap();
    let signals: &[Signal] = &[];
    let message = Message::parse(&mut parser, signals).unwrap();

    assert_eq!(message.id(), 0);
    assert_eq!(message.name(), "A");
    assert_eq!(message.dlc(), 1);
    assert_eq!(message.sender(), "B");
}

#[test]
fn test_message_signals_iterator_empty() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();
    let signals: &[Signal] = &[];
    let message = Message::parse(&mut parser, signals).unwrap();

    let mut iter = message.signals().iter();
    assert!(iter.next().is_none());
}

#[test]
fn test_message_signals_iterator_multiple() {
    let data = b"BO_ 256 EngineData : 8 ECM";
    let mut parser = Parser::new(data).unwrap();

    let signal1 =
        Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
            .unwrap();
    let signal2 =
        Signal::parse(&mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap())
            .unwrap();
    let signal3 =
        Signal::parse(&mut Parser::new(b"SG_ Signal3 : 16|8@0+ (1,0) [0|255] \"\"").unwrap())
            .unwrap();

    let message = Message::parse(&mut parser, &[signal1, signal2, signal3]).unwrap();

    let mut iter = message.signals().iter();
    assert_eq!(iter.next().unwrap().name(), "Signal1");
    assert_eq!(iter.next().unwrap().name(), "Signal2");
    assert_eq!(iter.next().unwrap().name(), "Signal3");
    assert!(iter.next().is_none());
}

// Tests that require std (for string formatting and Display trait)
#[cfg(feature = "std")]
mod tests_std {
    use super::*;

    #[test]
    fn test_message_can_2_0a_dlc_limits() {
        // CAN 2.0A: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_can_2_0b_dlc_limits() {
        // CAN 2.0B: DLC can be 1-8 bytes (8-64 bits)
        // Test valid DLC values
        for dlc in 1..=8 {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }

    #[test]
    fn test_message_can_fd_dlc_limits() {
        // CAN FD: DLC can be 1-64 bytes (8-512 bits)
        // Test valid DLC values up to 64
        for dlc in [1, 8, 12, 16, 20, 24, 32, 48, 64] {
            let data = format!("BO_ 256 EngineData : {} ECM", dlc);
            let mut parser = Parser::new(data.as_bytes()).unwrap();
            let signals: &[Signal] = &[];
            let message = Message::parse(&mut parser, signals).unwrap();
            assert_eq!(message.dlc(), dlc);
        }
    }
    #[test]
    fn test_message_to_dbc_string() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();
        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");
    }

    #[test]
    fn test_message_to_string_full() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\"").unwrap(),
        )
        .unwrap();
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\"").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
        let dbc_string = message.to_string_full();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
    }

    #[test]
    fn test_message_to_dbc_string_empty_signals() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 256 EngineData : 8 ECM");

        let dbc_string_with_signals = message.to_string_full();
        assert_eq!(dbc_string_with_signals, "BO_ 256 EngineData : 8 ECM\n");
    }

    #[test]
    fn test_message_to_dbc_string_special_characters() {
        let data = b"BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(
            dbc_string,
            "BO_ 1234 Test_Message_With_Underscores : 4 Sender_Node"
        );
    }

    #[test]
    fn test_message_to_dbc_string_extended_id() {
        // Use a valid extended ID (max is 0x1FFF_FFFF = 536870911)
        let data = b"BO_ 536870911 ExtendedID : 8 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        let dbc_string = message.to_dbc_string();
        assert_eq!(dbc_string, "BO_ 536870911 ExtendedID : 8 ECM");
    }

    #[test]
    fn test_message_to_dbc_string_dlc_edge_cases() {
        // Test DLC = 1
        let data = b"BO_ 256 MinDLC : 1 ECM";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();
        assert_eq!(message.to_dbc_string(), "BO_ 256 MinDLC : 1 ECM");

        // Test DLC = 64 (CAN FD max)
        let data2 = b"BO_ 257 MaxDLC : 64 ECM";
        let mut parser2 = Parser::new(data2).unwrap();
        let signals_empty: &[Signal] = &[];
        let message2 = Message::parse(&mut parser2, signals_empty).unwrap();
        assert_eq!(message2.to_dbc_string(), "BO_ 257 MaxDLC : 64 ECM");
    }

    #[test]
    fn test_message_display_trait() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signal from DBC string instead of using builder
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal]).unwrap();

        let display_str = format!("{}", message);
        assert!(display_str.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(display_str.contains("SG_ RPM"));
    }

    #[test]
    fn test_message_to_string_full_multiple() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Parse signals from DBC strings instead of using builders
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *").unwrap(),
        )
        .unwrap();

        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Temp : 16|8@0- (1,-40) [-40|215] \"\xC2\xB0C\" *").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();

        let dbc_string = message.to_string_full();
        assert!(dbc_string.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(dbc_string.contains("SG_ RPM"));
        assert!(dbc_string.contains("SG_ Temp"));
        // Should have newlines between signals
        let lines: Vec<&str> = dbc_string.lines().collect();
        assert!(lines.len() >= 3); // Message line + at least 2 signal lines
    }

    #[test]
    fn test_message_signals_iterator_collect() {
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal2 =
            Signal::parse(&mut Parser::new(b"SG_ Signal2 : 8|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();
        let signal3 =
            Signal::parse(&mut Parser::new(b"SG_ Signal3 : 16|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2, signal3]).unwrap();

        // Test that iterator can be used multiple times
        let names: Vec<&str> = message.signals().iter().map(|s| s.name()).collect();
        assert_eq!(names, vec!["Signal1", "Signal2", "Signal3"]);
    }
}

// Edge case tests using Dbc::parse
#[test]
fn test_very_large_message_id() {
    // Test maximum valid extended ID
    let dbc_str = format!(
        r#"VERSION "1.0"

BU_: ECM

BO_ {} MaxID : 8 ECM
"#,
        0x1FFFFFFF
    );
    let dbc = crate::Dbc::parse(&dbc_str).expect("Should accept max extended ID");
    assert_eq!(dbc.messages().at(0).unwrap().id(), 0x1FFFFFFF);
}

#[test]
fn test_invalid_message_id_too_large() {
    // Test ID exceeding 29-bit limit
    let dbc_str = format!(
        r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ {} InvalidID : 8 ECM
"#,
        0x20000000 // Exceeds 29-bit limit
    );
    let result = crate::Dbc::parse(&dbc_str);
    assert!(result.is_err(), "Should reject ID exceeding 29-bit limit");
}

#[test]
fn test_can_fd_dlc_limits() {
    // Test CAN FD DLC up to 64 bytes
    for dlc in [1, 8, 12, 16, 20, 24, 32, 48, 64] {
        let dbc_str = format!(
            r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : {} ECM
"#,
            dlc
        );
        let dbc =
            crate::Dbc::parse(&dbc_str).unwrap_or_else(|_| panic!("Should accept DLC {}", dlc));
        assert_eq!(dbc.messages().at(0).unwrap().dlc(), dlc);
    }
}

#[test]
fn test_invalid_dlc_zero() {
    // DLC 0 should be rejected
    let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 0 ECM
"#;
    let result = crate::Dbc::parse(dbc_str);
    assert!(result.is_err(), "Should reject DLC 0");
}

#[test]
fn test_invalid_dlc_too_large() {
    // DLC > 64 should be rejected
    let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 65 ECM
"#;
    let result = crate::Dbc::parse(dbc_str);
    assert!(result.is_err(), "Should reject DLC > 64");
}

#[test]
fn test_extended_can_id_range() {
    // Test standard and extended ID ranges
    let standard_id = 0x7FF; // Max standard ID
    let extended_id = 0x800; // Min extended ID

    let dbc_str = format!(
        r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ {} Standard : 8 ECM
BO_ {} Extended : 8 ECM
"#,
        standard_id, extended_id
    );
    let dbc = crate::Dbc::parse(&dbc_str).expect("Should accept both ID ranges");
    assert_eq!(dbc.messages().len(), 2);
}

#[test]
fn test_sender_not_in_nodes() {
    // Test sender validation
    let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Test : 8 TCM
"#;
    let result = crate::Dbc::parse(dbc_str);
    assert!(result.is_err(), "Should detect sender not in nodes");
}
