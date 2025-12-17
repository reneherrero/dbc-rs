use crate::{
    Dbc, Error, ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, MAX_MESSAGES,
    MAX_SIGNALS_PER_MESSAGE, Message, Nodes, Parser, Result, Signal, Version,
    compat::Vec,
    dbc::{Messages, Validate},
};
#[cfg(feature = "std")]
use crate::{ValueDescriptions, dbc::ValueDescriptionsMap};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

impl Dbc {
    /// Parse a DBC file from a string slice
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_content = r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 EngineData : 8 ECM
    ///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm""#;
    ///
    /// let dbc = Dbc::parse(dbc_content)?;
    /// assert_eq!(dbc.messages().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse(data: &str) -> Result<Self> {
        let mut parser = Parser::new(data.as_bytes())?;

        let mut messages_buffer: Vec<Message, { MAX_MESSAGES }> = Vec::new();

        let mut message_count_actual = 0;

        // Parse version, nodes, and messages
        use crate::{
            BA_, BA_DEF_, BA_DEF_DEF_, BO_, BO_TX_BU_, BS_, BU_, CM_, EV_, NS_, SG_, SG_MUL_VAL_,
            SIG_GROUP_, SIG_VALTYPE_, VAL_, VAL_TABLE_, VERSION,
        };

        let mut version: Option<Version> = None;
        let mut nodes: Option<Nodes> = None;

        // Store value descriptions during parsing: (message_id, signal_name, value, description)
        #[cfg(feature = "std")]
        type ValueDescriptionsBufferEntry = (
            Option<u32>,
            std::string::String,
            std::vec::Vec<(u64, std::string::String)>,
        );
        #[cfg(feature = "std")]
        let mut value_descriptions_buffer: std::vec::Vec<ValueDescriptionsBufferEntry> =
            std::vec::Vec::new();

        // Store extended multiplexing entries during parsing
        let mut extended_multiplexing_buffer: Vec<
            ExtendedMultiplexing,
            { MAX_EXTENDED_MULTIPLEXING },
        > = Vec::new();

        loop {
            // Skip comments (lines starting with //)
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b"//") {
                parser.skip_to_end_of_line();
                continue;
            }

            let keyword_result = parser.peek_next_keyword();
            let keyword = match keyword_result {
                Ok(kw) => kw,
                Err(Error::UnexpectedEof) => break,
                Err(Error::Expected(_)) => {
                    if parser.starts_with(b"//") {
                        parser.skip_to_end_of_line();
                        continue;
                    }
                    return Err(keyword_result.unwrap_err());
                }
                Err(e) => return Err(e),
            };

            // Save position after peek_next_keyword (which skips whitespace, so we're at the keyword)
            let pos_at_keyword = parser.pos();

