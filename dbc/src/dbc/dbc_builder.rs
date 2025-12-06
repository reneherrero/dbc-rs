use crate::{
    Dbc, Message, Nodes, Version,
    error::{Error, Result, messages},
};

/// Builder for constructing `Dbc` instances programmatically.
///
/// This builder allows you to create DBC files without parsing from a string.
/// It requires the `alloc` feature to be enabled.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{DbcBuilder, NodesBuilder, MessageBuilder, SignalBuilder, VersionBuilder};
///
/// let nodes = NodesBuilder::new()
///     .add_node("ECM")
///     .add_node("TCM")
///     .build()?;
///
/// let signal = SignalBuilder::new()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .build()?;
///
/// let message = MessageBuilder::new()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .add_signal(signal)
///     .build()?;
///
/// let dbc = DbcBuilder::new()
///     .version(VersionBuilder::new().version("1.0").build()?)
///     .nodes(nodes)
///     .add_message(message)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct DbcBuilder {
    version: Option<Version<'static>>,
    nodes: Option<Nodes<'static>>,
    messages: Vec<Message<'static>>,
}

impl DbcBuilder {
    /// Creates a new `DbcBuilder` with default values.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// let builder = DbcBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the version for the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, VersionBuilder};
    ///
    /// let builder = DbcBuilder::new()
    ///     .version(VersionBuilder::new().version("1.0").build()?);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn version(mut self, version: Version<'static>) -> Self {
        self.version = Some(version);
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
    ///     .nodes(NodesBuilder::new().add_node("ECM").build()?);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn nodes(mut self, nodes: Nodes<'static>) -> Self {
        self.nodes = Some(nodes);
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
    ///     .sender("ECM")
    ///     .build()?;
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_message(message);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn add_message(mut self, message: Message<'static>) -> Self {
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
    /// let messages = vec![
    ///     MessageBuilder::new().id(256).name("Msg1").dlc(8).sender("ECM").build()?,
    ///     MessageBuilder::new().id(512).name("Msg2").dlc(4).sender("TCM").build()?,
    /// ];
    ///
    /// let builder = DbcBuilder::new()
    ///     .add_messages(messages);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn add_messages(mut self, messages: impl IntoIterator<Item = Message<'static>>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Sets all messages for the DBC file, replacing any existing messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, MessageBuilder};
    ///
    /// let messages = vec![
    ///     MessageBuilder::new().id(256).name("Msg1").dlc(8).sender("ECM").build()?,
    /// ];
    ///
    /// let builder = DbcBuilder::new()
    ///     .messages(messages);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn messages(mut self, messages: Vec<Message<'static>>) -> Self {
        self.messages = messages;
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

    fn extract_fields(self) -> Result<(Version<'static>, Nodes<'static>, Vec<Message<'static>>)> {
        let version = self
            .version
            .ok_or_else(|| Error::Dbc(messages::DBC_VERSION_REQUIRED.to_string()))?;
        // Allow empty nodes (DBC spec allows empty BU_: line)
        let nodes = self.nodes.unwrap_or_default();
        Ok((version, nodes, self.messages))
    }

    /// Validates the builder without constructing the `Dbc`.
    ///
    /// This method performs all validation checks but returns the builder
    /// instead of constructing the `Dbc`. Useful for checking if the builder
    /// is valid before calling `build()`.
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
    pub fn validate(self) -> Result<Self> {
        let (version, nodes, messages) = self.extract_fields()?;
        // Convert Vec to Option array for validation (all Some)
        let messages_options: Vec<Option<Message<'static>>> =
            messages.into_iter().map(Some).collect();
        let messages_options_slice: &[Option<Message<'static>>] = &messages_options;
        Dbc::validate(
            Some(&version),
            &nodes,
            messages_options_slice,
            messages_options_slice.len(),
        )
        .map_err(|e| match e {
            crate::error::ParseError::Version(msg) => Error::Dbc(String::from(msg)),
            _ => Error::from(e),
        })?;
        Ok(Self {
            version: Some(version),
            nodes: Some(nodes),
            messages: messages_options.into_iter().map(|opt| opt.unwrap()).collect(),
        })
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
    ///     .version(VersionBuilder::new().version("1.0").build()?)
    ///     .nodes(NodesBuilder::new().add_node("ECM").build()?)
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<Dbc<'static>> {
        let (version, nodes, messages) = self.extract_fields()?;
        // Convert Vec to Option array for validation (all Some)
        let messages_options: Vec<Option<Message<'static>>> =
            messages.into_iter().map(Some).collect();
        let messages_options_slice: &[Option<Message<'static>>] = &messages_options;
        // Validate before construction
        Dbc::validate(
            Some(&version),
            &nodes,
            messages_options_slice,
            messages_options_slice.len(),
        )
        .map_err(|e| match e {
            crate::error::ParseError::Version(msg) => Error::Dbc(String::from(msg)),
            _ => Error::from(e),
        })?;
        // Convert Option array back to Vec for slice creation
        let messages: Vec<Message<'static>> =
            messages_options.into_iter().map(|opt| opt.unwrap()).collect();
        // Convert Vec to slice by leaking the boxed slice to get 'static lifetime
        let messages_boxed: Box<[Message<'static>]> = messages.into_boxed_slice();
        let messages_slice: &'static [Message<'static>] = Box::leak(messages_boxed);
        Ok(Dbc::new(Some(version), nodes, messages_slice))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{ByteOrder, Error, Parser, Version, error::lang, nodes::NodesBuilder};
    use crate::{MessageBuilder, ReceiversBuilder, SignalBuilder};

    #[test]
    fn test_dbc_builder_valid() {
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
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

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
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

        let result = DbcBuilder::new().nodes(nodes).add_message(message).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => assert!(msg.contains(lang::DBC_VERSION_REQUIRED)),
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    fn test_dbc_builder_missing_nodes() {
        // Empty nodes are now allowed per DBC spec
        // When nodes are empty, sender validation is skipped
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
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
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

        // Building without nodes should succeed (empty nodes allowed)
        let result = DbcBuilder::new().version(version).add_message(message).build();
        assert!(result.is_ok());
        let dbc = result.unwrap();
        assert!(dbc.nodes().is_empty());
    }

    #[test]
    fn test_dbc_builder_add_messages() {
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
        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build()
            .unwrap();
        let message2 = MessageBuilder::new()
            .id(512)
            .name("BrakeData")
            .dlc(4)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

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
        let message1 = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal.clone())
            .build()
            .unwrap();
        let message2 = MessageBuilder::new()
            .id(512)
            .name("BrakeData")
            .dlc(4)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .messages(vec![message1, message2])
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 2);
    }

    #[test]
    fn test_dbc_builder_clear_messages() {
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
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

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
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let result = DbcBuilder::new().nodes(nodes).validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => assert!(msg.contains(lang::DBC_VERSION_REQUIRED)),
            _ => panic!("Expected Dbc error"),
        }
    }

    #[test]
    fn test_dbc_builder_validate_missing_nodes() {
        // Empty nodes are now allowed per DBC spec
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let result = DbcBuilder::new().version(version).validate();
        // Validation should succeed with empty nodes
        assert!(result.is_ok());
    }

    #[test]
    fn test_dbc_builder_validate_valid() {
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
        let message = MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(signal)
            .build()
            .unwrap();

        let result =
            DbcBuilder::new().version(version).nodes(nodes).add_message(message).validate();
        assert!(result.is_ok());
        // Verify we can continue building after validation
        let validated = result.unwrap();
        let dbc = validated.build().unwrap();
        assert_eq!(dbc.messages().len(), 1);
    }
}
