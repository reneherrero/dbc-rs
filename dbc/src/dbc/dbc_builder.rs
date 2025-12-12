use std::collections::BTreeMap;

use crate::error::Result;
use crate::{
    Dbc, Message, MessageBuilder, MessageList, Nodes, NodesBuilder, Receivers, ReceiversBuilder,
    SignalBuilder, ValueDescriptionsBuilder, Version, VersionBuilder,
};

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

        Self {
            version,
            nodes,
            messages,
            value_descriptions,
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
        Dbc::validate(&nodes, &messages, Some(&value_descriptions))?;

        Ok(())
    }

    fn extract_fields(
        self,
    ) -> Result<(
        Version,
        Nodes,
        MessageList,
        crate::dbc::ValueDescriptionsList,
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

        Ok((version, nodes, messages, value_descriptions))
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
        let (version, nodes, messages, value_descriptions) = self.extract_fields()?;
        // Validate before construction
        // Get slice from MessageList for validation
        let messages_slice: std::vec::Vec<Message> = messages.iter().cloned().collect();
        Dbc::validate(&nodes, &messages_slice, Some(&value_descriptions))?;
        Ok(Dbc::new(Some(version), nodes, messages, value_descriptions))
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
