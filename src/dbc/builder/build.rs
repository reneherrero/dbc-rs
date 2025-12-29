use super::DbcBuilder;
use crate::{
    BitTiming, Dbc, ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, MAX_MESSAGES, MAX_NAME_SIZE,
    Message, Nodes, Result, Version,
    compat::{BTreeMap, String, Vec as CompatVec},
    dbc::{Messages, Validate},
};

impl DbcBuilder {
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
        let (_version, nodes, messages, value_descriptions, extended_multiplexing) = {
            let version = self.version.build()?;
            let nodes = self.nodes.build()?;
            let messages: std::vec::Vec<Message> = self
                .messages
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<std::vec::Vec<_>>>()?;
            let mut value_descriptions_map: BTreeMap<
                (Option<u32>, String<{ MAX_NAME_SIZE }>),
                crate::value_descriptions::ValueDescriptions,
                { MAX_MESSAGES },
            > = BTreeMap::new();
            for ((message_id, signal_name), vd_builder) in self.value_descriptions {
                let vd: crate::value_descriptions::ValueDescriptions = vd_builder.build()?;
                let compat_signal_name = String::try_from(signal_name.as_str())
                    .map_err(|_| crate::Error::Validation(crate::Error::MAX_NAME_SIZE_EXCEEDED))?;
                let _ = value_descriptions_map.insert((message_id, compat_signal_name), vd);
            }
            let value_descriptions = crate::dbc::ValueDescriptionsMap::new(value_descriptions_map);

            // Build extended multiplexing entries
            let extended_multiplexing_vec: std::vec::Vec<ExtendedMultiplexing> = self
                .extended_multiplexing
                .into_iter()
                .map(|builder| builder.build())
                .collect::<Result<std::vec::Vec<_>>>()?;

            (
                version,
                nodes,
                messages,
                value_descriptions,
                extended_multiplexing_vec,
            )
        };

