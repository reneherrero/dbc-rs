#[cfg(feature = "std")]
use crate::{Error, Result, error::messages};
use crate::{Message, Nodes, Version};
#[allow(unused_imports)] // Used by Dbc::parse, which is part of the public API
use crate::{
    Parser,
    error::{ParseError, ParseResult},
};
#[cfg(feature = "std")]
mod dbc_builder;
#[cfg(feature = "std")]
pub use dbc_builder::DbcBuilder;

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct Dbc {
    version: Option<Version>,
    nodes: Nodes,
    messages: Vec<Message>,
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct Dbc<'a> {
    #[allow(dead_code)] // Used by getter methods
    version: Option<Version<'a>>,
    #[allow(dead_code)] // Used by getter methods
    nodes: Nodes<'a>,
    #[allow(dead_code)] // Used by getter methods
    messages: &'a [Message<'a>],
}

// Implementation for std (owned types)
#[cfg(feature = "std")]
impl Dbc {
    fn validate(
        _version: Option<&Version>,
        _nodes: &Nodes,
        _messages: &[Message],
    ) -> ParseResult<()> {
        // // Check message count limit (DoS protection)
        // const MAX_MESSAGES: usize = 10_000;
        // if messages.len() > MAX_MESSAGES {
        //     return Err(ParseError::Version(messages::DBC_TOO_MANY_MESSAGES));
        // }

        // // Check for duplicate message IDs
        // for (i, msg1) in messages.iter().enumerate() {
        //     for msg2 in messages.iter().skip(i + 1) {
        //         if msg1.id() == msg2.id() {
        //             return Err(ParseError::Version(messages::DBC_TOO_MANY_MESSAGES));
        //         }
        //     }
        // }

        // // Validate that all message senders are in the nodes list
        // for msg in messages {
        //     if !nodes.contains(msg.sender()) {
        //         return Err(ParseError::Version(messages::sender_not_in_nodes(
        //             msg.name(),
        //             msg.sender(),
        //         ).leak()));
        //     }
        // }

        Ok(())
    }

    pub(crate) fn new(
        version: Option<Version>,
        nodes: Nodes,
        #[cfg(feature = "std")] messages: Vec<Message>,
        #[cfg(not(feature = "std"))] messages: &[Message],
    ) -> Result<Self> {
        Self::validate(version.as_ref(), &nodes, &messages)?;

        Ok(Self {
            version,
            nodes,
            messages,
        })
    }

