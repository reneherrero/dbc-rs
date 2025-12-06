use crate::{
    Dbc, Message, Nodes, Version,
    error::{Error, Result, messages},
};

#[derive(Debug, Default)]
pub struct DbcBuilder {
    version: Option<Version<'static>>,
    nodes: Option<Nodes<'static>>,
    messages: Vec<Message<'static>>,
}

impl DbcBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn version(mut self, version: Version<'static>) -> Self {
        self.version = Some(version);
        self
    }

    #[must_use]
    pub fn nodes(mut self, nodes: Nodes<'static>) -> Self {
        self.nodes = Some(nodes);
        self
    }

    #[must_use]
    pub fn add_message(mut self, message: Message<'static>) -> Self {
        self.messages.push(message);
        self
    }

    #[must_use]
    pub fn add_messages(mut self, messages: impl IntoIterator<Item = Message<'static>>) -> Self {
        self.messages.extend(messages);
        self
    }

    #[must_use]
    pub fn messages(mut self, messages: Vec<Message<'static>>) -> Self {
        self.messages = messages;
        self
    }

    #[must_use]
    pub fn clear_messages(mut self) -> Self {
        self.messages.clear();
        self
    }

    fn extract_fields(self) -> Result<(Version<'static>, Nodes<'static>, Vec<Message<'static>>)> {
        let version = self
            .version
            .ok_or_else(|| Error::Dbc(messages::DBC_VERSION_REQUIRED.to_string()))?;
        let nodes =
            self.nodes.ok_or_else(|| Error::Dbc(messages::DBC_NODES_REQUIRED.to_string()))?;
        Ok((version, nodes, self.messages))
    }

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

        let result = DbcBuilder::new().version(version).add_message(message).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => assert!(msg.contains(lang::DBC_NODES_REQUIRED)),
            _ => panic!("Expected Dbc error"),
        }
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
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let result = DbcBuilder::new().version(version).validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Dbc(msg) => assert!(msg.contains(lang::DBC_NODES_REQUIRED)),
            _ => panic!("Expected Dbc error"),
        }
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