        // Validate messages and extended multiplexing
        Validate::validate(
            &nodes,
            &messages,
            Some(&value_descriptions),
            Some(&extended_multiplexing),
        )?;

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn extract_fields(
        self,
    ) -> Result<(
        Version,
        Option<BitTiming>,
        Nodes,
        Messages,
        crate::dbc::ValueDescriptionsMap,
        CompatVec<ExtendedMultiplexing, { MAX_EXTENDED_MULTIPLEXING }>,
        Option<std::string::String>,
    )> {
        // Build version
        let version = self.version.build()?;

        // Build bit timing if present
        let bit_timing = match self.bit_timing {
            Some(builder) => {
                let bt = builder.build()?;
                if bt.is_empty() { None } else { Some(bt) }
            }
            None => None,
        };

        // Build nodes (allow empty - DBC spec allows empty BU_: line)
        let nodes = self.nodes.build()?;

        // Build messages
        // Collect into a temporary Vec first, then convert to slice for Messages::new
        let messages_vec: std::vec::Vec<Message> = self
            .messages
            .into_iter()
            .map(|builder| builder.build())
            .collect::<Result<std::vec::Vec<_>>>()?;
        let messages = Messages::new(&messages_vec)?;

        // Build value descriptions
        let mut value_descriptions_map: BTreeMap<
            (Option<u32>, String<{ MAX_NAME_SIZE }>),
            crate::value_descriptions::ValueDescriptions,
            { MAX_MESSAGES },
        > = BTreeMap::new();
        for ((message_id, signal_name), vd_builder) in self.value_descriptions {
            let vd: crate::value_descriptions::ValueDescriptions = vd_builder.build()?;
            let compat_signal_name = String::try_from(signal_name.as_str())
                .map_err(|_| crate::Error::Validation(crate::Error::MAX_NAME_SIZE_EXCEEDED))?;
            let _ = value_descriptions_map.insert((message_id, compat_signal_name), vd);
        }
        let value_descriptions = crate::dbc::ValueDescriptionsMap::new(value_descriptions_map);

        // Build extended multiplexing entries
        let extended_multiplexing_vec: std::vec::Vec<ExtendedMultiplexing> = self
            .extended_multiplexing
            .into_iter()
            .map(|builder| builder.build())
            .collect::<Result<std::vec::Vec<_>>>()?;

        // Convert to compat Vec
        let mut extended_multiplexing: CompatVec<
            ExtendedMultiplexing,
            { MAX_EXTENDED_MULTIPLEXING },
        > = CompatVec::new();
        for ext_mux in extended_multiplexing_vec {
            extended_multiplexing
                .push(ext_mux)
                .map_err(|_| crate::Error::expected("too many extended multiplexing entries"))?;
        }

        Ok((
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            self.comment,
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
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            comment,
        ) = self.extract_fields()?;
        // Validate before construction
        // Get slice from Messages for validation
        let messages_slice: std::vec::Vec<Message> = messages.iter().cloned().collect();
        let extended_multiplexing_slice: std::vec::Vec<ExtendedMultiplexing> =
            extended_multiplexing.iter().cloned().collect();
        Validate::validate(
            &nodes,
            &messages_slice,
            Some(&value_descriptions),
            Some(&extended_multiplexing_slice),
        )?;
        Ok(Dbc::new(
            Some(version),
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            comment.map(|c| c.into()),
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{
        ByteOrder, ExtendedMultiplexingBuilder, MessageBuilder, NodesBuilder, ReceiversBuilder,
        SignalBuilder, ValueDescriptionsBuilder, VersionBuilder,
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

    // ========================================================================
    // Value Descriptions Tests
    // ========================================================================

    #[test]
    fn test_dbc_builder_with_value_descriptions() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
        let signal = SignalBuilder::new()
            .name("Gear")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(5.0)
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("Transmission")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        let value_desc = ValueDescriptionsBuilder::new()
            .add_entry(0, "Park")
            .add_entry(1, "Reverse")
            .add_entry(2, "Neutral")
            .add_entry(3, "Drive");

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_value_description(256, "Gear", value_desc)
            .build()
            .unwrap();

        // Verify value descriptions are present
        let vd = dbc.value_descriptions_for_signal(256, "Gear").unwrap();
        assert_eq!(vd.get(0), Some("Park"));
        assert_eq!(vd.get(1), Some("Reverse"));
        assert_eq!(vd.get(2), Some("Neutral"));
        assert_eq!(vd.get(3), Some("Drive"));
    }

    #[test]
    fn test_dbc_builder_value_descriptions_message_not_found() {
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

        // Add value description for non-existent message
        let value_desc = ValueDescriptionsBuilder::new().add_entry(0, "Off").add_entry(1, "On");

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_value_description(999, "RPM", value_desc) // Message 999 doesn't exist
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_value_descriptions_signal_not_found() {
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

        // Add value description for non-existent signal
        let value_desc = ValueDescriptionsBuilder::new().add_entry(0, "Off").add_entry(1, "On");

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_value_description(256, "NonExistentSignal", value_desc)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_validate_with_value_descriptions() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);
        let signal = SignalBuilder::new()
            .name("Gear")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(5.0)
            .receivers(ReceiversBuilder::new().none());
        let message = MessageBuilder::new()
            .id(256)
            .name("Transmission")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal);

        let value_desc = ValueDescriptionsBuilder::new().add_entry(0, "Park").add_entry(1, "Drive");

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_value_description(256, "Gear", value_desc)
            .validate();

        assert!(result.is_ok());
    }

    // ========================================================================
    // Extended Multiplexing Tests
    // ========================================================================

    #[test]
    fn test_dbc_builder_with_extended_multiplexing() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        // Create a multiplexer signal
        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        // Create a multiplexed signal
        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(0.1)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a);

        // Create extended multiplexing entry
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5)
            .add_value_range(10, 15);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 1);
        assert_eq!(dbc.extended_multiplexing().len(), 1);