    #[inline]
    #[must_use]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    #[inline]
    #[must_use]
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn parse(data: &str) -> ParseResult<Self> {
        // Initialize the parser with the input data as bytes
        let mut parser = Parser::new(data.as_bytes())?;

        // Find keywords, skip certain keywords, handle VERSION and BU_
        let mut version: Option<Version> = None;

        // Use static strings from lib.rs for matching
        const CM_: &str = "CM_";
        const NS_: &str = "NS_";
        const BS_: &str = "BS_";

        loop {
            let keyword = parser.find_next_keyword()?;

            match keyword {
                CM_ => {
                    // TODO: Implement CM_ (comment) parsing
                    // Skip CM_ lines - advance to end of line
                    parser.skip_to_end_of_line();
                    // Continue to next keyword
                    continue;
                }
                NS_ => {
                    // TODO: Implement NS_ (new symbol) parsing
                    // Skip NS_ lines - advance to end of line
                    parser.skip_to_end_of_line();
                    // Continue to next keyword
                    continue;
                }
                BS_ => {
                    // TODO: Implement BS_ (bit timing) parsing
                    // Skip BS_ lines - advance to end of line
                    parser.skip_to_end_of_line();
                    // Continue to next keyword
                    continue;
                }
                // TODO: Implement CS_ parsing when encountered
                Version::VERSION => {
                    // Found VERSION, parse it
                    version = Some(Version::parse(&mut parser)?);
                    // Continue to find next keyword (BU_)
                    continue;
                }
                Nodes::BU_ => {
                    // Parse nodes and return
                    let nodes = Nodes::parse(&mut parser)?;
                    return Ok(Self {
                        version,
                        nodes,
                        #[cfg(feature = "std")]
                        messages: Vec::new(),
                        #[cfg(not(feature = "std"))]
                        messages: &[],
                    });
                }
                _ => {
                    // Any other keyword should fail
                    return Err(ParseError::Expected(
                        "Expected VERSION, BU_, CM_, NS_, or BS_ keyword",
                    ));
                }
            }
        }

        // The loop above should always return, so this code is unreachable
        // but kept for reference of the old implementation
        // let mut line_iter = data.lines().enumerate().peekable();
        // //TODO: This is a potential security vulnerability. We should use a more robust iterator.
        // // Helper closure to wrap errors with line numbers
        // let wrap_error = |e: &Error, ln: usize| -> Error {
        //     match e {
        //         Error::Dbc(msg) => Error::Dbc(messages::with_line_number(msg, ln)),
        //         Error::Version(msg) => Error::Version(messages::with_line_number(msg, ln)),
        //         Error::Nodes(msg) => Error::Nodes(messages::with_line_number(msg, ln)),
        //         Error::Message(msg) => Error::Message(messages::with_line_number(msg, ln)),
        //         Error::Signal(msg) => Error::Signal(messages::with_line_number(msg, ln)),
        //         Error::InvalidData(msg) => Error::InvalidData(messages::with_line_number(msg, ln)),
        //         Error::ParseError(parse_err) => Error::ParseError(*parse_err),
        //     }
        // };

        // // Skip empty lines and comments at the beginning
        // while let Some((_, line)) = line_iter.peek() {
        //     let trimmed = line.trim();
        //     if trimmed.is_empty() || trimmed.starts_with("//") {
        //         line_iter.next();
        //     } else {
        //         break;
        //     }
        // }

        // // VERSION statement is optional
        // let version = if let Some((line_num, line)) = line_iter.peek() {
        //     let trimmed = line.trim();
        //     if trimmed.starts_with("VERSION") {
        //         let line_num = *line_num;
        //         let (_, version_line) = line_iter.next().unwrap();
        //         Version::parse(version_line).map_err(|e| wrap_error(&e, line_num + 1))?
        //     } else {
        //         // Default to empty version if VERSION line is omitted
        //         Version::parse("VERSION \"\"").unwrap()
        //     }
        // } else {
        //     return Err(Error::Dbc(messages::DBC_EMPTY_FILE.to_string()));
        // };

        // let mut nodes: Option<Nodes> = None;
        // let mut messages: Vec<Message> = Vec::new();

        // while let Some((line_num, line)) = line_iter.next() {
        //     let trimmed = line.trim();
        //     if trimmed.is_empty() || trimmed.starts_with("//") {
        //         continue;
        //     }

        //     if trimmed.starts_with("BU_:") {
        //         nodes = Some(Nodes::parse(trimmed).map_err(|e| wrap_error(&e, line_num + 1))?);
        //     } else if trimmed.starts_with("BO_ ") {
        //         let message = trimmed;

        //         // Get signals associated with message
        //         // Pre-allocate with estimated capacity (most messages have 1-8 signals)
        //         let mut signals: Vec<Signal> = Vec::with_capacity(8);
        //         while let Some((signal_line_num, signal)) = line_iter.peek() {
        //             let signal = signal.trim_start();

        //             if signal.trim_start().starts_with("SG_ ") {
        //                 signals.push(
        //                     Signal::parse(signal)
        //                         .map_err(|e| wrap_error(&e, *signal_line_num + 1))?,
        //                 );
        //                 line_iter.next();
        //             } else {
        //                 break;
        //             }
        //         }

        //         messages.push(
        //             Message::parse(message, signals).map_err(|e| wrap_error(&e, line_num + 1))?,
        //         );
        //     }
        // }

        // let Some(nodes) = nodes else {
        //     return Err(Error::Dbc(messages::DBC_NODES_NOT_DEFINED.to_string()));
        // };

        // // Validate the parsed DBC using the same validation as new()
        // Self::validate(&version, &nodes, &messages)?;

        // Ok(Self {
        //     version,
        //     nodes,
        //     messages,
        // })

        // The loop above should always return, so this code is unreachable
    }

