use crate::{error::messages, Error, Message, Nodes, Signal, Version};
use alloc::{string::String, string::ToString, vec::Vec};

#[derive(Debug)]
pub struct Dbc {
    version: Version,
    nodes: Nodes,
    messages: Vec<Message>,
}

impl Dbc {
    /// Validate DBC parameters
    fn validate(_version: &Version, nodes: &Nodes, messages: &[Message]) -> Result<(), Error> {
        // Check for duplicate message IDs
        for (i, msg1) in messages.iter().enumerate() {
            for msg2 in messages.iter().skip(i + 1) {
                if msg1.id() == msg2.id() {
                    return Err(Error::InvalidData(messages::duplicate_message_id(
                        msg1.id(),
                        msg1.name(),
                        msg2.name(),
                    )));
                }
            }
        }

        // Validate that all message senders are in the nodes list
        for msg in messages {
            if !nodes.contains(msg.sender()) {
                return Err(Error::InvalidData(messages::sender_not_in_nodes(
                    msg.name(),
                    msg.sender(),
                )));
            }
        }

        Ok(())
    }

    /// Create a new DBC instance with the given parameters
    ///
    /// This method validates all input data before creating the DBC instance.
    /// Use this when programmatically creating DBC files rather than parsing from text.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There are duplicate message IDs
    /// - Any message sender is not in the nodes list
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::{Dbc, Version, Nodes, Message, Signal, ByteOrder, Receivers};
    ///
    /// let version = Version::new(1, Some(0), None)?;
    /// let nodes = Nodes::new(&["ECM", "TCM"]);
    ///
    /// let signal = Signal::new(
    ///     "RPM",
    ///     0,
    ///     16,
    ///     ByteOrder::BigEndian,
    ///     true,
    ///     0.25,
    ///     0.0,
    ///     0.0,
    ///     8000.0,
    ///     Some("rpm" as &str),
    ///     Receivers::Broadcast,
    /// )?;
    ///
    /// let message = Message::new(256, "EngineData", 8, "ECM", vec![signal])?;
    ///
    /// let dbc = Dbc::new(version, nodes, vec![message])?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`parse`](Self::parse) - Parse from string slice
    /// - [`Version::new`](crate::Version::new) - Create version
    /// - [`Nodes::new`](crate::Nodes::new) - Create nodes
    /// - [`Message::new`](crate::Message::new) - Create message
    pub fn new(version: Version, nodes: Nodes, messages: Vec<Message>) -> Result<Self, Error> {
        Self::validate(&version, &nodes, &messages)?;

        Ok(Self {
            version,
            nodes,
            messages,
        })
    }

    /// Get the version information
    #[inline]
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Get the nodes information
    #[inline]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Get a read-only slice of messages
    #[inline]
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Parse a DBC file from a string slice
    ///
    /// This is the core parsing method that works in both `std` and `no_std` environments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file is empty
    /// - The version line is missing or invalid
    /// - Nodes are not defined
    /// - Any message or signal fails to parse
    /// - Validation fails (duplicate IDs, invalid senders, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let content = "VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
    /// let dbc = Dbc::parse(content)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`parse_bytes`](Self::parse_bytes) - Parse from bytes
    /// - [`parse_from`](Self::parse_from) - Parse from owned String
    /// - [`from_reader`](Self::from_reader) - Parse from `std::io::Read` (requires `std` feature)
    pub fn parse(data: &str) -> Result<Self, Error> {
        let mut lines = data.lines().peekable();

        // Must start with VERSION statement
        let version = if let Some(v) = lines.next() {
            v
        } else {
            return Err(Error::InvalidData(messages::DBC_EMPTY_FILE.to_string()));
        };
        let version = Version::parse(version)?;

        let mut nodes: Option<Nodes> = None;
        let mut messages: Vec<Message> = Vec::new();

        while let Some(line) = lines.next() {
            if line.starts_with("BU_") {
                nodes = Some(Nodes::parse(line)?);
            } else if line.starts_with("BO_") {
                let message = line;

                // Get signals associated message
                // Pre-allocate with estimated capacity (most messages have 1-8 signals)
                let mut signals: Vec<Signal> = Vec::with_capacity(8);
                while let Some(signal) = lines.peek() {
                    let signal = signal.trim_start();

                    if signal.trim_start().starts_with("SG_") {
                        signals.push(Signal::parse(signal)?);
                        lines.next();
                    } else {
                        break;
                    }
                }

                messages.push(Message::parse(message, signals)?);
            }
        }

        let nodes = match nodes {
            Some(val) => val,
            None => {
                return Err(Error::InvalidData(
                    messages::DBC_NODES_NOT_DEFINED.to_string(),
                ));
            }
        };

        // Validate the parsed DBC using the same validation as new()
        Self::validate(&version, &nodes, &messages)?;

        Ok(Self {
            version,
            nodes,
            messages,
        })
    }

