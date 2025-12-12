use crate::{
    Error, MAX_MESSAGES, MAX_SIGNALS_PER_MESSAGE, Message, MessageList, Nodes, Parser, Result,
    Signal, Version, compat::Vec, error::lang,
};
#[cfg(feature = "std")]
use crate::{ValueDescriptions, ValueDescriptionsList};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

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
pub struct Dbc {
    version: Option<Version>,
    nodes: Nodes,
    messages: MessageList,
    #[cfg(feature = "std")]
    value_descriptions: ValueDescriptionsList,
}

impl Dbc {
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
                        return Err(Error::Validation(lang::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND));
                    }

                    // Check if signal exists in the message
                    let signal_exists = messages.iter().any(|msg| {
                        msg.id() == message_id && msg.signals().find(signal_name).is_some()
                    });
                    if !signal_exists {
                        return Err(Error::Validation(lang::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
                    }
                } else {
                    // For global value descriptions (message_id is None), check if signal exists in any message
                    let signal_exists =
                        messages.iter().any(|msg| msg.signals().find(signal_name).is_some());
                    if !signal_exists {
                        return Err(Error::Validation(lang::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
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
                    return Err(Error::Validation(lang::DUPLICATE_MESSAGE_ID));
                }
            }
        }

        // Validate that all message senders are in the nodes list
        // Skip validation if nodes list is empty (empty nodes allowed per DBC spec)
        if !nodes.is_empty() {
            for msg in messages {
                if !nodes.contains(msg.sender()) {
                    return Err(Error::Validation(lang::SENDER_NOT_IN_NODES));
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "std")]
    pub(crate) fn new(
        version: Option<Version>,
        nodes: Nodes,
        messages: MessageList,
        value_descriptions: ValueDescriptionsList,
    ) -> Self {
        // Validation should have been done prior (by builder)
        Self {
            version,
            nodes,
            messages,
            value_descriptions,
        }
    }

    /// Get the version of the DBC file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(version) = dbc.version() {
    ///     // Version is available
    ///     let _ = version.as_str();
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// Get a reference to the nodes collection
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let nodes = dbc.nodes();
    /// assert_eq!(nodes.len(), 2);
    /// // Iterate over nodes
    /// let mut iter = nodes.iter();
    /// assert_eq!(iter.next(), Some("ECM"));
    /// assert_eq!(iter.next(), Some("TCM"));
    /// assert_eq!(iter.next(), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Get a reference to the messages collection
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let messages = dbc.messages();
    /// assert_eq!(messages.len(), 1);
    /// let message = messages.at(0).unwrap();
    /// assert_eq!(message.name(), "Engine");
    /// assert_eq!(message.id(), 256);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn messages(&self) -> &MessageList {
        &self.messages
    }

    /// Get value descriptions for a specific signal
    ///
    /// Value descriptions map numeric signal values to human-readable text.
    /// Returns `None` if the signal has no value descriptions.
    ///
    /// **Global Value Descriptions**: According to the Vector DBC specification,
    /// a message_id of `-1` (0xFFFFFFFF) in a `VAL_` statement means the value
    /// descriptions apply to all signals with that name in ANY message. This
    /// method will first check for a message-specific entry, then fall back to
    /// a global entry if one exists.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(100, "Gear") {
    ///     if let Some(desc) = value_descriptions.get(0) {
    ///         println!("Value 0 means: {}", desc);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// Get a reference to the value descriptions list
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 100 Engine : 8 ECM
    ///  SG_ Gear : 0|8@1+ (1,0) [0|5] "" *
    ///
    /// VAL_ 100 Gear 0 "Park" 1 "Drive" ;"#)?;
    /// let value_descriptions_list = dbc.value_descriptions();
    /// assert_eq!(value_descriptions_list.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn value_descriptions(&self) -> &ValueDescriptionsList {
        &self.value_descriptions
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn value_descriptions_for_signal(
        &self,
        message_id: u32,
        signal_name: &str,
    ) -> Option<&ValueDescriptions> {
        self.value_descriptions.for_signal(message_id, signal_name)
    }

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
            BA_, BA_DEF_, BA_DEF_DEF_, BO_, BO_TX_BU_, BS_, BU_, CM_, EV_, NS_, SG_, SIG_GROUP_,
            SIG_VALTYPE_, VAL_, VAL_TABLE_, VERSION,
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
                        return Err(Error::Nodes(lang::NODES_TOO_MANY));
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

                    loop {
                        parser.skip_newlines_and_spaces();
                        if parser.starts_with(crate::SG_.as_bytes()) {
                            if let Some(next_byte) = parser.peek_byte_at(3) {
                                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t') {
                                    if signals_array.len() >= MAX_SIGNALS_PER_MESSAGE {
                                        return Err(Error::Receivers(
                                            lang::SIGNAL_RECEIVERS_TOO_MANY,
                                        ));
                                    }
                                    // Signal::parse expects SG_ keyword, which we've already verified with starts_with
                                    let signal = Signal::parse(&mut parser)?;
                                    signals_array.push(signal).map_err(|_| {
                                        Error::Receivers(lang::SIGNAL_RECEIVERS_TOO_MANY)
                                    })?;
                                    continue;
                                }
                            }
                        }
                        break;
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
                        .map_err(|_| Error::Message(lang::NODES_TOO_MANY))?;
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
        let value_descriptions_list = {
            let mut map: BTreeMap<(Option<u32>, std::string::String), ValueDescriptions> =
                BTreeMap::new();
            for (message_id, signal_name, entries) in value_descriptions_buffer {
                let key = (message_id, signal_name);
                let value_descriptions = ValueDescriptions::from_slice(&entries);
                map.insert(key, value_descriptions);
            }
            ValueDescriptionsList::from_map(map)
        };

        // Convert messages buffer to slice for validation and construction
        let messages_slice: &[Message] = messages_buffer.as_slice();

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        #[cfg(feature = "std")]
        Self::validate(&nodes, messages_slice, Some(&value_descriptions_list)).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(crate::error::lang::MESSAGE_ERROR_PREFIX)
            })
        })?;
        #[cfg(not(feature = "std"))]
        Self::validate(&nodes, messages_slice).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(crate::error::lang::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Construct directly (validation already done)
        let messages = MessageList::new(messages_slice)?;
        Ok(Self {
            version,
            nodes,
            messages,
            #[cfg(feature = "std")]
            value_descriptions: value_descriptions_list,
        })
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
    pub fn parse_bytes(data: &[u8]) -> Result<Dbc> {
        let content =
            core::str::from_utf8(data).map_err(|_e| Error::Expected(lang::INVALID_UTF8))?;
        Dbc::parse(content)
    }

    /// Serialize this DBC to a DBC format string
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let dbc_string = dbc.to_dbc_string();
    /// // The string can be written to a file or used elsewhere
    /// assert!(dbc_string.contains("VERSION"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
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
            result.push_str(&message.to_string_full());
        }

        result
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Dbc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{Error, error::lang};

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
            Error::Message(msg) => {
                assert!(msg.contains(lang::DUPLICATE_MESSAGE_ID));
            }
            _ => panic!("Expected Error::Message"),
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
            Error::Message(msg) => {
                assert!(msg.contains(lang::SENDER_NOT_IN_NODES));
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_parse_empty_file() {
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
    fn test_parse_bytes_invalid_utf8() {
        // Invalid UTF-8 sequence
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        let result = Dbc::parse_bytes(invalid_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected(msg) => {
                assert_eq!(msg, lang::INVALID_UTF8);
            }
            _ => panic!("Expected Error::Expected with INVALID_UTF8"),
        }
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
    #[cfg(feature = "std")]
    fn test_save_multiple_messages() {
        // Use parsing instead of builders
        let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        let saved = dbc.to_dbc_string();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    // Note: Builder limit tests have been moved to dbc_builder.rs
    // These tests require building many messages programmatically, which is builder functionality

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
        let result = Dbc::parse(data);
        assert!(result.is_err());
    }

    // Tests that require std (for to_dbc_string, value_descriptions, etc.)
    #[cfg(feature = "std")]
    mod tests_std {
        use super::*;

        #[test]
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
        fn test_save_basic() {
            // Use parsing instead of builders
            let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
"#;
            let dbc = Dbc::parse(dbc_content).unwrap();

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
        fn test_save_multiple_messages() {
            // Use parsing instead of builders
            let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;
            let dbc = Dbc::parse(dbc_content).unwrap();
            let saved = dbc.to_dbc_string();

            // Verify both messages are present
            assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
            assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
            assert!(saved.contains("SG_ RPM"));
            assert!(saved.contains("SG_ Pressure"));
        }

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

            // Verify signal exists
            let signal = message.signals().find("Signal").unwrap();
            assert_eq!(signal.name(), "Signal");
            assert_eq!(signal.start_bit(), 32);
            assert_eq!(signal.length(), 8);
            assert_eq!(signal.unit(), Some("Gear"));

            // Verify value descriptions are parsed and accessible
            let value_descriptions = dbc
                .value_descriptions_for_signal(100, "Signal")
                .expect("Value descriptions should exist for signal");

            // Verify all value mappings
            assert_eq!(value_descriptions.get(0xFFFFFFFF), Some("Reverse")); // -1 as u32
            assert_eq!(value_descriptions.get(0), Some("Neutral"));
            assert_eq!(value_descriptions.get(1), Some("First"));
            assert_eq!(value_descriptions.get(2), Some("Second"));
            assert_eq!(value_descriptions.get(3), Some("Third"));
            assert_eq!(value_descriptions.get(4), Some("Fourth"));

            // Verify non-existent values return None
            assert_eq!(value_descriptions.get(5), None);
            assert_eq!(value_descriptions.get(99), None);

            // Verify we can iterate over all value descriptions
            let iter = value_descriptions.iter();
            let mut entries: std::vec::Vec<_> = iter.collect();
            entries.sort_by_key(|(v, _)| *v);

            assert_eq!(entries.len(), 6);
            // After sorting by key (u64), 0xFFFFFFFF (4294967295) comes after smaller values
            assert_eq!(entries[0], (0, "Neutral".to_string()));
            assert_eq!(entries[1], (1, "First".to_string()));
            assert_eq!(entries[2], (2, "Second".to_string()));
            assert_eq!(entries[3], (3, "Third".to_string()));
            assert_eq!(entries[4], (4, "Fourth".to_string()));
            assert_eq!(entries[5], (0xFFFFFFFF, "Reverse".to_string()));
        }

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
    }
}