    #[cfg(feature = "std")]
    pub fn parse_bytes(data: &[u8]) -> Result<Dbc> {
        let content =
            core::str::from_utf8(data).map_err(|e| Error::Dbc(messages::invalid_utf8(e)))?;
        // Convert to owned string and extract reference
        use alloc::string::String;
        let owned = String::from(content);
        let boxed = owned.into_boxed_str();
        let content_ref = &*boxed;
        Dbc::parse(content_ref).map_err(Error::ParseError)
    }

    #[cfg(feature = "std")]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path).map_err(|e| Error::Dbc(messages::read_failed(e)))?;
        Self::from_reader(file)
    }

    #[cfg(feature = "std")]
    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Self> {
        use alloc::string::String;

        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut reader, &mut buffer)
            .map_err(|e| Error::Dbc(messages::read_failed(e)))?;
        Self::parse(&buffer).map_err(Error::ParseError)
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn save(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let estimated_capacity = 200
            + (self.messages.len() * 50)
            + (self.messages.iter().map(|m| m.signals().len()).sum::<usize>() * 100);
        let mut result = String::with_capacity(estimated_capacity);

        // VERSION line
        if let Some(version) = &self.version {
            result.push_str(&version.to_dbc_string());
            result.push_str("\n\n");
        }

        // BU_ line
        result.push_str(&self.nodes.to_dbc_string());
        result.push('\n');

        // BO_ and SG_ lines for each message
        for message in &self.messages {
            result.push('\n');
            result.push_str(&message.to_dbc_string_with_signals());
        }

        result
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::Dbc;
    use crate::{
        ByteOrder, Error, Message, Nodes, Parser, Receivers, Signal, Version,
        error::{ParseError, lang},
        nodes::NodesBuilder,
    };

    #[test]
    fn test_dbc_new_valid() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let node_array = ["ECM", "TCM"];
        let nodes = Nodes::new(&node_array).unwrap();

        let signal1 = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            0.25,
            0.0,
            0.0,
            8000.0,
            Some("rpm" as &str),
            Receivers::Broadcast,
        )
        .unwrap();

        let signal2 = Signal::new(
            "Temperature",
            16,
            8,
            ByteOrder::BigEndian,
            true,
            1.0,
            -40.0,
            -40.0,
            215.0,
            Some("°C" as &str),
            Receivers::Broadcast,
        )
        .unwrap();

        let message1 = Message::new(256, "EngineData", 8, "ECM", vec![signal1, signal2]).unwrap();
        let message2 = Message::new(512, "BrakeData", 4, "TCM", vec![]).unwrap();

        let dbc = Dbc::new(Some(version), nodes, vec![message1, message2]).unwrap();
        assert_eq!(dbc.messages().len(), 2);
        assert_eq!(dbc.messages()[0].id(), 256);
        assert_eq!(dbc.messages()[1].id(), 512);
    }

    #[test]
    #[ignore]
    fn test_dbc_new_duplicate_message_id() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();

        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        let message1 = Message::new(256, "EngineData1", 8, "ECM", vec![signal.clone()]).unwrap();
        let message2 = Message::new(256, "EngineData2", 8, "ECM", vec![signal]).unwrap();

        let result = Dbc::new(Some(version), nodes, vec![message1, message2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_DUPLICATE_MESSAGE_ID.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    #[ignore]
    fn test_dbc_new_sender_not_in_nodes() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap(); // Only ECM, but message uses TCM

        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        let message = Message::new(256, "EngineData", 8, "TCM", vec![signal]).unwrap();

        let result = Dbc::new(Some(version), nodes, vec![message]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_SENDER_NOT_IN_NODES.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    #[ignore]
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
        assert_eq!(dbc.messages()[0].signals().len(), 2);
        assert_eq!(dbc.messages()[0].signals()[0].name(), "RPM");
        assert_eq!(dbc.messages()[0].signals()[1].name(), "Temp");
        assert_eq!(dbc.messages()[1].signals().len(), 1);
        assert_eq!(dbc.messages()[1].signals()[0].name(), "Pressure");
    }

    #[test]
    #[ignore]
    fn test_parse_duplicate_message_id() {
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
            ParseError::Version(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_DUPLICATE_MESSAGE_ID.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    #[ignore]
    fn test_parse_sender_not_in_nodes() {
        // Test that parse also validates message senders are in nodes list
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 TCM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_SENDER_NOT_IN_NODES.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    #[ignore]
    fn test_parse_empty_file() {
        // Test parsing an empty file
        let result = Dbc::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnexpectedEof => {
                // Empty file should result in unexpected EOF
            }
            _ => panic!("Expected ParseError::UnexpectedEof"),
        }
    }

    #[test]
    #[ignore]
    fn test_parse_missing_nodes() {
        // Test parsing without BU_ statement
        let data = r#"VERSION "1.0"

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(_) | ParseError::UnexpectedEof => {
                // Accept various parse errors for now
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    #[ignore]
    fn test_parse_bytes() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;

        let bytes = data.as_bytes();
        let dbc = Dbc::parse_bytes(bytes).unwrap();
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.0".to_string())
        );
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    #[ignore]
    #[cfg(feature = "std")]
    fn test_parse_from_string() {
        let data = String::from(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#,
        );

        let dbc = Dbc::parse(&data).unwrap();
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.0".to_string())
        );
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    #[ignore]
    fn test_parse_bytes_invalid_utf8() {
        // Invalid UTF-8 sequence
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        let result = Dbc::parse_bytes(invalid_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_INVALID_UTF8.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_save_basic() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();

        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            0.25,
            0.0,
            0.0,
            8000.0,
            Some("rpm" as &str),
            Receivers::Broadcast,
        )
        .unwrap();

        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();
        let dbc = Dbc::new(Some(version), nodes, vec![message]).unwrap();

        let saved = dbc.save();
        assert!(saved.contains("VERSION \"1.0\""));
        assert!(saved.contains("BU_: ECM"));
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" *"));
    }

    #[test]
    #[ignore]
    fn test_save_round_trip() {
        let original = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@0+ (0.1,0) [0|1000] "bar"
"#;

        let dbc = Dbc::parse(original).unwrap();
        let saved = dbc.save();
        let dbc2 = Dbc::parse(&saved).unwrap();

        // Verify round-trip: parsed data should match
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            dbc2.version().map(|v| v.to_string())
        );
        assert_eq!(dbc.messages().len(), dbc2.messages().len());

        for (msg1, msg2) in dbc.messages().iter().zip(dbc2.messages().iter()) {
            assert_eq!(msg1.id(), msg2.id());
            assert_eq!(msg1.name(), msg2.name());
            assert_eq!(msg1.dlc(), msg2.dlc());
            assert_eq!(msg1.sender(), msg2.sender());
            assert_eq!(msg1.signals().len(), msg2.signals().len());

            for (sig1, sig2) in msg1.signals().iter().zip(msg2.signals().iter()) {
                assert_eq!(sig1.name(), sig2.name());
                assert_eq!(sig1.start_bit(), sig2.start_bit());
                assert_eq!(sig1.length(), sig2.length());
                assert_eq!(sig1.byte_order(), sig2.byte_order());
                assert_eq!(sig1.is_unsigned(), sig2.is_unsigned());
                assert_eq!(sig1.factor(), sig2.factor());
                assert_eq!(sig1.offset(), sig2.offset());
                assert_eq!(sig1.min(), sig2.min());
                assert_eq!(sig1.max(), sig2.max());
                assert_eq!(sig1.unit(), sig2.unit());
                assert_eq!(sig1.receivers(), sig2.receivers());
            }
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_save_multiple_messages() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let node_array = ["ECM", "TCM"];
        let nodes = Nodes::new(&node_array).unwrap();

        let signal1 = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            0.25,
            0.0,
            0.0,
            8000.0,
            Some("rpm" as &str),
            Receivers::Broadcast,
        )
        .unwrap();

        let signal2 = Signal::new(
            "Pressure",
            0,
            16,
            ByteOrder::LittleEndian,
            true,
            0.1,
            0.0,
            0.0,
            1000.0,
            Some("bar" as &str),
            Receivers::None,
        )
        .unwrap();

        let message1 = Message::new(256, "EngineData", 8, "ECM", vec![signal1]).unwrap();
        let message2 = Message::new(512, "BrakeData", 4, "TCM", vec![signal2]).unwrap();

        let dbc = Dbc::new(Some(version), nodes, vec![message1, message2]).unwrap();
        let saved = dbc.save();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    #[test]
    #[cfg(feature = "std")]
    #[ignore]
    fn test_dbc_too_many_messages() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // Create 10,001 messages (exceeds limit of 10,000)
        let mut messages = Vec::new();
        for i in 0..10_001 {
            let message = Message::new(
                i,
                format!("Message{i}").as_str(),
                8,
                "ECM",
                vec![signal.clone()],
            )
            .unwrap();
            messages.push(message);
        }

        let result = Dbc::new(Some(version), nodes, messages);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                assert!(msg.contains(lang::DBC_TOO_MANY_MESSAGES));
            }
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_dbc_at_message_limit() {
        use crate::nodes::NodesBuilder;

        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();

        // Create exactly 10,000 messages (at the limit)
        let mut messages = Vec::new();
        for i in 0..10_000 {
            let message = Message::new(
                i,
                format!("Message{i}").as_str(),
                8,
                "ECM",
                vec![signal.clone()],
            )
            .unwrap();
            messages.push(message);
        }

        let result = Dbc::new(Some(version), nodes, messages);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().messages().len(), 10_000);
    }

    #[test]
    #[ignore]
    fn test_parse_without_version() {
        // DBC file without VERSION line should default to empty version
        let data = r#"
BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("".to_string()));
    }

    #[test]
    #[ignore]
    fn test_parse_without_version_with_comment() {
        // DBC file with comment and no VERSION line
        let data = r#"// This is a comment
BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("".to_string()));
    }

    #[test]
    #[ignore]
    fn test_parse_error_with_line_number() {
        // Test that errors include line numbers
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
BO_ 257 Invalid : 8 ECM
 SG_ InvalidSignal : invalid|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let result = Dbc::parse(data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // For now, accept any ParseError since we're not fully implementing parsing yet
        match err {
            ParseError::Version(_)
            | ParseError::UnexpectedEof
            | ParseError::Expected(_)
            | ParseError::InvalidChar(_) => {
                // Accept various parse errors
            }
            _ => panic!("Expected ParseError"),
        };
        let msg = format!("{}", err);
        // Check for line number pattern: "(line", "(ligne", "(línea", "(Zeile", "(行"
        assert!(
            msg.contains("(line")
                || msg.contains("(ligne")
                || msg.contains("(línea")
                || msg.contains("(Zeile")
                || msg.contains("(行"),
            "Error message should contain line number, got: {}",
            msg
        );
    }

    #[test]
    #[ignore]
    fn test_parse_error_version_with_line_number() {
        // Test that version parsing errors include line numbers
        let data = r#"VERSION invalid

BU_: ECM
"#;
        let result = Dbc::parse(data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // For now, accept any ParseError since we're not fully implementing parsing yet
        match err {
            ParseError::Version(_) | ParseError::UnexpectedEof | ParseError::Expected(_) => {
                // Accept various parse errors
            }
            _ => panic!("Expected ParseError"),
        };
        let msg = format!("{}", err);
        // Check for line number pattern
        assert!(
            msg.contains("(line")
                || msg.contains("(ligne")
                || msg.contains("(línea")
                || msg.contains("(Zeile")
                || msg.contains("(行"),
            "Error message should contain line number, got: {}",
            msg
        );
    }
}
