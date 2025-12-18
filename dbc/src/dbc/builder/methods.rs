use super::DbcBuilder;
use crate::{
    ExtendedMultiplexingBuilder, MessageBuilder, NodesBuilder, ValueDescriptionsBuilder,
    VersionBuilder,
};

impl DbcBuilder {
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{
        ByteOrder, MessageBuilder, NodesBuilder, ReceiversBuilder, SignalBuilder, VersionBuilder,
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
}
