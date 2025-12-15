use std::collections::BTreeMap;

use crate::dbc::validation::Validation;
use crate::error::Result;
use crate::{
    Dbc, Message, MessageBuilder, MessageList, Nodes, NodesBuilder, Receivers, ReceiversBuilder,
    SignalBuilder, ValueDescriptionsBuilder, Version, VersionBuilder,
    attributes::{Attribute, AttributeDefault, AttributeDefinition},
    comment::Comment,
    value_table::ValueTable,
};

// ExtendedMultiplexing is not exported, so we need a local type for the builder
#[derive(Debug, Clone, PartialEq)]
struct ExtendedMultiplexingBuilder {
    message_id: u32,
    signal_name: String,
    multiplexer_switch: String,
    value_ranges: Vec<(u8, u8)>,
}

/// Builder for constructing `Dbc` instances programmatically.
///
/// This builder allows you to create DBC files without parsing from a string.
/// It requires the `std` feature to be enabled.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{DbcBuilder, NodesBuilder, MessageBuilder, SignalBuilder, VersionBuilder};
///
/// let nodes = NodesBuilder::new()
///     .add_node("ECM")
///     .add_node("TCM");
///
/// let signal = SignalBuilder::new()
///     .name("RPM")
///     .start_bit(0)
///     .length(16);
///
/// let message = MessageBuilder::new()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .add_signal(signal);
///
/// let dbc = DbcBuilder::new()
///     .version(VersionBuilder::new().version("1.0"))
///     .nodes(nodes)
///     .add_message(message)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct DbcBuilder {
    version: VersionBuilder,
    nodes: NodesBuilder,
    messages: Vec<MessageBuilder>,
    value_descriptions: BTreeMap<(Option<u32>, String), ValueDescriptionsBuilder>,
    attributes: Vec<AttributeDefinition>,
    attribute_defaults: Vec<AttributeDefault>,
    attribute_values: Vec<Attribute>,
    comments: Vec<Comment>,
    value_tables: Vec<ValueTable>,
    extended_multiplexing: Vec<ExtendedMultiplexingBuilder>,
}

impl DbcBuilder {
    /// Creates a new empty `DbcBuilder`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, VersionBuilder, NodesBuilder, MessageBuilder};
    ///
    /// let dbc = DbcBuilder::new()
    ///     .version(VersionBuilder::new().version("1.0"))
    ///     .nodes(NodesBuilder::new().add_node("ECM"))
    ///     .add_message(MessageBuilder::new()
    ///         .id(512)
    ///         .name("Brake")
    ///         .dlc(4)
    ///         .sender("ECM"))
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `DbcBuilder` from an existing `Dbc`.
    ///
    /// This allows you to modify an existing DBC file by creating a builder
    /// initialized with all data from the provided DBC.
    ///
    /// # Arguments
    ///
    /// * `dbc` - The existing `Dbc` to create a builder from
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{Dbc, DbcBuilder, MessageBuilder};
    ///
    /// let original = Dbc::parse(r#"VERSION "1.0"\nBU_: ECM\n"#)?;
    /// let modified = DbcBuilder::from_dbc(&original)
    ///     .add_message(MessageBuilder::new().id(256).name("Msg").dlc(8).sender("ECM"))
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn from_dbc(dbc: &Dbc) -> Self {
        // Convert version to builder (store builder, not final type)
        let version = if let Some(v) = dbc.version() {
            VersionBuilder::new().version(v.as_str())
        } else {
            VersionBuilder::new()
        };

        // Convert nodes to builder (store builder, not final type)
        // Note: We unwrap here because we're converting from a valid Dbc, so names should already fit MAX_NAME_SIZE
        let nodes = {
            let mut builder = NodesBuilder::new();
            for node in dbc.nodes().iter() {
                // Convert compat::String to std::string::String for the builder
                let node_str = node.to_string();
                // Should never fail for valid Dbc - unwrap is safe
                builder = builder.add_node(node_str);
            }
            builder
        };

        // Convert messages to builders (store builders, not final types)
        let messages: Vec<MessageBuilder> = dbc
            .messages()
            .iter()
            .map(|msg| {
                let mut msg_builder = MessageBuilder::new()
                    .id(msg.id())
                    .name(msg.name())
                    .dlc(msg.dlc())
                    .sender(msg.sender());

                // Convert signals using SignalBuilder
                for sig in msg.signals().iter() {
                    let mut sig_builder = SignalBuilder::new()
                        .name(sig.name())
                        .start_bit(sig.start_bit())
                        .length(sig.length())
                        .byte_order(sig.byte_order())
                        .unsigned(sig.is_unsigned())
                        .factor(sig.factor())
                        .offset(sig.offset())
                        .min(sig.min())
                        .max(sig.max());

                    if let Some(unit) = sig.unit() {
                        sig_builder = sig_builder.unit(unit);
                    }

                    // Convert receivers using ReceiversBuilder
                    let receivers_builder = match sig.receivers() {
                        Receivers::Broadcast => ReceiversBuilder::new().broadcast(),
                        Receivers::None => ReceiversBuilder::new().none(),
                        Receivers::Nodes(nodes) => {
                            let mut rb = ReceiversBuilder::new();
                            // nodes is Vec<String<{MAX_NAME_SIZE}>>, iterate directly
                            for receiver in nodes.iter() {
                                // receiver is &String<{MAX_NAME_SIZE}>, clone it
                                let receiver_str = receiver.clone();
                                // Should never fail for valid Dbc - unwrap is safe
                                rb = rb.add_node(receiver_str);
                            }
                            rb
                        }
                    };
                    sig_builder = sig_builder.receivers(receivers_builder);

                    msg_builder = msg_builder.add_signal(sig_builder);
                }

                msg_builder
            })
            .collect();

        // Convert value descriptions from Dbc to builder format (store builders, not final types)
        let mut value_descriptions: BTreeMap<(Option<u32>, String), ValueDescriptionsBuilder> =
            BTreeMap::new();
        for ((message_id, signal_name), vd) in dbc.value_descriptions().iter() {
            // Store as String and ValueDescriptionsBuilder (no leak)
            let mut builder = ValueDescriptionsBuilder::new();
            for (value, desc) in vd.iter() {
                builder = builder.add_entry(value, desc);
            }
            value_descriptions.insert((message_id, signal_name.to_string()), builder);
        }

        // Copy attributes, comments, value tables, and extended multiplexing from parsed Dbc
        // Note: These are not exposed via public API yet, so we'll need to access them directly
        // For now, we'll initialize them as empty and add methods to populate them
        let attributes = std::vec::Vec::new(); // TODO: Copy from dbc when getters are available
        let attribute_defaults = std::vec::Vec::new();
        let attribute_values = std::vec::Vec::new();
        let comments = std::vec::Vec::new();
        let value_tables = std::vec::Vec::new();
        let extended_multiplexing = std::vec::Vec::new();

        Self {
            version,
            nodes,
            messages,
            value_descriptions,
            attributes,
            attribute_defaults,
            attribute_values,
            comments,
            value_tables,
            extended_multiplexing,
        }
    }

