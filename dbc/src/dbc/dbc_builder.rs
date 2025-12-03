use crate::{
    Dbc, Message, Nodes, Version,
    error::{Error, Result, messages},
};

#[derive(Debug, Default)]
pub struct DbcBuilder {
    version: Option<Version>,
    nodes: Option<Nodes>,
    messages: Vec<Message>,
}

impl DbcBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    #[must_use]
    pub fn nodes(mut self, nodes: Nodes) -> Self {
        self.nodes = Some(nodes);
        self
    }

    #[must_use]
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    #[must_use]
    pub fn add_messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    #[must_use]
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    #[must_use]
    pub fn clear_messages(mut self) -> Self {
        self.messages.clear();
        self
    }

    fn extract_fields(self) -> Result<(Version, Nodes, Vec<Message>)> {
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
        Dbc::validate(Some(&version), &nodes, &messages).map_err(Error::from)?;
        Ok(Self {
            version: Some(version),
            nodes: Some(nodes),
            messages,
        })
    }

    pub fn build(self) -> Result<Dbc> {
        let (version, nodes, messages) = self.extract_fields()?;
        Dbc::new(Some(version), nodes, messages)
    }
}

#[cfg(all(feature = "std", test))]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::DbcBuilder;
    use crate::{
        ByteOrder, Error, Message, Parser, Receivers, Signal, Version, error::lang,
        nodes::NodesBuilder,
    };

    #[test]
    fn test_dbc_builder_valid() {
        let mut parser = Parser::new(b"VERSION \"1.0\"").unwrap();
        let version = Version::parse(&mut parser).unwrap();
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();

        let dbc = DbcBuilder::new()
            .version(version)
            .nodes(nodes)
            .add_message(message)
            .build()
            .unwrap();

        assert_eq!(dbc.messages().len(), 1);
        assert_eq!(dbc.messages()[0].id(), 256);
    }

    #[test]
    fn test_dbc_builder_missing_version() {
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();

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
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();

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
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message1 = Message::new(256, "EngineData", 8, "ECM", vec![signal.clone()]).unwrap();
        let message2 = Message::new(512, "BrakeData", 4, "ECM", vec![signal]).unwrap();

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
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message1 = Message::new(256, "EngineData", 8, "ECM", vec![signal.clone()]).unwrap();
        let message2 = Message::new(512, "BrakeData", 4, "ECM", vec![signal]).unwrap();

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
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();

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
        let signal = Signal::new(
            "RPM",
            0,
            16,
            ByteOrder::BigEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None::<&str>,
            Receivers::None,
        )
        .unwrap();
        let message = Message::new(256, "EngineData", 8, "ECM", vec![signal]).unwrap();

        let result =
            DbcBuilder::new().version(version).nodes(nodes).add_message(message).validate();
        assert!(result.is_ok());
        // Verify we can continue building after validation
        let validated = result.unwrap();
        let dbc = validated.build().unwrap();
        assert_eq!(dbc.messages().len(), 1);
    }
}
