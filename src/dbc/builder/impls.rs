use super::DbcBuilder;
use crate::{
    BitTimingBuilder, Dbc, ExtendedMultiplexingBuilder, MessageBuilder, NodesBuilder, Receivers,
    ReceiversBuilder, SignalBuilder, ValueDescriptionsBuilder, VersionBuilder,
};
use std::collections::BTreeMap;

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
        Self {
            version: VersionBuilder::new(),
            bit_timing: None,
            nodes: NodesBuilder::new(),
            messages: Vec::new(),
            value_descriptions: BTreeMap::new(),
            extended_multiplexing: Vec::new(),
            comment: None,
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
    #[must_use = "builder method returns modified builder"]
    pub fn version(mut self, version: VersionBuilder) -> Self {
        self.version = version;
        self
    }

    /// Sets the bit timing configuration for the DBC file.
    ///
    /// The BS_ section in DBC files specifies CAN bus timing parameters.
    /// This section is typically empty as bit timing is obsolete in modern CAN systems.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, BitTimingBuilder};
    ///
    /// let builder = DbcBuilder::new()
    ///     .bit_timing(BitTimingBuilder::new().baudrate(500000));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn bit_timing(mut self, bit_timing: BitTimingBuilder) -> Self {
        self.bit_timing = Some(bit_timing);
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
    pub fn clear_messages(mut self) -> Self {
        self.messages.clear();
        self
    }

    /// Adds value descriptions for a signal in a message.
    ///
    /// Value descriptions (VAL_) map numeric signal values to human-readable text.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The CAN message ID containing the signal
    /// * `signal_name` - The name of the signal
    /// * `value_descriptions` - The value descriptions builder
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, ValueDescriptionsBuilder};
    ///
    /// let value_desc = ValueDescriptionsBuilder::new()
    ///     .add_entry(0, "Park")
    ///     .add_entry(1, "Drive");
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_value_description(256, "Gear", value_desc);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_value_description(
        mut self,
        message_id: u32,
        signal_name: impl AsRef<str>,
        value_descriptions: ValueDescriptionsBuilder,
    ) -> Self {
        self.value_descriptions.insert(
            (Some(message_id), signal_name.as_ref().to_string()),
            value_descriptions,
        );
        self
    }

    /// Adds global value descriptions for a signal (applies to all messages with this signal).
    ///
    /// Global value descriptions (VAL_ with message_id -1) apply to signals with the given
    /// name in any message.
    ///
    /// # Arguments
    ///
    /// * `signal_name` - The name of the signal
    /// * `value_descriptions` - The value descriptions builder
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, ValueDescriptionsBuilder};
    ///
    /// let value_desc = ValueDescriptionsBuilder::new()
    ///     .add_entry(0, "Off")
    ///     .add_entry(1, "On");
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_global_value_description("Status", value_desc);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_global_value_description(
        mut self,
        signal_name: impl AsRef<str>,
        value_descriptions: ValueDescriptionsBuilder,
    ) -> Self {
        self.value_descriptions
            .insert((None, signal_name.as_ref().to_string()), value_descriptions);
        self
    }

    /// Clears all value descriptions from the builder.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new()
    ///     .clear_value_descriptions();
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn clear_value_descriptions(mut self) -> Self {
        self.value_descriptions.clear();
        self
    }

    /// Adds an extended multiplexing entry to the DBC file.
    ///
    /// Extended multiplexing (SG_MUL_VAL_) entries define which multiplexer switch
    /// values activate specific multiplexed signals, allowing for range-based activation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, ExtendedMultiplexingBuilder};
    ///
    /// let ext_mux = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500)
    ///     .signal_name("Signal_A")
    ///     .multiplexer_switch("Mux1")
    ///     .add_value_range(0, 5);
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_extended_multiplexing(ext_mux);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_extended_multiplexing(mut self, ext_mux: ExtendedMultiplexingBuilder) -> Self {
        self.extended_multiplexing.push(ext_mux);
        self
    }

    /// Adds multiple extended multiplexing entries to the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, ExtendedMultiplexingBuilder};
    ///
    /// let ext_mux1 = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500)
    ///     .signal_name("Signal_A")
    ///     .multiplexer_switch("Mux1")
    ///     .add_value_range(0, 5);
    /// let ext_mux2 = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500)
    ///     .signal_name("Signal_B")
    ///     .multiplexer_switch("Mux1")
    ///     .add_value_range(10, 15);
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_extended_multiplexings(vec![ext_mux1, ext_mux2]);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_extended_multiplexings(
        mut self,
        ext_muxes: impl IntoIterator<Item = ExtendedMultiplexingBuilder>,
    ) -> Self {
        self.extended_multiplexing.extend(ext_muxes);
        self
    }

    /// Clears all extended multiplexing entries from the builder.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new()
    ///     .clear_extended_multiplexing();
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn clear_extended_multiplexing(mut self) -> Self {
        self.extended_multiplexing.clear();
        self
    }

    /// Sets the database-level comment (general CM_ entry).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new()
    ///     .comment("CAN database for powertrain");
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn comment(mut self, comment: impl AsRef<str>) -> Self {
        self.comment = Some(comment.as_ref().to_string());
        self
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

        // Copy bit timing - convert to builder
        let bit_timing = dbc.bit_timing().map(|bt| {
            let mut builder = BitTimingBuilder::new();
            if let Some(baudrate) = bt.baudrate() {
                builder = builder.baudrate(baudrate);
            }
            if let Some(btr1) = bt.btr1() {
                builder = builder.btr1(btr1);
            }
            if let Some(btr2) = bt.btr2() {
                builder = builder.btr2(btr2);
            }
            builder
        });

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

        // Convert extended multiplexing entries to builders
        let extended_multiplexing: Vec<ExtendedMultiplexingBuilder> = dbc
            .extended_multiplexing()
            .iter()
            .map(|ext_mux| {
                let mut builder = ExtendedMultiplexingBuilder::new()
                    .message_id(ext_mux.message_id())
                    .signal_name(ext_mux.signal_name())
                    .multiplexer_switch(ext_mux.multiplexer_switch());

                // Add all value ranges
                for (min, max) in ext_mux.value_ranges() {
                    builder = builder.add_value_range(*min, *max);
                }

                builder
            })
            .collect();

        // Copy comment if present
        let comment = dbc.comment().map(|c| c.to_string());

        Self {
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            comment,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{
        ByteOrder, Dbc, ExtendedMultiplexingBuilder, MessageBuilder, NodesBuilder,
        ReceiversBuilder, SignalBuilder, VersionBuilder,
    };

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

    #[test]
    fn test_dbc_builder_from_dbc_with_extended_multiplexing() {
        // Parse a DBC with extended multiplexing
        let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 500 MuxMessage : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ SignalA m0 : 16|16@1+ (0.1,0) [0|100] ""

SG_MUL_VAL_ 500 SignalA Mux1 0-5,10-15 ;
"#;
        let original_dbc = Dbc::parse(dbc_content).unwrap();

        // Verify original has extended multiplexing
        assert_eq!(original_dbc.extended_multiplexing().len(), 1);

        // Create builder from existing DBC and modify it
        let modified_dbc = DbcBuilder::from_dbc(&original_dbc)
            .add_extended_multiplexing(
                ExtendedMultiplexingBuilder::new()
                    .message_id(500)
                    .signal_name("SignalA")
                    .multiplexer_switch("Mux1")
                    .add_value_range(20, 25),
            )
            .build()
            .unwrap();

        // Verify extended multiplexing is preserved and new one added
        assert_eq!(modified_dbc.extended_multiplexing().len(), 2);

        // Check original entry
        let original_entry = &modified_dbc.extended_multiplexing()[0];
        assert_eq!(original_entry.signal_name(), "SignalA");
        assert_eq!(original_entry.value_ranges().len(), 2);
        assert_eq!(original_entry.value_ranges()[0], (0, 5));
        assert_eq!(original_entry.value_ranges()[1], (10, 15));

        // Check new entry
        let new_entry = &modified_dbc.extended_multiplexing()[1];
        assert_eq!(new_entry.signal_name(), "SignalA");
        assert_eq!(new_entry.value_ranges().len(), 1);
        assert_eq!(new_entry.value_ranges()[0], (20, 25));
    }
}

impl Default for DbcBuilder {
    fn default() -> Self {
        Self::new()
    }
}
