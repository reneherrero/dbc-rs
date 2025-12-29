use crate::{
    BitTiming, Dbc, Error, ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, MAX_MESSAGES,
    MAX_NODES, MAX_SIGNALS_PER_MESSAGE, Message, Nodes, Parser, Result, Signal, ValueDescriptions,
    Version,
    compat::{BTreeMap, Comment, Name, ValueDescEntries, Vec},
    dbc::{Messages, Validate, ValueDescriptionsMap},
};
#[cfg(feature = "attributes")]
use crate::{
    MAX_ATTRIBUTE_DEFINITIONS, MAX_ATTRIBUTE_VALUES,
    attribute::{
        AttributeDefinition, AttributeTarget, AttributeValue,
        parse::{parse_attribute_assignment, parse_attribute_default},
    },
    dbc::{AttributeDefaultsMap, AttributeDefinitionsMap, AttributeValuesMap},
};

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
        let mut bit_timing: Option<BitTiming> = None;
        let mut nodes: Option<Nodes> = None;

        // Type aliases for parsing buffers
        type ValueDescBufferEntry = (Option<u32>, Name, ValueDescEntries);
        type ValueDescBuffer = Vec<ValueDescBufferEntry, { MAX_MESSAGES }>;
        type ExtMuxBuffer = Vec<ExtendedMultiplexing, { MAX_EXTENDED_MULTIPLEXING }>;

        // Comment buffers - CM_ entries can appear anywhere in the file
        // so we collect them first and apply after parsing messages
        type MessageCommentBuffer = Vec<(u32, Comment), { MAX_MESSAGES }>;
        // Signal comments: (message_id, signal_name, comment)
        type SignalCommentBuffer = Vec<(u32, Name, Comment), { MAX_MESSAGES * 4 }>;

        let mut value_descriptions_buffer: ValueDescBuffer = ValueDescBuffer::new();
        let mut extended_multiplexing_buffer: ExtMuxBuffer = ExtMuxBuffer::new();

        // Comment buffers
        let mut db_comment: Option<Comment> = None;
        // Node comments: (node_name, comment)
        type NodeCommentBuffer = Vec<(Name, Comment), { MAX_NODES }>;
        let mut node_comments_buffer: NodeCommentBuffer = NodeCommentBuffer::new();
        let mut message_comments_buffer: MessageCommentBuffer = MessageCommentBuffer::new();
        let mut signal_comments_buffer: SignalCommentBuffer = SignalCommentBuffer::new();

        // Attribute buffers - BA_DEF_, BA_DEF_DEF_, BA_ entries can appear anywhere
        #[cfg(feature = "attributes")]
        type AttrDefBuffer = Vec<AttributeDefinition, { MAX_ATTRIBUTE_DEFINITIONS }>;
        #[cfg(feature = "attributes")]
        type AttrDefaultBuffer = Vec<(Name, AttributeValue), { MAX_ATTRIBUTE_DEFINITIONS }>;
        #[cfg(feature = "attributes")]
        type AttrValueBuffer =
            Vec<(Name, AttributeTarget, AttributeValue), { MAX_ATTRIBUTE_VALUES }>;

        #[cfg(feature = "attributes")]
        let mut attribute_definitions_buffer: AttrDefBuffer = AttrDefBuffer::new();
        #[cfg(feature = "attributes")]
        let mut attribute_defaults_buffer: AttrDefaultBuffer = AttrDefaultBuffer::new();
        #[cfg(feature = "attributes")]
        let mut attribute_values_buffer: AttrValueBuffer = AttrValueBuffer::new();

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
                Err(Error::UnexpectedEof { .. }) => break,
                Err(Error::Expected { .. }) => {
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
                    let line = parser.line();
                    parser
                        .expect(crate::NS_.as_bytes())
                        .map_err(|_| Error::expected_at("Failed to consume NS_ keyword", line))?;
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
                BS_ => {
                    // Parse bit timing section (usually empty)
                    let parsed = BitTiming::parse(&mut parser)?;
                    // Only store if not empty (has actual values)
                    if !parsed.is_empty() {
                        bit_timing = Some(parsed);
                    }
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[cfg(feature = "attributes")]
                BA_DEF_ => {
                    // Parse attribute definition: BA_DEF_ [object_type] "attr_name" value_type ;
                    let _ = parser.expect(BA_DEF_.as_bytes()).ok();
                    if let Some(def) = AttributeDefinition::parse(&mut parser) {
                        let _ = attribute_definitions_buffer.push(def);
                    }
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[cfg(feature = "attributes")]
                BA_DEF_DEF_ => {
                    // Parse attribute default: BA_DEF_DEF_ "attr_name" value ;
                    let _ = parser.expect(BA_DEF_DEF_.as_bytes()).ok();
                    if let Some((name, value)) = parse_attribute_default(&mut parser) {
                        let _ = attribute_defaults_buffer.push((name, value));
                    }
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[cfg(feature = "attributes")]
                BA_ => {
                    // Parse attribute value: BA_ "attr_name" [object_ref] value ;
                    let _ = parser.expect(BA_.as_bytes()).ok();
                    if let Some((name, target, value)) = parse_attribute_assignment(&mut parser) {
                        let _ = attribute_values_buffer.push((name, target, value));
                    }
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[cfg(not(feature = "attributes"))]
                BA_DEF_ | BA_DEF_DEF_ | BA_ => {
                    // Skip attribute entries when feature is disabled
                    let _ = parser.expect(keyword.as_bytes()).ok();
                    parser.skip_to_end_of_line();
                    continue;
                }
                VAL_TABLE_ | SIG_GROUP_ | SIG_VALTYPE_ | EV_ | BO_TX_BU_ => {
                    // TODO: These DBC sections are recognized but not parsed:
                    //   VAL_TABLE_   - Global value tables (rarely used)
                    //   SIG_GROUP_   - Signal groups (rarely used)
                    //   SIG_VALTYPE_ - Signal extended value types: float/double (medium priority)
                    //   EV_          - Environment variables (rarely used)
                    //   BO_TX_BU_    - Multiple message transmitters (rarely used)
                    //
                    // Not yet recognized (rarely used):
                    //   ENVVAR_DATA_, SGTYPE_, BA_DEF_SGTYPE_, BA_SGTYPE_, SIG_TYPE_REF_,
                    //   BA_DEF_REL_, BA_REL_, BA_DEF_DEF_REL_, BU_SG_REL_, BU_EV_REL_, BU_BO_REL_
                    //
                    // Consume keyword then skip to end of line
                    let _ = parser.expect(keyword.as_bytes()).ok();
                    parser.skip_to_end_of_line();
                    continue;
                }
                CM_ => {
                    // Parse CM_ comment entry
                    // Formats:
                    //   CM_ "general comment";
                    //   CM_ BU_ node_name "comment";
                    //   CM_ BO_ message_id "comment";
                    //   CM_ SG_ message_id signal_name "comment";
                    let _ = parser.expect(crate::CM_.as_bytes()).ok();
                    parser.skip_newlines_and_spaces();

                    // Determine comment type by peeking next token
                    if parser.starts_with(b"\"") {
                        // General database comment: CM_ "string";
                        if parser.expect(b"\"").is_ok() {
                            if let Ok(comment_bytes) = parser.take_until_quote(false, 1024) {
                                if let Ok(comment_str) = core::str::from_utf8(comment_bytes) {
                                    if let Ok(comment) = Comment::try_from(comment_str) {
                                        db_comment = Some(comment);
                                    }
                                }
                            }
                        }
                        parser.skip_to_end_of_line();
                    } else if parser.starts_with(BU_.as_bytes()) {
                        // Node comment: CM_ BU_ node_name "string";
                        let _ = parser.expect(BU_.as_bytes()).ok();
                        parser.skip_newlines_and_spaces();
                        if let Ok(node_name_bytes) = parser.parse_identifier() {
                            if let Ok(node_name) = Name::try_from(node_name_bytes) {
                                parser.skip_newlines_and_spaces();
                                if parser.expect(b"\"").is_ok() {
                                    if let Ok(comment_bytes) = parser.take_until_quote(false, 1024)
                                    {
                                        if let Ok(comment_str) = core::str::from_utf8(comment_bytes)
                                        {
                                            if let Ok(comment) = Comment::try_from(comment_str) {
                                                let _ =
                                                    node_comments_buffer.push((node_name, comment));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        parser.skip_to_end_of_line();
                    } else if parser.starts_with(BO_.as_bytes()) {
                        // Message comment: CM_ BO_ message_id "string";
                        let _ = parser.expect(BO_.as_bytes()).ok();
                        parser.skip_newlines_and_spaces();
                        if let Ok(message_id) = parser.parse_u32() {
                            parser.skip_newlines_and_spaces();
                            if parser.expect(b"\"").is_ok() {
                                if let Ok(comment_bytes) = parser.take_until_quote(false, 1024) {
                                    if let Ok(comment_str) = core::str::from_utf8(comment_bytes) {
                                        if let Ok(comment) = Comment::try_from(comment_str) {
                                            let _ =
                                                message_comments_buffer.push((message_id, comment));
                                        }
                                    }
                                }
                            }
                        }
                        parser.skip_to_end_of_line();
                    } else if parser.starts_with(SG_.as_bytes()) {
                        // Signal comment: CM_ SG_ message_id signal_name "string";
                        let _ = parser.expect(SG_.as_bytes()).ok();
                        parser.skip_newlines_and_spaces();
                        if let Ok(message_id) = parser.parse_u32() {
                            parser.skip_newlines_and_spaces();
                            if let Ok(signal_name_bytes) = parser.parse_identifier() {
                                if let Ok(signal_name) = Name::try_from(signal_name_bytes) {
                                    parser.skip_newlines_and_spaces();
                                    if parser.expect(b"\"").is_ok() {
                                        if let Ok(comment_bytes) =
                                            parser.take_until_quote(false, 1024)
                                        {
                                            if let Ok(comment_str) =
                                                core::str::from_utf8(comment_bytes)
                                            {
                                                if let Ok(comment) = Comment::try_from(comment_str)
                                                {
                                                    let _ = signal_comments_buffer.push((
                                                        message_id,
                                                        signal_name,
                                                        comment,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        parser.skip_to_end_of_line();
                    } else {
                        // Unknown comment type, skip
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                SG_MUL_VAL_ => {
                    // Consume SG_MUL_VAL_ keyword
                    let line = parser.line();
                    parser.expect(SG_MUL_VAL_.as_bytes()).map_err(|_| {
                        Error::expected_at("Failed to consume SG_MUL_VAL_ keyword", line)
                    })?;

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
                        Ok(name) => match Name::try_from(name) {
                            Ok(s) => s,
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        },
                        Err(_) => {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                    };
                    // Parse value-description pairs
                    let mut entries: ValueDescEntries = ValueDescEntries::new();
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
                        let description = match core::str::from_utf8(description_bytes)
                            .ok()
                            .and_then(|s| Name::try_from(s).ok())
                        {
                            Some(desc) => desc,
                            None => {
                                parser.skip_to_end_of_line();
                                break;
                            }
                        };
                        let _ = entries.push((value, description));
                    }
                    if !entries.is_empty() {
                        let _ = value_descriptions_buffer.push((message_id, signal_name, entries));
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
                        return Err(parser.err_nodes(Error::NODES_TOO_MANY));
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
                            Err(Error::UnexpectedEof { .. }) => break,
                            Err(_) => break, // Not a keyword, no more signals
                        };

                        // Only process SG_ signals here (SG_MUL_VAL_ is handled in main loop)
                        if keyword != SG_ {
                            break; // Not a signal, exit signal parsing loop
                        }

                        // Check limit before parsing
                        if signals_array.len() >= MAX_SIGNALS_PER_MESSAGE {
                            return Err(parser.err_message(Error::MESSAGE_TOO_MANY_SIGNALS));
                        }

                        // Parse signal - Signal::parse consumes SG_ itself
                        match Signal::parse(&mut parser) {
                            Ok(signal) => {
                                signals_array.push(signal).map_err(|_| {
                                    parser.err_receivers(Error::SIGNAL_RECEIVERS_TOO_MANY)
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
                        .map_err(|_| parser.err_message(Error::NODES_TOO_MANY))?;
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
        let mut nodes = nodes.unwrap_or_default();

        // Apply node comments to nodes (consume buffer to avoid cloning)
        for (node_name, comment) in node_comments_buffer {
            nodes.set_node_comment(node_name.as_str(), comment);
        }

        // If no version was parsed, default to empty version
        let version = version.or_else(|| {
            static EMPTY_VERSION: &[u8] = b"VERSION \"\"";
            let mut parser = Parser::new(EMPTY_VERSION).ok()?;
            Version::parse(&mut parser).ok()
        });

        // Build value descriptions map for storage in Dbc (consume buffer to avoid cloning)
        let value_descriptions_map = {
            let mut map: BTreeMap<(Option<u32>, Name), ValueDescriptions, { MAX_MESSAGES }> =
                BTreeMap::new();
            for (message_id, signal_name, entries) in value_descriptions_buffer {
                let key = (message_id, signal_name);
                let value_descriptions = ValueDescriptions::new(entries);
                let _ = map.insert(key, value_descriptions);
            }
            ValueDescriptionsMap::new(map)
        };

        // Build attribute maps from buffers (consume buffers to avoid cloning)
        #[cfg(feature = "attributes")]
        let (attribute_definitions, attribute_defaults, attribute_values) = {
            use crate::attribute::AttributeDefinitions;

            let attribute_definitions = AttributeDefinitionsMap::from_vec({
                let mut defs: AttributeDefinitions = AttributeDefinitions::new();
                for def in attribute_definitions_buffer {
                    let _ = defs.push(def);
                }
                defs
            });

            let attribute_defaults = AttributeDefaultsMap::from_map({
                let mut map: BTreeMap<Name, AttributeValue, { MAX_ATTRIBUTE_DEFINITIONS }> =
                    BTreeMap::new();
                for (name, value) in attribute_defaults_buffer {
                    let _ = map.insert(name, value);
                }
                map
            });

            let attribute_values = AttributeValuesMap::from_map({
                let mut map: BTreeMap<
                    (Name, AttributeTarget),
                    AttributeValue,
                    { MAX_ATTRIBUTE_VALUES },
                > = BTreeMap::new();
                for (name, target, value) in attribute_values_buffer {
                    let _ = map.insert((name, target), value);
                }
                map
            });

            (attribute_definitions, attribute_defaults, attribute_values)
        };

        // Apply comments to messages and signals (consume buffers to avoid cloning)
        // Message comments are applied by matching message_id
        for (message_id, comment) in message_comments_buffer {
            for msg in messages_buffer.iter_mut() {
                if msg.id() == message_id || msg.id_with_flag() == message_id {
                    msg.set_comment(comment);
                    break;
                }
            }
        }

        // Signal comments are applied by matching (message_id, signal_name)
        for (message_id, signal_name, comment) in signal_comments_buffer {
            for msg in messages_buffer.iter_mut() {
                if msg.id() == message_id || msg.id_with_flag() == message_id {
                    if let Some(signal) = msg.signals_mut().find_mut(signal_name.as_str()) {
                        signal.set_comment(comment);
                    }
                    break;
                }
            }
        }

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        Validate::validate(
            &nodes,
            messages_buffer.as_slice(),
            Some(&value_descriptions_map),
            Some(extended_multiplexing_buffer.as_slice()),
        )
        .map_err(|e| {
            crate::error::map_val_error(e, Error::message, || {
                Error::message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Construct directly from owned buffer (avoids cloning all messages)
        let messages = Messages::from_vec(messages_buffer)?;

        #[cfg(feature = "attributes")]
        return Ok(Dbc::new(
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions_map,
            extended_multiplexing_buffer,
            db_comment,
            attribute_definitions,
            attribute_defaults,
            attribute_values,
        ));

        #[cfg(not(feature = "attributes"))]
        Ok(Dbc::new(
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions_map,
            extended_multiplexing_buffer,
            db_comment,
        ))
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
            core::str::from_utf8(data).map_err(|_e| Error::expected(Error::INVALID_UTF8))?;
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
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

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
            Error::Message { msg, .. } => {
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
            Error::Message { msg, .. } => {
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
            Error::UnexpectedEof { .. } => {
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
            Error::Expected { msg, .. } => {
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
        // id() returns the raw CAN ID without the extended flag
        assert_eq!(msg.id(), 0x494); // Raw extended ID
        assert!(msg.is_extended()); // is_extended() tells if it's a 29-bit ID
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
        // id() returns the raw 29-bit CAN ID without the extended flag
        assert_eq!(msg.id(), 0x1FFFFFFF);
        assert!(msg.is_extended());
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

    #[test]
    fn test_error_includes_line_number() {
        // Test that parsing errors include line numbers
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ invalid EngineData : 8 ECM
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // The error should have line information
        assert!(err.line().is_some(), "Error should include line number");
    }

    // ============================================================================
    // CM_ Comment Parsing Tests (Section 14 of SPECIFICATIONS.md)
    // ============================================================================

    /// Test parsing general database comment: CM_ "string";
    #[test]
    fn test_parse_cm_database_comment() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM

CM_ "This is the database comment";
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.comment(), Some("This is the database comment"));
    }

    /// Test parsing node comment: CM_ BU_ node_name "string";
    #[test]
    fn test_parse_cm_node_comment() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM

CM_ BU_ ECM "Engine Control Module";
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.node_comment("ECM"), Some("Engine Control Module"));
    }

    /// Test parsing message comment: CM_ BO_ message_id "string";
    #[test]
    fn test_parse_cm_message_comment() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM

CM_ BO_ 256 "Engine status message";
"#;
        let dbc = Dbc::parse(data).unwrap();
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.comment(), Some("Engine status message"));
    }

    /// Test parsing signal comment: CM_ SG_ message_id signal_name "string";
    #[test]
    fn test_parse_cm_signal_comment() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"

CM_ SG_ 256 RPM "Engine rotations per minute";
"#;
        let dbc = Dbc::parse(data).unwrap();
        let msg = dbc.messages().iter().next().unwrap();
        let signal = msg.signals().find("RPM").unwrap();
        assert_eq!(signal.comment(), Some("Engine rotations per minute"));
    }

    /// Test multiple comments in one file
    #[test]
    fn test_parse_cm_multiple_comments() {
        let data = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"

BO_ 512 Trans : 8 TCM
 SG_ Gear : 0|8@1+ (1,0) [0|6] ""

CM_ "Vehicle CAN database";
CM_ BU_ ECM "Engine Control Module";
CM_ BU_ TCM "Transmission Control Module";
CM_ BO_ 256 "Engine status message";
CM_ BO_ 512 "Transmission status";
CM_ SG_ 256 RPM "Engine rotations per minute";
CM_ SG_ 512 Gear "Current gear position";
"#;
        let dbc = Dbc::parse(data).unwrap();

        // Database comment
        assert_eq!(dbc.comment(), Some("Vehicle CAN database"));

        // Node comments
        assert_eq!(dbc.node_comment("ECM"), Some("Engine Control Module"));
        assert_eq!(dbc.node_comment("TCM"), Some("Transmission Control Module"));

        // Message comments
        let engine = dbc.messages().iter().find(|m| m.id() == 256).unwrap();
        let trans = dbc.messages().iter().find(|m| m.id() == 512).unwrap();
        assert_eq!(engine.comment(), Some("Engine status message"));
        assert_eq!(trans.comment(), Some("Transmission status"));

        // Signal comments
        let rpm = engine.signals().find("RPM").unwrap();
        let gear = trans.signals().find("Gear").unwrap();
        assert_eq!(rpm.comment(), Some("Engine rotations per minute"));
        assert_eq!(gear.comment(), Some("Current gear position"));
    }

    /// Test CM_ appearing before the entities they describe
    #[test]
    fn test_parse_cm_before_entity() {
        let data = r#"VERSION "1.0"

BU_: ECM

CM_ BO_ 256 "Engine status message";
CM_ SG_ 256 RPM "Engine RPM";

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.comment(), Some("Engine status message"));
        let signal = msg.signals().find("RPM").unwrap();
        assert_eq!(signal.comment(), Some("Engine RPM"));
    }

    /// Test multiple CM_ entries for same entity - last wins
    #[test]
    fn test_parse_cm_last_wins() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"

CM_ BO_ 256 "First message comment";
CM_ BO_ 256 "Second message comment";
CM_ SG_ 256 RPM "First signal comment";
CM_ SG_ 256 RPM "Second signal comment";
CM_ BU_ ECM "First node comment";
CM_ BU_ ECM "Second node comment";
"#;
        let dbc = Dbc::parse(data).unwrap();

        // Last comment wins for each entity
        let msg = dbc.messages().iter().next().unwrap();
        assert_eq!(msg.comment(), Some("Second message comment"));
        let signal = msg.signals().find("RPM").unwrap();
        assert_eq!(signal.comment(), Some("Second signal comment"));
        assert_eq!(dbc.node_comment("ECM"), Some("Second node comment"));
    }

    /// Test comment round-trip (parse -> serialize -> parse)
    #[test]
    #[cfg(feature = "std")]
    fn test_parse_cm_round_trip() {
        let data = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"

CM_ "Database comment";
CM_ BU_ ECM "Engine Control Module";
CM_ BO_ 256 "Engine status message";
CM_ SG_ 256 RPM "Engine rotations per minute";
"#;
        let dbc = Dbc::parse(data).unwrap();

        // Serialize and re-parse
        let serialized = dbc.to_dbc_string();
        let dbc2 = Dbc::parse(&serialized).unwrap();

        // Verify comments are preserved
        assert_eq!(dbc2.comment(), Some("Database comment"));
        assert_eq!(dbc2.node_comment("ECM"), Some("Engine Control Module"));
        let msg = dbc2.messages().iter().next().unwrap();
        assert_eq!(msg.comment(), Some("Engine status message"));
        let signal = msg.signals().find("RPM").unwrap();
        assert_eq!(signal.comment(), Some("Engine rotations per minute"));
    }

    /// Test CM_ serialization in output
    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_cm_comments() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"

CM_ "Database comment";
CM_ BU_ ECM "Engine Control Module";
CM_ BO_ 256 "Engine status";
CM_ SG_ 256 RPM "RPM signal";
"#;
        let dbc = Dbc::parse(data).unwrap();
        let serialized = dbc.to_dbc_string();

        // Verify CM_ lines are present in output
        assert!(serialized.contains("CM_ \"Database comment\";"));
        assert!(serialized.contains("CM_ BU_ ECM \"Engine Control Module\";"));
        assert!(serialized.contains("CM_ BO_ 256 \"Engine status\";"));
        assert!(serialized.contains("CM_ SG_ 256 RPM \"RPM signal\";"));
    }
}
