#[cfg(feature = "alloc")]
use crate::{Error, Result, error::messages as error_messages};
use crate::{
    Message, Messages, Nodes, ParseOptions, Parser, Signal, Signals, Version,
    error::{ParseError, ParseResult},
};

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Represents a complete DBC (CAN database) file.
///
/// A `Dbc` contains:
/// - An optional version string
/// - A list of nodes (ECUs)
/// - A collection of messages with their signals
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM TCM
///
/// BO_ 256 EngineData : 8 ECM
///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// println!("Parsed {} messages", dbc.messages().len());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct Dbc<'a> {
    version: Option<Version<'a>>,
    nodes: Nodes<'a>,
    messages: Messages<'a>,
}

impl<'a> Dbc<'a> {
    pub(crate) fn validate(
        _version: Option<&Version<'_>>,
        nodes: &Nodes<'_>,
        messages: &[Option<Message<'_>>],
        message_count: usize,
    ) -> ParseResult<()> {
        #[cfg(feature = "alloc")]
        use crate::error::messages as error_messages;

        // Check for duplicate message IDs
        let messages_slice = &messages[..message_count];
        for (i, msg1_opt) in messages_slice.iter().enumerate() {
            let msg1 = match msg1_opt {
                Some(m) => m,
                None => continue, // Should not happen, but be safe
            };
            for msg2_opt in messages_slice.iter().skip(i + 1) {
                let msg2 = match msg2_opt {
                    Some(m) => m,
                    None => continue, // Should not happen, but be safe
                };
                if msg1.id() == msg2.id() {
                    #[cfg(feature = "alloc")]
                    {
                        let msg = error_messages::duplicate_message_id(
                            msg1.id(),
                            msg1.name(),
                            msg2.name(),
                        );
                        return Err(ParseError::Version(msg.leak()));
                    }
                    #[cfg(not(feature = "alloc"))]
                    {
                        return Err(ParseError::Version(
                            crate::error::lang::FORMAT_DUPLICATE_MESSAGE_ID,
                        ));
                    }
                }
            }
        }

        // Validate that all message senders are in the nodes list
        // Skip validation if nodes list is empty (empty nodes allowed per DBC spec)
        if !nodes.is_empty() {
            for msg_opt in messages_slice {
                let msg = match msg_opt {
                    Some(m) => m,
                    None => continue, // Should not happen, but be safe
                };
                if !nodes.contains(msg.sender()) {
                    #[cfg(feature = "alloc")]
                    {
                        let msg_str = error_messages::sender_not_in_nodes(msg.name(), msg.sender());
                        return Err(ParseError::Version(msg_str.leak()));
                    }
                    #[cfg(not(feature = "alloc"))]
                    {
                        return Err(ParseError::Version(
                            crate::error::lang::FORMAT_SENDER_NOT_IN_NODES,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)] // Only used by builders (std-only)
    pub(crate) fn new(
        version: Option<Version<'a>>,
        nodes: Nodes<'a>,
        messages: &'a [Message<'a>],
    ) -> Self {
        // Validation should have been done prior (by builder)
        Self {
            version,
            nodes,
            messages: Messages::from_messages_slice(messages),
        }
    }

    fn new_from_options(
        version: Option<Version<'a>>,
        nodes: Nodes<'a>,
        messages: &[Option<Message<'a>>],
        message_count: usize,
    ) -> Self {
        // Validation should have been done prior (by parse)
        Self {
            version,
            nodes,
            messages: Messages::from_options_slice(messages, message_count),
        }
    }

    /// Get the version of the DBC file
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(version) = dbc.version() {
    ///     println!("Version: {}", version.to_string());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn version(&self) -> Option<&Version<'a>> {
        self.version.as_ref()
    }

    /// Get a reference to the nodes collection
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let nodes = dbc.nodes();
    /// println!("Nodes: {}", nodes.to_string());
    /// println!("Node count: {}", nodes.len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &Nodes<'a> {
        &self.nodes
    }

    /// Get a reference to the messages collection
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let messages = dbc.messages();
    /// println!("Message count: {}", messages.len());
    /// for message in messages.iter() {
    ///     println!("Message: {} (ID: {})", message.name(), message.id());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn messages(&self) -> &Messages<'a> {
        &self.messages
    }

    /// Parse a DBC file from a string slice
    ///
    /// # Examples
    ///
    /// ```
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
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse(data: &'a str) -> ParseResult<Self> {
        Self::parse_with_options(data, ParseOptions::default())
    }

    /// Parses a DBC file from a string with custom parsing options.
    ///
    /// # Arguments
    ///
    /// * `data` - The DBC file content as a string
    /// * `options` - Parsing options to control validation behavior
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::{Dbc, ParseOptions};
    ///
    /// let dbc_content = r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Test : 8 ECM
    ///  SG_ Signal1 : 0|8@1+ (1,0) [0|255] ""
    /// "#;
    ///
    /// // Use lenient mode to allow signals that extend beyond message boundaries
    /// let options = ParseOptions::lenient();
    /// let dbc = Dbc::parse_with_options(dbc_content, options)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse_with_options(data: &'a str, options: ParseOptions) -> ParseResult<Self> {
        // FIRST PASS: Count messages (two-pass parsing to allocate correct sizes)
        let mut parser1 = Parser::new(data.as_bytes())?;
        let _ = Messages::count_messages_and_signals(&mut parser1)?;

        // SECOND PASS: Parse into messages array
        let mut parser2 = Parser::new(data.as_bytes())?;

        // Allocate messages buffer - Messages will handle the size internally
        // We use a temporary buffer that Messages can work with (no alloc in no_std)
        // Messages handles capacity internally, we just need a buffer
        #[cfg(not(feature = "alloc"))]
        let mut messages_buffer = Messages::new_parse_buffer();

        #[cfg(feature = "alloc")]
        let mut messages_buffer: alloc::vec::Vec<Option<Message<'a>>> =
            alloc::vec::Vec::with_capacity(Messages::max_capacity());

        let mut message_count_actual = 0;

        // Parse version, nodes, and messages
        use crate::{
            BA_, BA_DEF_, BA_DEF_DEF_, BO_, BO_TX_BU_, BS_, BU_, CM_, EV_, NS_, SG_, SIG_GROUP_,
            SIG_VALTYPE_, VAL_, VAL_TABLE_, VERSION,
        };

        let mut version: Option<Version<'a>> = None;
        let mut nodes: Option<Nodes<'a>> = None;

        loop {
            // Skip comments (lines starting with //)
            parser2.skip_newlines_and_spaces();
            if parser2.starts_with(b"//") {
                parser2.skip_to_end_of_line();
                continue;
            }

            let keyword_result = parser2.find_next_keyword();
            let keyword = match keyword_result {
                Ok(kw) => kw,
                Err(ParseError::UnexpectedEof) => break,
                Err(ParseError::Expected(_)) => {
                    if parser2.starts_with(b"//") {
                        parser2.skip_to_end_of_line();
                        continue;
                    }
                    return Err(keyword_result.unwrap_err());
                }
                Err(e) => return Err(e),
            };

            match keyword {
                NS_ => {
                    parser2.skip_newlines_and_spaces();
                    let _ = parser2.expect(b":").ok();
                    loop {
                        parser2.skip_newlines_and_spaces();
                        if parser2.is_empty() {
                            break;
                        }
                        if parser2.starts_with(b" ") || parser2.starts_with(b"\t") {
                            parser2.skip_to_end_of_line();
                            continue;
                        }
                        if parser2.starts_with(b"//") {
                            parser2.skip_to_end_of_line();
                            continue;
                        }
                        if parser2.starts_with(BS_.as_bytes())
                            || parser2.starts_with(BU_.as_bytes())
                            || parser2.starts_with(BO_.as_bytes())
                            || parser2.starts_with(SG_.as_bytes())
                            || parser2.starts_with(VERSION.as_bytes())
                        {
                            break;
                        }
                        parser2.skip_to_end_of_line();
                    }
                    continue;
                }
                CM_ | BS_ | VAL_TABLE_ | BA_DEF_ | BA_DEF_DEF_ | BA_ | VAL_ | SIG_GROUP_
                | SIG_VALTYPE_ | EV_ | BO_TX_BU_ => {
                    parser2.skip_to_end_of_line();
                    continue;
                }
                VERSION => {
                    version = Some(Version::parse(&mut parser2)?);
                    continue;
                }
                BU_ => {
                    nodes = Some(Nodes::parse(&mut parser2)?);
                    continue;
                }
                BO_ => {
                    // Check limit using Messages (which knows about the capacity)
                    if message_count_actual >= Messages::max_capacity() {
                        return Err(ParseError::Version(crate::error::lang::NODES_TOO_MANY));
                    }

                    // Save parser position (after BO_ keyword, before message header)
                    let message_start_pos = parser2.pos();

                    // Parse message header to get past it, then parse signals
                    parser2.skip_newlines_and_spaces();
                    let _id = parser2.parse_u32().ok();
                    parser2.skip_newlines_and_spaces();
                    let _name = parser2.parse_identifier().ok();
                    parser2.skip_newlines_and_spaces();
                    let _ = parser2.expect(b":").ok();
                    parser2.skip_newlines_and_spaces();
                    let _dlc = parser2.parse_u8().ok();
                    parser2.skip_newlines_and_spaces();
                    let _sender = parser2.parse_identifier().ok();
                    let message_header_end_pos = parser2.pos();
                    parser2.skip_to_end_of_line();

                    // Parse signals into fixed array
                    #[cfg(not(feature = "alloc"))]
                    let mut signals_array = Signals::new_parse_buffer();

                    #[cfg(feature = "alloc")]
                    let mut signals_array: alloc::vec::Vec<Option<Signal<'a>>> =
                        alloc::vec::Vec::with_capacity(Signals::max_capacity());

                    let mut signal_count = 0;
                    loop {
                        parser2.skip_newlines_and_spaces();
                        if parser2.starts_with(crate::SG_.as_bytes()) {
                            if let Some(next_byte) = parser2.peek_byte_at(3) {
                                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t') {
                                    if signal_count >= Signals::max_capacity() {
                                        return Err(ParseError::Version(
                                            crate::error::messages::SIGNAL_RECEIVERS_TOO_MANY,
                                        ));
                                    }
                                    let _kw = parser2.find_next_keyword().map_err(|e| match e {
                                        ParseError::Expected(_) => {
                                            ParseError::Expected("Expected SG_ keyword")
                                        }
                                        _ => e,
                                    })?;
                                    let signal = Signal::parse(&mut parser2)?;
                                    #[cfg(not(feature = "alloc"))]
                                    {
                                        signals_array[signal_count] = Some(signal);
                                    }
                                    #[cfg(feature = "alloc")]
                                    {
                                        signals_array.push(Some(signal));
                                    }
                                    signal_count += 1;
                                    continue;
                                }
                            }
                        }
                        break;
                    }

                    // Restore parser to start of message line and use Message::parse
                    // Create a new parser from the original input, but only up to the end of the header
                    // (not including signals, so Message::parse doesn't complain about extra content)
                    let message_input = &data.as_bytes()[message_start_pos..message_header_end_pos];
                    let mut message_parser = Parser::new(message_input)?;

                    // Use Message::parse which will parse the header and use our signals
                    let signals_slice: &[Option<Signal<'a>>] = {
                        #[cfg(not(feature = "alloc"))]
                        {
                            &signals_array[..signal_count]
                        }
                        #[cfg(feature = "alloc")]
                        {
                            &signals_array[..]
                        }
                    };
                    let message =
                        Message::parse(&mut message_parser, signals_slice, signal_count, options)?;

                    #[cfg(not(feature = "alloc"))]
                    {
                        messages_buffer[message_count_actual] = Some(message);
                    }
                    #[cfg(feature = "alloc")]
                    {
                        messages_buffer.push(Some(message));
                    }
                    message_count_actual += 1;
                    continue;
                }
                SG_ => {
                    let _ = Signal::parse(&mut parser2)?;
                    continue;
                }
                _ => {
                    parser2.skip_to_end_of_line();
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

        // Convert messages buffer to slice for validation and construction
        let messages_slice: &[Option<Message<'a>>] = {
            #[cfg(not(feature = "alloc"))]
            {
                &messages_buffer[..message_count_actual]
            }
            #[cfg(feature = "alloc")]
            {
                &messages_buffer[..]
            }
        };

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        Self::validate(
            version.as_ref(),
            &nodes,
            messages_slice,
            message_count_actual,
        )?;

        // Construct directly (validation already done)
        Ok(Self::new_from_options(
            version,
            nodes,
            messages_slice,
            message_count_actual,
        ))
    }

    /// Parse a DBC file from a byte slice
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
    /// let dbc = Dbc::parse_bytes(dbc_bytes)?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn parse_bytes(data: &[u8]) -> Result<Dbc<'static>> {
        let content =
            core::str::from_utf8(data).map_err(|e| Error::Dbc(error_messages::invalid_utf8(e)))?;
        // Convert to owned string, box it, and leak to get 'static lifetime
        let owned = String::from(content);
        let boxed = owned.into_boxed_str();
        let content_ref: &'static str = Box::leak(boxed);
        Dbc::parse(content_ref).map_err(Error::ParseError)
    }

    /// Parse a DBC file from a file path
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// // Create a temporary file for the example
    /// let dbc_content = r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Signal1 : 0|8@1+ (1,0) [0|255] ""
    /// "#;
    /// std::fs::write("/tmp/example.dbc", dbc_content)?;
    ///
    /// let dbc = Dbc::from_file("/tmp/example.dbc")?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "std")]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Dbc<'static>> {
        let file =
            std::fs::File::open(path).map_err(|e| Error::Dbc(error_messages::read_failed(e)))?;
        Self::from_reader(file)
    }

    /// Parse a DBC file from a reader
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    /// use std::io::Cursor;
    ///
    /// let data = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
    /// let reader = Cursor::new(data);
    /// let dbc = Dbc::from_reader(reader)?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Dbc<'static>> {
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut reader, &mut buffer)
            .map_err(|e| Error::Dbc(error_messages::read_failed(e)))?;
        // Convert to boxed str and leak to get 'static lifetime
        // The leaked memory will live for the duration of the program
        let boxed = buffer.into_boxed_str();
        let content_ref: &'static str = Box::leak(boxed);
        Dbc::parse(content_ref).map_err(Error::ParseError)
    }

    /// Serialize this DBC to a DBC format string
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let dbc_string = dbc.to_dbc_string();
    /// // The string can be written to a file or used elsewhere
    /// assert!(dbc_string.contains("VERSION"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let signal_count: usize = self.messages().iter().map(|m| m.signals().len()).sum();
        let estimated_capacity = 200 + (self.messages.len() * 50) + (signal_count * 100);
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
        for message in self.messages().iter() {
            result.push('\n');
            result.push_str(&message.to_dbc_string_with_signals());
        }

        result
    }
}