    /// Parse a DBC file from a byte slice
    ///
    /// This method accepts `&[u8]` and converts it to a string for parsing.
    /// Works in both `std` and `no_std` environments.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
    /// let dbc = Dbc::parse_bytes(bytes)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse_bytes(data: &[u8]) -> Result<Self, Error> {
        let content = core::str::from_utf8(data)
            .map_err(|e| Error::InvalidData(messages::invalid_utf8(e)))?;
        Self::parse(content)
    }

    /// Parse a DBC file from any type that can be converted to a string slice
    ///
    /// This is a convenience method that works with `String`, `&str`, `Box<str>`, etc.
    /// Works in both `std` and `no_std` environments.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let content = String::from("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"");
    /// let dbc = Dbc::parse_from(content)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`parse`](Self::parse) - Parse from string slice
    /// - [`parse_bytes`](Self::parse_bytes) - Parse from bytes
    /// - [`from_reader`](Self::from_reader) - Parse from `std::io::Read` (requires `std` feature)
    pub fn parse_from<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        Self::parse(data.as_ref())
    }

    /// Serialize the DBC structure back to DBC file format
    ///
    /// This method converts a `Dbc` instance back into a string representation
    /// that matches the DBC file format. It uses the `to_dbc_string()` methods
    /// of the individual components (Version, Nodes, Message, Signal) to compose
    /// the complete DBC file.
    ///
    /// Works in both `std` and `no_std` environments.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let content = "VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
    /// let dbc = Dbc::parse(content)?;
    /// let saved = dbc.save();
    /// // The saved content should be equivalent to the original
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`parse`](Self::parse) - Parse a DBC file from string
    /// - [`Version::to_dbc_string`](crate::Version::to_dbc_string) - Serialize version
    /// - [`Message::to_dbc_string_with_signals`](crate::Message::to_dbc_string_with_signals) - Serialize message
    pub fn save(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let estimated_capacity = 200
            + (self.messages.len() * 50)
            + (self
                .messages
                .iter()
                .map(|m| m.signals().len())
                .sum::<usize>()
                * 100);
        let mut result = String::with_capacity(estimated_capacity);

        // VERSION line
        result.push_str(&self.version.to_dbc_string());
        result.push_str("\n\n");

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

#[cfg(feature = "std")]
impl Dbc {
    /// Parse a DBC file from any type implementing `std::io::Read`
    ///
    /// This method reads from files, network streams, in-memory buffers, or any other
    /// source that implements `std::io::Read`. Only available when the `std` feature is enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading from the source fails
    /// - The data is not valid UTF-8
    /// - The DBC file format is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dbc_rs::Dbc;
    /// use std::fs::File;
    ///
    /// let file = File::open("example.dbc").expect("file not found");
    /// let dbc = Dbc::from_reader(file).expect("failed to parse");
    /// ```
    ///
    /// Reading from a buffer:
    ///
    /// ```
    /// use dbc_rs::Dbc;
    /// use std::io::Cursor;
    ///
    /// let data = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
    /// let cursor = Cursor::new(data);
    /// let dbc = Dbc::from_reader(cursor)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`parse`](Self::parse) - Parse from string slice (works in `no_std`)
    /// - [`parse_bytes`](Self::parse_bytes) - Parse from bytes (works in `no_std`)
    /// - [`parse_from`](Self::parse_from) - Parse from owned string types (works in `no_std`)
    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Self, Error> {
        use alloc::string::String;

        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut reader, &mut buffer)
            .map_err(|e| Error::InvalidData(messages::read_failed(e)))?;
        Self::parse(&buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::Dbc;
    use crate::{ByteOrder, Error, Message, Nodes, Receivers, Signal, Version};

    #[test]
    fn test_dbc_new_valid() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::new(&["ECM", "TCM"]);

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

        let dbc = Dbc::new(version, nodes, vec![message1, message2]).unwrap();
        assert_eq!(dbc.messages().len(), 2);
        assert_eq!(dbc.messages()[0].id(), 256);
        assert_eq!(dbc.messages()[1].id(), 512);
    }

    #[test]
    fn test_dbc_new_duplicate_message_id() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::new(&["ECM"]);

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

        let result = Dbc::new(version, nodes, vec![message1, message2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Duplicate message ID")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_dbc_new_sender_not_in_nodes() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::new(&["ECM"]); // Only ECM, but message uses TCM

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

        let result = Dbc::new(version, nodes, vec![message]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("not in the nodes list")),
            _ => panic!("Expected InvalidData error"),
        }
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
        assert_eq!(dbc.messages()[0].signals().len(), 2);
        assert_eq!(dbc.messages()[0].signals()[0].name(), "RPM");
        assert_eq!(dbc.messages()[0].signals()[1].name(), "Temp");
        assert_eq!(dbc.messages()[1].signals().len(), 1);
        assert_eq!(dbc.messages()[1].signals()[0].name(), "Pressure");
    }

    #[test]
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
            Error::InvalidData(msg) => assert!(msg.contains("Duplicate message ID")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
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
            Error::InvalidData(msg) => assert!(msg.contains("not in the nodes list")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_parse_empty_file() {
        // Test parsing an empty file
        let result = Dbc::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Empty DBC file")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_parse_missing_nodes() {
        // Test parsing without BU_ statement
        let data = r#"VERSION "1.0"

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Nodes (BU_) are not defined")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_parse_bytes() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;

        let bytes = data.as_bytes();
        let dbc = Dbc::parse_bytes(bytes).unwrap();
        assert_eq!(dbc.version().major(), 1);
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_from_string() {
        let data = String::from(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#,
        );

        let dbc = Dbc::parse_from(data).unwrap();
        assert_eq!(dbc.version().major(), 1);
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_bytes_invalid_utf8() {
        // Invalid UTF-8 sequence
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        let result = Dbc::parse_bytes(invalid_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Invalid UTF-8")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_save_basic() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::new(&["ECM"]);

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
        let dbc = Dbc::new(version, nodes, vec![message]).unwrap();

        let saved = dbc.save();
        assert!(saved.contains("VERSION \"1.0\""));
        assert!(saved.contains("BU_: ECM"));
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\" *"));
    }

    #[test]
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
        assert_eq!(dbc.version().major(), dbc2.version().major());
        assert_eq!(dbc.version().minor(), dbc2.version().minor());
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
    fn test_save_multiple_messages() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::new(&["ECM", "TCM"]);

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

        let dbc = Dbc::new(version, nodes, vec![message1, message2]).unwrap();
        let saved = dbc.save();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    #[test]
    fn test_save_empty_nodes() {
        let version = Version::new(1, Some(0), None).unwrap();
        let nodes = Nodes::empty();
        let dbc = Dbc::new(version, nodes, vec![]).unwrap();

        let saved = dbc.save();
        assert!(saved.contains("VERSION \"1.0\""));
        assert!(saved.contains("BU_:"));
        // Should not have a space after BU_: when empty
        let bu_line = saved.lines().find(|l| l.starts_with("BU_")).unwrap();
        assert_eq!(bu_line, "BU_:");
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod std_tests {
    use super::Dbc;
    use std::io::Cursor;

    #[test]
    fn test_from_reader_cursor() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;

        let cursor = Cursor::new(data.as_bytes());
        let dbc = Dbc::from_reader(cursor).unwrap();
        assert_eq!(dbc.version().major(), 1);
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_from_reader_file() {
        // Test reading from an actual file
        let content =
            std::fs::read_to_string("tests/data/simple.dbc").expect("Failed to read test file");
        let cursor = Cursor::new(content.as_bytes());
        let dbc = Dbc::from_reader(cursor).unwrap();
        assert_eq!(dbc.messages().len(), 2);
    }
}