    /// Sets the version for the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, VersionBuilder};
    ///
    /// let vb = VersionBuilder::new().version("1.0");
    /// let builder = DbcBuilder::new()
    ///     .version(vb);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn version(mut self, version: VersionBuilder) -> Self {
        self.version = version;
        self
    }

    /// Sets the nodes (ECUs) for the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, NodesBuilder};
    ///
    /// let builder = DbcBuilder::new()
    ///     .nodes(NodesBuilder::new().add_node("ECM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn nodes(mut self, nodes: NodesBuilder) -> Self {
        self.nodes = nodes;
        self
    }

    /// Adds a message to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, MessageBuilder};
    ///
    /// let message = MessageBuilder::new()
    ///     .id(256)
    ///     .name("EngineData")
    ///     .dlc(8)
    ///     .sender("ECM");
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_message(message);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn add_message(mut self, message: MessageBuilder) -> Self {
        self.messages.push(message);
        self
    }

    /// Adds multiple messages to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, MessageBuilder};
    ///
    /// let msg1 = MessageBuilder::new().id(256).name("Msg1").dlc(8).sender("ECM");
    /// let msg2 = MessageBuilder::new().id(512).name("Msg2").dlc(4).sender("TCM");
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_messages(vec![msg1, msg2]);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn add_messages(mut self, messages: impl IntoIterator<Item = MessageBuilder>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Clears all messages from the builder.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new()
    ///     .clear_messages();
    /// ```
    #[must_use]
    pub fn clear_messages(mut self) -> Self {
        self.messages.clear();
        self
    }

    /// Adds a comment to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, comment::Comment, comment::CommentObjectType};
    ///
    /// let comment = Comment::new(
    ///     CommentObjectType::General,
    ///     None,
    ///     None,
    ///     "This is a general comment".to_string(),
    /// );
    /// let builder = DbcBuilder::new()
    ///     .add_comment(comment);
    /// ```
    #[must_use]
    pub fn add_comment(mut self, comment: Comment) -> Self {
        self.comments.push(comment);
        self
    }

    /// Adds a value table to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, value_table::ValueTable};
    ///
    /// let value_table = ValueTable::new(
    ///     "GearState".to_string(),
    ///     vec![(0, "Park".to_string()), (1, "Drive".to_string())],
    /// );
    /// let builder = DbcBuilder::new()
    ///     .add_value_table(value_table);
    /// ```
    #[must_use]
    pub fn add_value_table(mut self, value_table: ValueTable) -> Self {
        self.value_tables.push(value_table);
        self
    }

    /// Adds an attribute definition to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, attributes::{AttributeDefinition, AttributeObjectType, AttributeValueType}};
    ///
    /// let attr_def = AttributeDefinition::new(
    ///     AttributeObjectType::Message,
    ///     "GenMsgCycleTime".to_string(),
    ///     AttributeValueType::Int(0, 65535),
    /// );
    /// let builder = DbcBuilder::new()
    ///     .add_attribute_definition(attr_def);
    /// ```
    #[must_use]
    pub fn add_attribute_definition(mut self, attr_def: AttributeDefinition) -> Self {
        self.attributes.push(attr_def);
        self
    }

    /// Adds an attribute default value to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, attributes::{AttributeDefault, AttributeValue}};
    ///
    /// let attr_default = AttributeDefault::new(
    ///     "GenMsgCycleTime".to_string(),
    ///     AttributeValue::Int(100),
    /// );
    /// let builder = DbcBuilder::new()
    ///     .add_attribute_default(attr_default);
    /// ```
    #[must_use]
    pub fn add_attribute_default(mut self, attr_default: AttributeDefault) -> Self {
        self.attribute_defaults.push(attr_default);
        self
    }

    /// Adds an attribute value assignment to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, attributes::{Attribute, AttributeObjectType, AttributeValue}};
    ///
    /// let attr = Attribute::new(
    ///     "GenMsgCycleTime".to_string(),
    ///     AttributeObjectType::Message,
    ///     None,
    ///     Some(256),
    ///     AttributeValue::Int(50),
    /// );
    /// let builder = DbcBuilder::new()
    ///     .add_attribute_value(attr);
    /// ```
    #[must_use]
    pub fn add_attribute_value(mut self, attr: Attribute) -> Self {
        self.attribute_values.push(attr);
        self
    }

    /// Adds extended multiplexing information to the DBC file.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID
    /// * `signal_name` - The name of the multiplexed signal
    /// * `multiplexer_switch` - The name of the multiplexer switch signal
    /// * `value_ranges` - Vector of (min, max) value ranges
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_extended_multiplexing(256, "SignalA".to_string(), "Mux".to_string(), vec![(0, 10), (20, 30)]);
    /// ```
    #[must_use]
    pub fn add_extended_multiplexing(
        mut self,
        message_id: u32,
        signal_name: String,
        multiplexer_switch: String,
        value_ranges: Vec<(u8, u8)>,
    ) -> Self {
        self.extended_multiplexing.push(ExtendedMultiplexingBuilder {
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        });
        self
    }

    /// Validates the builder without constructing the `Dbc`.
    ///
    /// This method performs all validation checks. Note that this consumes
    /// the builder. If you want to keep the builder after validation, call
    /// `build()` instead and check the result.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new();
    /// if builder.validate().is_err() {
    ///     // Handle validation error
    /// }
    /// ```
    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<()> {
        // Build and validate (extract_fields builds everything)
        // We need to call extract_fields from the impl<'a> block
        // Since validate doesn't need the lifetime, we can just build and drop
        let (_version, nodes, messages, value_descriptions) = {
            let version = self.version.build()?;
            let nodes = self.nodes.build()?;
            let messages: std::vec::Vec<Message> = self
                .messages
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<std::vec::Vec<_>>>()?;
            let mut value_descriptions_map: BTreeMap<
                (Option<u32>, String),
                crate::ValueDescriptions,
            > = BTreeMap::new();
            for ((message_id, signal_name), vd_builder) in self.value_descriptions {
                let vd = vd_builder.build()?;
                value_descriptions_map.insert((message_id, signal_name), vd);
            }
            let value_descriptions =
                crate::dbc::ValueDescriptionsList::from_map(value_descriptions_map);
            (version, nodes, messages, value_descriptions)
        };

        // Validate messages
        Validation::validate(&nodes, &messages, Some(&value_descriptions))?;

        Ok(())
    }

    #[allow(clippy::type_complexity)] // Builder needs to return all fields
    fn extract_fields(
        self,
    ) -> Result<(
        Version,
        Nodes,
        MessageList,
        crate::dbc::ValueDescriptionsList,
        std::vec::Vec<AttributeDefinition>,
        std::vec::Vec<AttributeDefault>,
        std::vec::Vec<Attribute>,
        std::vec::Vec<Comment>,
        std::vec::Vec<ValueTable>,
        std::vec::Vec<ExtendedMultiplexingBuilder>,
    )> {
        // Build version
        let version = self.version.build()?;

        // Build nodes (allow empty - DBC spec allows empty BU_: line)
        let nodes = self.nodes.build()?;

        // Build messages
        // Collect into a temporary Vec first, then convert to slice for MessageList::new
        let messages_vec: std::vec::Vec<Message> = self
            .messages
            .into_iter()
            .map(|builder| builder.build())
            .collect::<Result<std::vec::Vec<_>>>()?;
        let messages = MessageList::new(&messages_vec)?;

        // Build value descriptions
        let mut value_descriptions_map: BTreeMap<(Option<u32>, String), crate::ValueDescriptions> =
            BTreeMap::new();
        for ((message_id, signal_name), vd_builder) in self.value_descriptions {
            let vd = vd_builder.build()?;
            value_descriptions_map.insert((message_id, signal_name), vd);
        }
        let value_descriptions =
            crate::dbc::ValueDescriptionsList::from_map(value_descriptions_map);

        Ok((
            version,
            nodes,
            messages,
            value_descriptions,
            self.attributes,
            self.attribute_defaults,
            self.attribute_values,
            self.comments,
            self.value_tables,
            self.extended_multiplexing,
        ))
    }

    /// Builds the `Dbc` from the builder.
    ///
    /// This method validates all fields and constructs the `Dbc` instance.
    /// Returns an error if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, VersionBuilder, NodesBuilder};
    ///
    /// let dbc = DbcBuilder::new()
    ///     .version(VersionBuilder::new().version("1.0"))
    ///     .nodes(NodesBuilder::new().add_node("ECM"))
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<Dbc> {
        let (
            version,
            nodes,
            messages,
            value_descriptions,
            attributes,
            attribute_defaults,
            attribute_values,
            comments,
            value_tables,
            extended_multiplexing_builders,
        ) = self.extract_fields()?;
        // Validate before construction
        // Get slice from MessageList for validation
        let messages_slice: std::vec::Vec<Message> = messages.iter().cloned().collect();
        Validation::validate(&nodes, &messages_slice, Some(&value_descriptions))?;

        // Convert ExtendedMultiplexingBuilder to ExtendedMultiplexing
        // ExtendedMultiplexing is defined in dbc.rs (as dbc_impl module) but not exported
        // Since we're in the same module, we can access it via super::dbc_impl
        use crate::ExtendedMultiplexing;
        use crate::compat::String;
        let extended_multiplexing: std::vec::Vec<ExtendedMultiplexing> =
            extended_multiplexing_builders
                .into_iter()
                .map(|em| {
                    let signal_name =
                        String::try_from(em.signal_name.as_str()).unwrap_or_else(|_| String::new());
                    let multiplexer_switch = String::try_from(em.multiplexer_switch.as_str())
                        .unwrap_or_else(|_| String::new());
                    let mut value_ranges = crate::compat::Vec::<(u64, u64), 64>::new();
                    for (min, max) in em.value_ranges {
                        let _ = value_ranges.push((min as u64, max as u64));
                    }
                    ExtendedMultiplexing::new(
                        em.message_id,
                        signal_name,
                        multiplexer_switch,
                        value_ranges,
                    )
                })
                .collect();

        Ok(Dbc::new_with_extras(
            Some(version),
            nodes,
            messages,
            value_descriptions,
            attributes,
            attribute_defaults,
            attribute_values,
            comments,
            value_tables,
            extended_multiplexing,
            std::collections::BTreeMap::new(), // signal_value_types - not yet supported in builder
            std::vec::Vec::new(),              // signal_types - not yet supported in builder
            std::vec::Vec::new(), // signal_type_references - not yet supported in builder
            std::vec::Vec::new(), // signal_type_values - not yet supported in builder
            None,                 // bit_timing - not yet supported in builder
            std::vec::Vec::new(), // signal_groups - not yet supported in builder
            std::vec::Vec::new(), // message_transmitters - not yet supported in builder
            std::vec::Vec::new(), // signal_type_attribute_definitions - not yet supported in builder
            std::vec::Vec::new(), // signal_type_attributes - not yet supported in builder
            std::vec::Vec::new(), // environment_variables - not yet supported in builder
            std::vec::Vec::new(), // environment_variable_data - not yet supported in builder
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{
        ByteOrder, Dbc, MessageBuilder, NodesBuilder, ReceiversBuilder, SignalBuilder,
        VersionBuilder,
    };

    #[test]
    fn test_dbc_builder_valid() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
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
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 1);
        assert_eq!(dbc.messages().at(0).unwrap().id(), 256);
    }

    #[test]
    fn test_dbc_builder_missing_version() {
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
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
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        let result = DbcBuilder::new().nodes(nodes).add_message(message).build();
        // VersionBuilder now allows empty version, so this should succeed
        assert!(result.is_ok());
        let dbc = result.unwrap();
        assert_eq!(dbc.version().unwrap().as_str(), "");
    }

    #[test]
    fn test_dbc_builder_missing_nodes() {
        // Empty nodes are now allowed per DBC spec
        // When nodes are empty, sender validation is skipped
        let version = VersionBuilder::new().version("1.0");
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
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        // Building without nodes should succeed (empty nodes allowed)
        let result = DbcBuilder::new().version(version).add_message(message).build();
        assert!(result.is_ok());
        let dbc = result.unwrap();
        assert!(dbc.nodes().is_empty());
    }

    #[test]
    fn test_dbc_builder_add_messages() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());
        let signal2 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());
        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1);
        let message2 = MessageBuilder::new()
            .id(512)
            .name("BrakeData")
            .dlc(4)
            .sender("ECM")
            .add_signal(signal2);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_messages(vec![message1, message2])
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 2);
    }

    #[test]
    fn test_dbc_builder_messages() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let signal1 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());
        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal1);

        let signal2 = SignalBuilder::new()
            .name("RPM")
            .start_bit(0)
            .length(16)
            .byte_order(ByteOrder::BigEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());
        let message2 = MessageBuilder::new()
            .id(512)
            .name("EngineData2")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal2);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message1)
            .add_message(message2)
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 2);
    }

    #[test]
    fn test_dbc_builder_clear_messages() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
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
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .clear_messages()
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_dbc_builder_validate_missing_version() {
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
        // VersionBuilder now allows empty version, so validation should succeed
        let result = DbcBuilder::new().nodes(nodes).validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dbc_builder_validate_missing_nodes() {
        // Empty nodes are now allowed per DBC spec
        let version = VersionBuilder::new().version("1.0");
        let result = DbcBuilder::new().version(version).validate();
        // Validation should succeed with empty nodes
        assert!(result.is_ok());
    }

    #[test]
    fn test_dbc_builder_validate_valid() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
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
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        // validate() consumes the builder, so we can't use it after
        // But we can check it doesn't error
        let builder = DbcBuilder::new().version(version).nodes(nodes).add_message(message);
        let result = builder.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dbc_builder_from_dbc() {
        // Parse an existing DBC
        let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let original_dbc = Dbc::parse(dbc_content).unwrap();

        // Create builder from existing DBC
        let modified_dbc = DbcBuilder::from_dbc(&original_dbc)
            .add_message(MessageBuilder::new().id(512).name("Brake").dlc(4).sender("TCM"))
            .build()
            .unwrap();

        // Verify original data is preserved
        assert_eq!(modified_dbc.version().map(|v| v.as_str()), Some("1.0"));
        assert_eq!(modified_dbc.nodes().len(), 2);
        assert!(modified_dbc.nodes().contains("ECM"));
        assert!(modified_dbc.nodes().contains("TCM"));

        // Verify original message is present
        assert_eq!(modified_dbc.messages().len(), 2);
        assert!(modified_dbc.messages().iter().any(|m| m.id() == 256));
        assert!(modified_dbc.messages().iter().any(|m| m.id() == 512));

        // Verify original message's signal is preserved
        let engine_msg = modified_dbc.messages().iter().find(|m| m.id() == 256).unwrap();
        assert_eq!(engine_msg.signals().len(), 1);
        assert_eq!(engine_msg.signals().at(0).unwrap().name(), "RPM");
    }

    #[test]
    fn test_dbc_builder_from_dbc_empty() {
        // Parse a minimal DBC
        let dbc_content = r#"VERSION "1.0"

BU_:
"#;
        let original_dbc = Dbc::parse(dbc_content).unwrap();

        // Create builder from existing DBC
        let modified_dbc = DbcBuilder::from_dbc(&original_dbc)
            .add_message(MessageBuilder::new().id(256).name("Test").dlc(8).sender("ECM"))
            .build()
            .unwrap();

        // Verify version is preserved
        assert_eq!(modified_dbc.version().map(|v| v.as_str()), Some("1.0"));
        // Empty nodes are preserved
        assert!(modified_dbc.nodes().is_empty());
        // New message is added
        assert_eq!(modified_dbc.messages().len(), 1);
    }
}