#[cfg(feature = "alloc")]
impl<'a> core::fmt::Display for Dbc<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{
        ByteOrder, Error, Parser, Receivers, Version,
        error::{ParseError, lang},
        nodes::NodesBuilder,
    };
    use crate::{DbcBuilder, MessageBuilder, ReceiversBuilder, SignalBuilder};

    #[test]
    fn test_dbc_new_valid() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").add_node("TCM").build().unwrap();

        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(0.25)
            .offset(0.0)
            .min(0.0)
            .max(8000.0)
            .unit("rpm")
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Temperature")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(-40.0)
            .min(-40.0)
            .max(215.0)
            .unit("°C")
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .add_signal(signal2)
            .build()
            .unwrap();
        let message2 = MessageBuilder::new()
            .id(512)
            .name("BrakeData")
            .dlc(4)
            .sender("TCM")
            .build()
            .unwrap();

        let dbc = DbcBuilder::new(None)
            .version(version)
            .nodes(nodes)
            .add_message(message1)
            .add_message(message2)
            .build()
            .unwrap();
        assert_eq!(dbc.messages().len(), 2);
        let mut messages_iter = dbc.messages().iter();
        assert_eq!(messages_iter.next().unwrap().id(), 256);
        assert_eq!(messages_iter.next().unwrap().id(), 512);
    }

    #[test]
    fn test_dbc_new_duplicate_message_id() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();

        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData1")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build()
            .unwrap();
        let message2 = MessageBuilder::new()
            .id(256)
            .name("EngineData2")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

        let result = DbcBuilder::new(None)
            .version(version)
            .nodes(nodes)
            .add_message(message1)
            .add_message(message2)
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_DUPLICATE_MESSAGE_ID.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end_matches(':').trim_end()));
            }
            _ => panic!("Expected Error::Dbc"),
        }
    }

    #[test]
    fn test_dbc_new_sender_not_in_nodes() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap(); // Only ECM, but message uses TCM

        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("TCM")
            .add_signal(signal)
            .build()
            .unwrap();

        let result =
            DbcBuilder::new(None).version(version).nodes(nodes).add_message(message).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => {
                // Check for format template text (language-agnostic) - extract text before first placeholder
                let template_text = lang::FORMAT_SENDER_NOT_IN_NODES.split("{}").next().unwrap();
                assert!(msg.contains(template_text.trim_end()));
            }
            _ => panic!("Expected Error::Dbc"),
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
    fn test_parse_missing_nodes() {
        // Test parsing without BU_ statement
        // Note: The parser allows missing BU_ line and treats it as empty nodes
        // This is consistent with allowing empty nodes per DBC spec
        let data = r#"VERSION "1.0"

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        // Parser should succeed with empty nodes (missing BU_ is treated as empty nodes)
        assert!(result.is_ok());
        let dbc = result.unwrap();
        assert!(dbc.nodes().is_empty());
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
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.0".to_string())
        );
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    #[cfg(feature = "alloc")]
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
    #[cfg(feature = "alloc")]
    fn test_save_basic() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();

        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(0.25)
            .offset(0.0)
            .min(0.0)
            .max(8000.0)
            .unit("rpm")
            .receivers(ReceiversBuilder::new().broadcast().build().unwrap())
            .build()
            .unwrap();

        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();
        let messages_array = [message];
        // For validation, we need Option array but Message doesn't implement Clone
        // Since this is a simple test with one message, we'll just skip validation
        // In real usage, builders handle this conversion properly
        let dbc = Dbc::new(Some(version), nodes, &messages_array);

        let saved = dbc.to_dbc_string();
        assert!(saved.contains("VERSION \"1.0\""));
        assert!(saved.contains("BU_: ECM"));
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *")); // BigEndian = @0
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
        let saved = dbc.to_dbc_string();
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
    #[cfg(feature = "alloc")]
    fn test_save_multiple_messages() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").add_node("TCM").build().unwrap();

        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(0.25)
            .offset(0.0)
            .min(0.0)
            .max(8000.0)
            .unit("rpm")
            .receivers(Receivers::Broadcast)
            .build()
            .unwrap();

        let signal2 = SignalBuilder::new()
            .name("Pressure")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(0.1)
            .offset(0.0)
            .min(0.0)
            .max(1000.0)
            .unit("bar")
            .receivers(Receivers::None)
            .build()
            .unwrap();

        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1)
            .build()
            .unwrap();
        let message2 = MessageBuilder::new()
            .id(512)
            .name("BrakeData")
            .dlc(4)
            .sender("TCM")
            .add_signal(signal2)
            .build()
            .unwrap();

        let dbc = DbcBuilder::new(None)
            .version(version)
            .nodes(nodes)
            .add_message(message1)
            .add_message(message2)
            .build()
            .unwrap();
        let saved = dbc.to_dbc_string();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_dbc_too_many_messages() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
            .unwrap();

        // Create 10,001 messages (exceeds limit of 10,000)
        let mut messages = Vec::new();
        let message_names: Vec<String> = (0..10_001).map(|i| format!("Message{i}")).collect();
        for i in 0..10_001 {
            let message = MessageBuilder::new()
                .id(i)
                .name(&message_names[i as usize])
                .dlc(8)
                .sender("ECM")
                .add_signal(signal.clone())
                .build()
                .unwrap();
            messages.push(message);
        }

        // Should cap at max capacity even if more messages are provided
        let dbc = Dbc::new(Some(version), nodes, &messages);
        // Use the actual count from the DBC (which will be capped by Messages)
        assert_eq!(dbc.messages().len(), 10_000); // MAX_MESSAGES default
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_dbc_at_message_limit() {
        use crate::nodes::NodesBuilder;

        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none().build().unwrap())
            .build()
            .unwrap();

        // Create exactly 10,000 messages (at the limit)
        let mut messages = Vec::new();
        let message_names: Vec<String> = (0..10_000).map(|i| format!("Message{i}")).collect();
        for i in 0..10_000 {
            let message = MessageBuilder::new()
                .id(i)
                .name(&message_names[i as usize])
                .dlc(8)
                .sender("ECM")
                .add_signal(signal.clone())
                .build()
                .unwrap();
            messages.push(message);
        }

        // Note: Message doesn't implement Clone, so we can't easily convert to Option array for validation
        // Since this test is about testing limits (not validation), we'll skip validation
        // In real usage, builders handle this conversion
        let dbc = Dbc::new(Some(version), nodes, &messages);
        assert_eq!(dbc.messages().len(), 10_000);
    }

    #[test]
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
    fn test_parse_error_with_line_number() {
        // Test that errors include line numbers (or at least that errors are returned)
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
        // Accept any ParseError - line number tracking is not yet implemented
        match err {
            ParseError::Version(_)
            | ParseError::UnexpectedEof
            | ParseError::Expected(_)
            | ParseError::InvalidChar(_) => {
                // Accept various parse errors
            }
            _ => panic!("Expected ParseError"),
        };
        // Note: Line number tracking is not yet implemented, so we just verify an error is returned
    }

    #[test]
    fn test_parse_error_version_with_line_number() {
        // Test that version parsing errors are returned (line number tracking not yet implemented)
        let data = r#"VERSION invalid

BU_: ECM
"#;
        let result = Dbc::parse(data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Accept any ParseError - line number tracking is not yet implemented
        match err {
            ParseError::Version(_) | ParseError::UnexpectedEof | ParseError::Expected(_) => {
                // Accept various parse errors
            }
            _ => panic!("Expected ParseError"),
        };
        // Note: Line number tracking is not yet implemented, so we just verify an error is returned
    }

    #[test]
    fn test_parse_with_lenient_boundary_check() {
        // Test that lenient mode allows signals that extend beyond message boundaries
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ CHECKSUM : 63|8@1+ (1,0) [0|255] ""
"#;

        // Strict mode should fail
        let result = Dbc::parse(data);
        assert!(result.is_err());

        // Lenient mode should succeed
        let options = ParseOptions::lenient();
        let dbc = Dbc::parse_with_options(data, options).unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let message = dbc.messages().at(0).unwrap();
        assert_eq!(message.signals().len(), 1);
        assert_eq!(message.signals().at(0).unwrap().name(), "CHECKSUM");
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

        // Explicit strict mode should also fail
        let options = ParseOptions::new();
        let result = Dbc::parse_with_options(data, options);
        assert!(result.is_err());
    }
}
