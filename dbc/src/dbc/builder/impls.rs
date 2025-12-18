use super::DbcBuilder;
use crate::{
    Dbc, ExtendedMultiplexingBuilder, MessageBuilder, NodesBuilder, Receivers, ReceiversBuilder,
    SignalBuilder, ValueDescriptionsBuilder, VersionBuilder,
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

        Self {
            version,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DbcBuilder;
    use crate::{Dbc, ExtendedMultiplexingBuilder, MessageBuilder};

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
