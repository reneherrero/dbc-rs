use super::DbcBuilder;
use crate::{
    Dbc, ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, Message, Nodes, Result, Version,
    dbc::{Messages, Validate},
};
use std::collections::BTreeMap;

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
                crate::value_descriptions::ValueDescriptions,
            > = BTreeMap::new();
            for ((message_id, signal_name), vd_builder) in self.value_descriptions {
                let vd: crate::value_descriptions::ValueDescriptions = vd_builder.build()?;
                value_descriptions_map.insert((message_id, signal_name), vd);
            }
            let value_descriptions =
                crate::dbc::ValueDescriptionsMap::from_map(value_descriptions_map);
            (version, nodes, messages, value_descriptions)
        };

        // Validate messages
        Validate::validate(&nodes, &messages, Some(&value_descriptions))?;

        Ok(())
    }

    fn extract_fields(
        self,
    ) -> Result<(Version, Nodes, Messages, crate::dbc::ValueDescriptionsMap)> {
        // Build version
        let version = self.version.build()?;

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
            (Option<u32>, String),
            crate::value_descriptions::ValueDescriptions,
        > = BTreeMap::new();
        for ((message_id, signal_name), vd_builder) in self.value_descriptions {
            let vd: crate::value_descriptions::ValueDescriptions = vd_builder.build()?;
            value_descriptions_map.insert((message_id, signal_name), vd);
        }
        let value_descriptions = crate::dbc::ValueDescriptionsMap::from_map(value_descriptions_map);

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
        // Get slice from Messages for validation
        let messages_slice: std::vec::Vec<Message> = messages.iter().cloned().collect();
        Validate::validate(&nodes, &messages_slice, Some(&value_descriptions))?;
        // TODO: Add extended multiplexing
        let extended_multiplexing: crate::compat::Vec<
            ExtendedMultiplexing,
            { MAX_EXTENDED_MULTIPLEXING },
        > = crate::compat::Vec::new();
        Ok(Dbc::new(
            Some(version),
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
        ))
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
}
