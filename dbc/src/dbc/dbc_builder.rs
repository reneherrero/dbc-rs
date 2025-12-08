#[cfg(any(feature = "alloc", feature = "kernel"))]
use crate::compat::{Box, Vec};
use crate::{
    Dbc, Message, Nodes, Version,
    error::{Error, Result},
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
/// let dbc = DbcBuilder::new(None)
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
    /// Creates a new `DbcBuilder`.
    ///
    /// If a `Dbc` is provided, the builder is initialized with all data from it,
    /// allowing you to modify an existing DBC file. If `None` is provided, an
    /// empty builder is created.
    ///
    /// # Arguments
    ///
    /// * `dbc` - Optional reference to an existing `Dbc` to initialize from
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::DbcBuilder;
    ///
    /// // Create empty builder
    /// let builder = DbcBuilder::new(None);
    /// ```
    ///
    /// ```rust,no_run
    /// use dbc_rs::{Dbc, DbcBuilder, MessageBuilder};
    ///
    /// // Parse existing DBC
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    /// "#)?;
    ///
    /// // Create builder from existing DBC
    /// let modified = DbcBuilder::new(Some(&dbc))
    ///     .add_message(MessageBuilder::new()
    ///         .id(512)
    ///         .name("Brake")
    ///         .dlc(4)
    ///         .sender("ECM")
    ///         .build()?)
    ///     .build()?;
    ///
    /// assert_eq!(modified.messages().len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new(dbc: Option<&Dbc<'_>>) -> Self {
        match dbc {
            Some(dbc) => Self::from_dbc(dbc),
            None => Self::default(),
        }
    }

    /// Creates a `DbcBuilder` from an existing `Dbc`.
    ///
    /// This is a helper method used internally by `new()`. You can also call it
    /// directly if you prefer.
    ///
    /// # Arguments
    ///
    /// * `dbc` - The existing `Dbc` to create a builder from
    fn from_dbc(dbc: &Dbc<'_>) -> Self {
        use crate::{
            MessageBuilder, NodesBuilder, ReceiversBuilder, SignalBuilder, VersionBuilder,
        };

        // Convert version to 'static using VersionBuilder
        let version = dbc.version().map(|v| {
            VersionBuilder::new()
                .version(v.as_str())
                .build()
                .expect("Version conversion should always succeed")
        });

        // Convert nodes to 'static using NodesBuilder
        let nodes = {
            let mut builder = NodesBuilder::new();
            for node in dbc.nodes().iter() {
                builder = builder.add_node(node);
            }
            builder.build().expect("Nodes conversion should always succeed")
        };

        // Convert messages to 'static using MessageBuilder and SignalBuilder
        let messages: Vec<Message<'static>> = dbc
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
                    let receivers = match sig.receivers() {
                        crate::Receivers::Broadcast => {
                            ReceiversBuilder::new().broadcast().build().unwrap()
                        }
                        crate::Receivers::None => ReceiversBuilder::new().none().build().unwrap(),
                        crate::Receivers::Nodes(_, _) => {
                            let mut recv_builder = ReceiversBuilder::new();
                            for receiver in sig.receivers().iter() {
                                recv_builder = recv_builder.add_node(receiver);
                            }
                            recv_builder.build().unwrap()
                        }
                    };
                    sig_builder = sig_builder.receivers(receivers);

                    msg_builder = msg_builder.add_signal(
                        sig_builder.build().expect("Signal conversion should always succeed"),
                    );
                }

                msg_builder.build().expect("Message conversion should always succeed")
            })
            .collect();

        Self {
            version,
            nodes: Some(nodes),
            messages,
        }
    }

    /// Sets the version for the DBC file.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{DbcBuilder, VersionBuilder};
    ///
    /// let builder = DbcBuilder::new(None)
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
    /// let builder = DbcBuilder::new(None)
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
    /// let builder = DbcBuilder::new(None)
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
    /// let msg1 = MessageBuilder::new().id(256).name("Msg1").dlc(8).sender("ECM").build()?;
    /// let msg2 = MessageBuilder::new().id(512).name("Msg2").dlc(4).sender("TCM").build()?;
    ///
    /// let builder = DbcBuilder::new(None)
    ///     .add_message(msg1)
    ///     .add_message(msg2);
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
    /// let msg = MessageBuilder::new().id(256).name("Msg1").dlc(8).sender("ECM").build()?;
    ///
    /// let builder = DbcBuilder::new(None)
    ///     .add_message(msg);
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
    /// let builder = DbcBuilder::new(None)
    ///     .clear_messages();
    /// ```
    #[must_use]
    pub fn clear_messages(mut self) -> Self {
        self.messages.clear();
        self
    }

    fn extract_fields(self) -> Result<(Version<'static>, Nodes<'static>, Vec<Message<'static>>)> {
        let version = self.version.ok_or(Error::dbc(crate::error::lang::DBC_VERSION_REQUIRED))?;
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
    /// let builder = DbcBuilder::new(None);
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
            crate::error::ParseError::Message(msg) => Error::message(msg),
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
    /// let dbc = DbcBuilder::new(None)
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
            crate::error::ParseError::Message(msg) => Error::message(msg),
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
    use crate::{ByteOrder, Dbc, Error, Parser, Version, error::lang};
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use crate::{MessageBuilder, NodesBuilder, ReceiversBuilder, SignalBuilder};
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use alloc::vec;

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

        let dbc = DbcBuilder::new(None)
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

        let result = DbcBuilder::new(None).nodes(nodes).add_message(message).build();
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
        let result = DbcBuilder::new(None).version(version).add_message(message).build();
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

        let dbc = DbcBuilder::new(None)
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

        let dbc = DbcBuilder::new(None)
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

        let dbc = DbcBuilder::new(None)
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
        let result = DbcBuilder::new(None).nodes(nodes).validate();
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
        let result = DbcBuilder::new(None).version(version).validate();
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

        let result = DbcBuilder::new(None)
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .validate();
        assert!(result.is_ok());
        // Verify we can continue building after validation
        let validated = result.unwrap();
        let dbc = validated.build().unwrap();
        assert_eq!(dbc.messages().len(), 1);
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
        let modified_dbc = DbcBuilder::new(Some(&original_dbc))
            .add_message(
                MessageBuilder::new()
                    .id(512)
                    .name("Brake")
                    .dlc(4)
                    .sender("TCM")
                    .build()
                    .unwrap(),
            )
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
        let modified_dbc = DbcBuilder::new(Some(&original_dbc))
            .add_message(
                MessageBuilder::new().id(256).name("Test").dlc(8).sender("ECM").build().unwrap(),
            )
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