        let ext_mux = &dbc.extended_multiplexing()[0];
        assert_eq!(ext_mux.message_id(), 500);
        assert_eq!(ext_mux.signal_name(), "SignalA");
        assert_eq!(ext_mux.multiplexer_switch(), "Mux1");
        assert_eq!(ext_mux.value_ranges().len(), 2);
        assert_eq!(ext_mux.value_ranges()[0], (0, 5));
        assert_eq!(ext_mux.value_ranges()[1], (10, 15));
    }

    #[test]
    fn test_dbc_builder_extended_multiplexing_message_not_found() {
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

        // Create extended multiplexing referencing non-existent message
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(999) // Non-existent message
            .signal_name("RPM")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_extended_multiplexing_signal_not_found() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
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
            .add_signal(mux_signal);

        // Create extended multiplexing referencing non-existent signal
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(256)
            .signal_name("NonExistent") // Non-existent signal
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_extended_multiplexing_switch_not_found() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let signal = SignalBuilder::new()
            .name("SignalA")
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

        // Create extended multiplexing referencing non-existent multiplexer switch
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(256)
            .signal_name("SignalA")
            .multiplexer_switch("NonExistentMux") // Non-existent switch
            .add_value_range(0, 5);

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_extended_multiplexing_invalid_range() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(0.1)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a);

        // Create extended multiplexing with invalid range (min > max)
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(10, 5); // Invalid: min > max

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_dbc_builder_validate_with_extended_multiplexing() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(0.1)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a);

        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let result = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_dbc_builder_multiple_extended_multiplexing() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_b = SignalBuilder::new()
            .name("SignalB")
            .start_bit(24)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a)
            .add_signal(signal_b);

        let ext_mux_a = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let ext_mux_b = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalB")
            .multiplexer_switch("Mux1")
            .add_value_range(10, 15);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexings(vec![ext_mux_a, ext_mux_b])
            .build()
            .unwrap();

        assert_eq!(dbc.extended_multiplexing().len(), 2);
        assert_eq!(dbc.extended_multiplexing()[0].signal_name(), "SignalA");
        assert_eq!(dbc.extended_multiplexing()[1].signal_name(), "SignalB");
    }

    #[test]
    fn test_dbc_builder_clear_extended_multiplexing() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(16)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(0.1)
            .offset(0.0)
            .min(0.0)
            .max(100.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a);

        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_extended_multiplexing(ext_mux)
            .clear_extended_multiplexing()
            .build()
            .unwrap();

        assert_eq!(dbc.extended_multiplexing().len(), 0);
    }

    // ========================================================================
    // Combined Tests (Value Descriptions + Extended Multiplexing)
    // ========================================================================

    #[test]
    fn test_dbc_builder_with_value_descriptions_and_extended_multiplexing() {
        let version = VersionBuilder::new().version("1.0");
        let nodes = NodesBuilder::new().add_nodes(["ECM"]);

        let mux_signal = SignalBuilder::new()
            .name("Mux1")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let signal_a = SignalBuilder::new()
            .name("SignalA")
            .start_bit(16)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(3.0)
            .receivers(ReceiversBuilder::new().none());

        let message = MessageBuilder::new()
            .id(500)
            .name("MuxMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(mux_signal)
            .add_signal(signal_a);

        let value_desc = ValueDescriptionsBuilder::new()
            .add_entry(0, "Off")
            .add_entry(1, "Low")
            .add_entry(2, "Medium")
            .add_entry(3, "High");

        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("SignalA")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5);

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .add_value_description(500, "SignalA", value_desc)
            .add_extended_multiplexing(ext_mux)
            .build()
            .unwrap();

        // Verify value descriptions
        let vd = dbc.value_descriptions_for_signal(500, "SignalA").unwrap();
        assert_eq!(vd.get(0), Some("Off"));
        assert_eq!(vd.get(3), Some("High"));

        // Verify extended multiplexing
        assert_eq!(dbc.extended_multiplexing().len(), 1);
        assert_eq!(dbc.extended_multiplexing()[0].signal_name(), "SignalA");
    }
}