            match keyword {
                NS_ => {
                    // Consume NS_ keyword
                    parser
                        .expect(crate::NS_.as_bytes())
                        .map_err(|_| Error::Expected("Failed to consume NS_ keyword"))?;
                    parser.skip_newlines_and_spaces();
                    let _ = parser.expect(b":").ok();
                    loop {
                        parser.skip_newlines_and_spaces();
                        if parser.is_empty() {
                            break;
                        }
                        if parser.starts_with(b" ") || parser.starts_with(b"\t") {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        if parser.starts_with(b"//") {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        if parser.starts_with(BS_.as_bytes())
                            || parser.starts_with(BU_.as_bytes())
                            || parser.starts_with(BO_.as_bytes())
                            || parser.starts_with(SG_.as_bytes())
                            || parser.starts_with(VERSION.as_bytes())
                        {
                            break;
                        }
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                CM_ | BS_ | VAL_TABLE_ | BA_DEF_ | BA_DEF_DEF_ | BA_ | SIG_GROUP_
                | SIG_VALTYPE_ | EV_ | BO_TX_BU_ => {
                    // Consume keyword then skip to end of line
                    let _ = parser.expect(keyword.as_bytes()).ok();
                    parser.skip_to_end_of_line();
                    continue;
                }
                SG_MUL_VAL_ => {
                    // Consume SG_MUL_VAL_ keyword
                    parser
                        .expect(SG_MUL_VAL_.as_bytes())
                        .map_err(|_| Error::Expected("Failed to consume SG_MUL_VAL_ keyword"))?;

                    // Parse the extended multiplexing entry
                    if let Some(ext_mux) = ExtendedMultiplexing::parse(&mut parser) {
                        if extended_multiplexing_buffer.push(ext_mux).is_err() {
                            // Buffer full - return error instead of silently dropping entries
                            return Err(Error::Validation(Error::EXTENDED_MULTIPLEXING_TOO_MANY));
                        }
                    } else {
                        // Parsing failed, skip to end of line
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                VAL_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume VAL_ keyword
                        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
                        // Parse VAL_ statement: VAL_ message_id signal_name value1 "desc1" value2 "desc2" ... ;
                        // Note: message_id of -1 (0xFFFFFFFF) means the value descriptions apply to
                        // all signals with this name in ANY message (global value descriptions)
                        parser.skip_newlines_and_spaces();
                        let message_id = match parser.parse_i64() {
                            Ok(id) => {
                                // -1 (0xFFFFFFFF) is the magic number for global value descriptions
                                if id == -1 {
                                    None
                                } else if id >= 0 && id <= u32::MAX as i64 {
                                    Some(id as u32)
                                } else {
                                    parser.skip_to_end_of_line();
                                    continue;
                                }
                            }
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        parser.skip_newlines_and_spaces();
                        let signal_name = match parser.parse_identifier() {
                            Ok(name) => name.to_string(),
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        // Parse value-description pairs
                        let mut entries: std::vec::Vec<(u64, std::string::String)> =
                            std::vec::Vec::new();
                        loop {
                            parser.skip_newlines_and_spaces();
                            // Check for semicolon (end of VAL_ statement)
                            if parser.starts_with(b";") {
                                parser.expect(b";").ok();
                                break;
                            }
                            // Parse value (as i64 first to handle negative values like -1, then convert to u64)
                            // Note: -1 (0xFFFFFFFF) is the magic number for global value descriptions in message_id,
                            // but values in VAL_ can also be negative
                            let value = match parser.parse_i64() {
                                Ok(v) => {
                                    // Handle -1 specially: convert to 0xFFFFFFFF (u32::MAX) instead of large u64
                                    if v == -1 { 0xFFFF_FFFFu64 } else { v as u64 }
                                }
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            parser.skip_newlines_and_spaces();
                            // Parse description string (expect quote, then take until quote)
                            if parser.expect(b"\"").is_err() {
                                parser.skip_to_end_of_line();
                                break;
                            }
                            let description_bytes = match parser.take_until_quote(false, 1024) {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            let description = match core::str::from_utf8(description_bytes) {
                                Ok(s) => s.to_string(),
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            entries.push((value, description));
                        }
                        if !entries.is_empty() {
                            value_descriptions_buffer.push((message_id, signal_name, entries));
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume VAL_ keyword and skip the rest
                        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                VERSION => {
                    // Version::parse expects VERSION keyword, don't consume it here
                    version = Some(Version::parse(&mut parser)?);
                    continue;
                }
                BU_ => {
                    // Nodes::parse expects BU_ keyword, create parser from original input including it
                    parser.skip_to_end_of_line();
                    let bu_input = &data.as_bytes()[pos_at_keyword..parser.pos()];
                    let mut bu_parser = Parser::new(bu_input)?;
                    nodes = Some(Nodes::parse(&mut bu_parser)?);
                    continue;
                }
                BO_ => {
                    // Check limit using MAX_MESSAGES constant
                    if message_count_actual >= MAX_MESSAGES {
                        return Err(Error::Nodes(Error::NODES_TOO_MANY));
                    }

                    // Save parser position (at BO_ keyword, so Message::parse can consume it)
                    let message_start_pos = pos_at_keyword;

                    // Don't manually parse - just find where the header ends by looking for the colon and sender
                    // We need to find the end of the header line to separate it from signals
                    let header_line_end = {
                        // Skip to end of line to find where header ends
                        let mut temp_parser = Parser::new(&data.as_bytes()[pos_at_keyword..])?;
                        // Skip BO_ keyword
                        temp_parser.expect(crate::BO_.as_bytes()).ok();
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_u32().ok(); // ID
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_identifier().ok(); // name
                        temp_parser.skip_whitespace().ok();
                        temp_parser.expect(b":").ok(); // colon
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_u8().ok(); // DLC
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_identifier().ok(); // sender
                        pos_at_keyword + temp_parser.pos()
                    };

                    // Now parse signals from the original parser
                    parser.skip_to_end_of_line(); // Skip past header line

                    let mut signals_array: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }> = Vec::new();

                    // Parse signals until we find a non-signal line
                    loop {
                        parser.skip_newlines_and_spaces();

                        // Use peek_next_keyword to check for SG_ keyword
                        // peek_next_keyword correctly distinguishes SG_ from SG_MUL_VAL_ (checks longer keywords first)
                        let keyword_result = parser.peek_next_keyword();
                        let keyword = match keyword_result {
                            Ok(kw) => kw,
                            Err(Error::UnexpectedEof) => break,
                            Err(_) => break, // Not a keyword, no more signals
                        };

                        // Only process SG_ signals here (SG_MUL_VAL_ is handled in main loop)
                        if keyword != SG_ {
                            break; // Not a signal, exit signal parsing loop
                        }

                        // Check limit before parsing
                        if signals_array.len() >= MAX_SIGNALS_PER_MESSAGE {
                            return Err(Error::Message(Error::MESSAGE_TOO_MANY_SIGNALS));
                        }

                        // Parse signal - Signal::parse consumes SG_ itself
                        match Signal::parse(&mut parser) {
                            Ok(signal) => {
                                signals_array.push(signal).map_err(|_| {
                                    Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY)
                                })?;
                                // Receivers::parse stops at newline but doesn't consume it
                                // Consume it so next iteration starts at the next line
                                if parser.at_newline() {
                                    parser.skip_to_end_of_line();
                                }
                            }
                            Err(_) => {
                                // Parsing failed, skip to end of line and stop
                                parser.skip_to_end_of_line();
                                break;
                            }
                        }
                    }

                    // Restore parser to start of message line and use Message::parse
                    // Create a new parser from the original input, but only up to the end of the header
                    // (not including signals, so Message::parse doesn't complain about extra content)
                    let message_input = &data.as_bytes()[message_start_pos..header_line_end];
                    let mut message_parser = Parser::new(message_input)?;

                    // Use Message::parse which will parse the header and use our signals
                    let message = Message::parse(&mut message_parser, signals_array.as_slice())?;

                    messages_buffer
                        .push(message)
                        .map_err(|_| Error::Message(Error::NODES_TOO_MANY))?;
                    message_count_actual += 1;
                    continue;
                }
                SG_ => {
                    // Orphaned signal (not inside a message) - skip it
                    parser.skip_to_end_of_line();
                    continue;
                }
                _ => {
                    parser.skip_to_end_of_line();
                    continue;
                }
            }
        }

        // Allow empty nodes (DBC spec allows empty BU_: line)
        let nodes = nodes.unwrap_or_default();

        // If no version was parsed, default to empty version
        let version = version.or_else(|| {
            static EMPTY_VERSION: &[u8] = b"VERSION \"\"";
            let mut parser = Parser::new(EMPTY_VERSION).ok()?;
            Version::parse(&mut parser).ok()
        });

        // Build value descriptions map for storage in Dbc
        #[cfg(feature = "std")]
        let value_descriptions_map = {
            let mut map: BTreeMap<(Option<u32>, std::string::String), ValueDescriptions> =
                BTreeMap::new();
            for (message_id, signal_name, entries) in value_descriptions_buffer {
                let key = (message_id, signal_name);
                let value_descriptions = ValueDescriptions::from_slice(&entries);
                map.insert(key, value_descriptions);
            }
            ValueDescriptionsMap::from_map(map)
        };

        // Convert messages buffer to slice for validation and construction
        let messages_slice: &[Message] = messages_buffer.as_slice();

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        #[cfg(feature = "std")]
        Validate::validate(&nodes, messages_slice, Some(&value_descriptions_map)).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;
        #[cfg(not(feature = "std"))]
        Validate::validate(&nodes, messages_slice).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Construct directly (validation already done)
        let messages = Messages::new(messages_slice)?;
        #[cfg(feature = "std")]
        let dbc = Dbc::new(
            version,
            nodes,
            messages,
            value_descriptions_map,
            extended_multiplexing_buffer,
        );
        #[cfg(not(feature = "std"))]
        let dbc = Dbc::new(version, nodes, messages, extended_multiplexing_buffer);
        Ok(dbc)
    }

    /// Parse a DBC file from a byte slice
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
    /// let dbc = Dbc::parse_bytes(dbc_bytes)?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse_bytes(data: &[u8]) -> Result<Self> {
        let content =
            core::str::from_utf8(data).map_err(|_e| Error::Expected(Error::INVALID_UTF8))?;
        Dbc::parse(content)
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_parse_basic() {
        let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        assert_eq!(dbc.version().map(|v| v.as_str()), Some("1.0"));
        assert!(dbc.nodes().contains("ECM"));
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_bytes() {
        let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
        let dbc = Dbc::parse_bytes(dbc_bytes).unwrap();
        assert_eq!(dbc.version().map(|v| v.as_str()), Some("1.0"));
        assert!(dbc.nodes().contains("ECM"));
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_empty_nodes() {
        let dbc_content = r#"VERSION "1.0"

BU_:

BO_ 256 Engine : 8 ECM
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        assert!(dbc.nodes().is_empty());
    }

    #[test]
    fn test_parse_no_version() {
        let dbc_content = r#"BU_: ECM

BO_ 256 Engine : 8 ECM
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        // Should default to empty version
        assert!(dbc.version().is_some());
    }

    #[test]
    fn parses_real_dbc() {
        let data = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar""#;

        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 2);
        let mut messages_iter = dbc.messages().iter();
        let msg0 = messages_iter.next().unwrap();
        assert_eq!(msg0.signals().len(), 2);
        let mut signals_iter = msg0.signals().iter();
        assert_eq!(signals_iter.next().unwrap().name(), "RPM");
        assert_eq!(signals_iter.next().unwrap().name(), "Temp");
        let msg1 = messages_iter.next().unwrap();
        assert_eq!(msg1.signals().len(), 1);
        assert_eq!(msg1.signals().iter().next().unwrap().name(), "Pressure");
    }

    #[test]
    fn test_parse_duplicate_message_id() {
        use crate::Error;
        // Test that parse also validates duplicate message IDs
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData1 : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"

BO_ 256 EngineData2 : 8 ECM
 SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                assert!(msg.contains(Error::DUPLICATE_MESSAGE_ID));
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_parse_sender_not_in_nodes() {
        use crate::Error;
        // Test that parse also validates message senders are in nodes list
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 TCM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                assert!(msg.contains(Error::SENDER_NOT_IN_NODES));
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_parse_empty_file() {
        use crate::Error;
        // Test parsing an empty file
        let result = Dbc::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof => {
                // Empty file should result in unexpected EOF
            }
            _ => panic!("Expected Error::UnexpectedEof"),
        }
    }

    #[test]
    fn test_parse_bytes_invalid_utf8() {
        use crate::Error;
        // Invalid UTF-8 sequence
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        let result = Dbc::parse_bytes(invalid_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected(msg) => {
                assert_eq!(msg, Error::INVALID_UTF8);
            }
            _ => panic!("Expected Error::Expected with INVALID_UTF8"),
        }
    }

    #[test]
    fn test_parse_without_version_with_comment() {
        // DBC file with comment and no VERSION line
        let data = r#"// This is a comment
BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.version().map(|v| v.as_str()), Some(""));
    }

    #[test]
    fn test_parse_with_strict_boundary_check() {
        // Test that strict mode (default) rejects signals that extend beyond boundaries
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ CHECKSUM : 63|8@1+ (1,0) [0|255] ""
"#;

        // Default (strict) mode should fail
        let result = Dbc::parse(data);
        assert!(result.is_err());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_val_value_descriptions() {
        let data = r#"VERSION ""

NS_ :

BS_:

BU_: Node1 Node2

BO_ 100 Message1 : 8 Node1
 SG_ Signal : 32|8@1- (1,0) [-1|4] "Gear" Node2

VAL_ 100 Signal -1 "Reverse" 0 "Neutral" 1 "First" 2 "Second" 3 "Third" 4 "Fourth" ;
"#;

        let dbc = match Dbc::parse(data) {
            Ok(dbc) => dbc,
            Err(e) => panic!("Failed to parse DBC: {:?}", e),
        };

        // Verify basic structure
        assert_eq!(dbc.messages().len(), 1);
        let message = dbc.messages().iter().find(|m| m.id() == 100).unwrap();
        assert_eq!(message.name(), "Message1");
        assert_eq!(message.sender(), "Node1");

        // Verify value descriptions
        let value_descriptions = dbc
            .value_descriptions_for_signal(100, "Signal")
            .expect("Value descriptions should exist");
        assert_eq!(value_descriptions.get(0xFFFFFFFF), Some("Reverse")); // -1 as u64
        assert_eq!(value_descriptions.get(0), Some("Neutral"));
        assert_eq!(value_descriptions.get(1), Some("First"));
        assert_eq!(value_descriptions.get(2), Some("Second"));
        assert_eq!(value_descriptions.get(3), Some("Third"));
        assert_eq!(value_descriptions.get(4), Some("Fourth"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_val_global_value_descriptions() {
        // Test global value descriptions (VAL_ -1) that apply to all signals with the same name
        let data = r#"VERSION "1.0"

NS_ :

    VAL_

BS_:

BU_: ECU DASH

BO_ 256 EngineData: 8 ECU
 SG_ EngineRPM : 0|16@1+ (0.125,0) [0|8000] "rpm" Vector__XXX
 SG_ DI_gear : 24|3@1+ (1,0) [0|7] "" Vector__XXX

BO_ 512 DashboardDisplay: 8 DASH
 SG_ DI_gear : 0|3@1+ (1,0) [0|7] "" Vector__XXX
 SG_ SpeedDisplay : 8|16@1+ (0.01,0) [0|300] "km/h" Vector__XXX

VAL_ -1 DI_gear 0 "INVALID" 1 "P" 2 "R" 3 "N" 4 "D" 5 "S" 6 "L" 7 "SNA" ;
"#;

        let dbc = match Dbc::parse(data) {
            Ok(dbc) => dbc,
            Err(e) => panic!("Failed to parse DBC: {:?}", e),
        };

        // Verify basic structure
        assert_eq!(dbc.messages().len(), 2);

        // Verify first message (EngineData)
        let engine_msg = dbc.messages().iter().find(|m| m.id() == 256).unwrap();
        assert_eq!(engine_msg.name(), "EngineData");
        assert_eq!(engine_msg.sender(), "ECU");
        let di_gear_signal1 = engine_msg.signals().find("DI_gear").unwrap();
        assert_eq!(di_gear_signal1.name(), "DI_gear");
        assert_eq!(di_gear_signal1.start_bit(), 24);

        // Verify second message (DashboardDisplay)
        let dash_msg = dbc.messages().iter().find(|m| m.id() == 512).unwrap();
        assert_eq!(dash_msg.name(), "DashboardDisplay");
        assert_eq!(dash_msg.sender(), "DASH");
        let di_gear_signal2 = dash_msg.signals().find("DI_gear").unwrap();
        assert_eq!(di_gear_signal2.name(), "DI_gear");
        assert_eq!(di_gear_signal2.start_bit(), 0);

        // Verify global value descriptions apply to DI_gear in message 256
        let value_descriptions1 = dbc
            .value_descriptions_for_signal(256, "DI_gear")
            .expect("Global value descriptions should exist for DI_gear in message 256");

        assert_eq!(value_descriptions1.get(0), Some("INVALID"));
        assert_eq!(value_descriptions1.get(1), Some("P"));
        assert_eq!(value_descriptions1.get(2), Some("R"));
        assert_eq!(value_descriptions1.get(3), Some("N"));
        assert_eq!(value_descriptions1.get(4), Some("D"));
        assert_eq!(value_descriptions1.get(5), Some("S"));
        assert_eq!(value_descriptions1.get(6), Some("L"));
        assert_eq!(value_descriptions1.get(7), Some("SNA"));

        // Verify global value descriptions also apply to DI_gear in message 512
        let value_descriptions2 = dbc
            .value_descriptions_for_signal(512, "DI_gear")
            .expect("Global value descriptions should exist for DI_gear in message 512");

        // Both should return the same value descriptions (same reference or same content)
        assert_eq!(value_descriptions2.get(0), Some("INVALID"));
        assert_eq!(value_descriptions2.get(1), Some("P"));
        assert_eq!(value_descriptions2.get(2), Some("R"));
        assert_eq!(value_descriptions2.get(3), Some("N"));
        assert_eq!(value_descriptions2.get(4), Some("D"));
        assert_eq!(value_descriptions2.get(5), Some("S"));
        assert_eq!(value_descriptions2.get(6), Some("L"));
        assert_eq!(value_descriptions2.get(7), Some("SNA"));

        // Verify they should be the same instance (both reference the global entry)
        // Since we store by (Option<u32>, &str), both should return the same entry
        assert_eq!(value_descriptions1.len(), value_descriptions2.len());
        assert_eq!(value_descriptions1.len(), 8);

        // Verify other signals don't have value descriptions
        assert_eq!(dbc.value_descriptions_for_signal(256, "EngineRPM"), None);
        assert_eq!(dbc.value_descriptions_for_signal(512, "SpeedDisplay"), None);
    }

    // ============================================================================
    // Specification Compliance Tests
    // These tests verify against exact requirements from dbc/SPECIFICATIONS.md
    // ============================================================================

    /// Verify Section 8.3: DLC = 0 is valid
    /// "CAN 2.0: 0 to 8 bytes"
    /// "CAN FD: 0 to 64 bytes"
    #[test]
    fn test_spec_section_8_3_dlc_zero_is_valid() {
        // DLC = 0 is valid per spec (e.g., for control messages without data payload)
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 ControlMessage : 0 ECM
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.dlc(), 0);
    }

    /// Verify Section 8.1: Extended CAN ID format
    /// "Extended ID in DBC = 0x80000000 | actual_extended_id"
    /// "Example: 0x80001234 represents extended ID 0x1234"
    #[test]
    fn test_spec_section_8_1_extended_can_id_format() {
        // Extended ID 0x494 is stored as 0x80000000 | 0x494 = 0x80000494 = 2147484820
        // 0x80000000 = 2147483648, 0x494 = 1172, 2147483648 + 1172 = 2147484820
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 2147484820 ExtendedMessage : 8 ECM
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.id(), 2147484820);
        // Verify this is extended ID 0x494 with bit 31 set
        assert_eq!(msg.id() & 0x80000000, 0x80000000); // Bit 31 is set
        assert_eq!(msg.id() & 0x1FFFFFFF, 0x494); // Lower 29 bits are the actual ID
    }

    /// Verify Section 8.3: Maximum extended ID (0x1FFFFFFF) with bit 31 flag
    #[test]
    fn test_spec_section_8_1_max_extended_id() {
        // Maximum extended ID: 0x80000000 | 0x1FFFFFFF = 0x9FFFFFFF = 2684354559
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 2684354559 MaxExtendedId : 8 ECM
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.id(), 0x9FFFFFFF);
    }

    /// Verify Section 8.4: Vector__XXX as transmitter
    /// "Vector__XXX - No sender / unknown sender"
    #[test]
    fn test_spec_section_8_4_vector_xxx_transmitter() {
        let data = r#"VERSION "1.0"

BU_: Gateway

BO_ 256 UnknownSender : 8 Vector__XXX
 SG_ Signal1 : 0|8@1+ (1,0) [0|255] "" Gateway
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.sender(), "Vector__XXX");
    }

    /// Verify Section 9.5: Receivers format
    /// Parser accepts both comma-separated (per spec) and space-separated (tool extension)
    #[test]
    fn test_spec_section_9_5_receivers_comma_separated() {
        // Comma-separated receivers (per spec)
        // Note: The parser identifier function stops at commas, so we test that comma-separated
        // receiver parsing works correctly
        use crate::{Parser, Signal};

        // Test comma-separated receivers directly via Signal::parse
        let signal = Signal::parse(
            &mut Parser::new(b"SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" Gateway,Dashboard")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(signal.receivers().len(), 2);
        let mut receivers = signal.receivers().iter();
        assert_eq!(receivers.next(), Some("Gateway"));
        assert_eq!(receivers.next(), Some("Dashboard"));
    }

    /// Verify Section 9.4: Multiplexer indicator patterns
    /// "M" for multiplexer switch, "m0", "m1", etc. for multiplexed signals
    #[test]
    fn test_spec_section_9_4_multiplexer_indicators() {
        let data = r#"VERSION "1.0"

BU_: ECM Gateway

BO_ 400 MultiplexedMsg : 8 ECM
 SG_ MuxSwitch M : 0|8@1+ (1,0) [0|255] "" Gateway
 SG_ Signal_0 m0 : 8|16@1+ (0.1,0) [0|1000] "kPa" Gateway
 SG_ Signal_1 m1 : 8|16@1+ (0.01,0) [0|100] "degC" Gateway
"#;
        let dbc = Dbc::parse(data).unwrap();
        let msg = dbc.messages().iter().next().unwrap();

        // Find signals by name
        let mux_switch = msg.signals().find("MuxSwitch").unwrap();
        let signal_0 = msg.signals().find("Signal_0").unwrap();
        let signal_1 = msg.signals().find("Signal_1").unwrap();

        // Verify multiplexer switch
        assert!(mux_switch.is_multiplexer_switch());
        assert_eq!(mux_switch.multiplexer_switch_value(), None);

        // Verify multiplexed signals
        assert!(!signal_0.is_multiplexer_switch());
        assert_eq!(signal_0.multiplexer_switch_value(), Some(0));

        assert!(!signal_1.is_multiplexer_switch());
        assert_eq!(signal_1.multiplexer_switch_value(), Some(1));
    }
}
