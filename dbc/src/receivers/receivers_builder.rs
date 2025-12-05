use super::Receivers;
use crate::{error::Error, error::Result};

#[derive(Debug, Default)]
pub struct ReceiversBuilder {
    is_broadcast: bool,
    is_none: bool,
    nodes: Vec<String>,
}

impl ReceiversBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn broadcast(mut self) -> Self {
        self.is_broadcast = true;
        self.is_none = false;
        self.nodes.clear();
        self
    }

    #[must_use]
    pub fn none(mut self) -> Self {
        self.is_none = true;
        self.is_broadcast = false;
        self.nodes.clear();
        self
    }

    #[must_use]
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        self.is_broadcast = false;
        self.is_none = false;
        self.nodes.push(node.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.is_broadcast = false;
        self.is_none = false;
        self.nodes.extend(nodes.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    #[must_use]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self.is_broadcast = false;
        self.is_none = false;
        self
    }

    pub fn build(self) -> Result<Receivers<'static>> {
        if self.is_broadcast {
            Ok(Receivers::new_broadcast())
        } else if self.is_none || self.nodes.is_empty() {
            Ok(Receivers::new_none())
        } else {
            // Convert owned Strings to static references by leaking Box<str>
            let mut node_refs: Vec<&'static str> = Vec::new();
            for s in self.nodes {
                let boxed: Box<str> = s.into_boxed_str();
                node_refs.push(Box::leak(boxed));
            }
            // Validate before construction
            const MAX_RECEIVER_NODES: usize = 64;
            if node_refs.len() > MAX_RECEIVER_NODES {
                return Err(Error::Signal(String::from(
                    crate::error::messages::SIGNAL_RECEIVERS_TOO_MANY,
                )));
            }
            Ok(Receivers::new_nodes(&node_refs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_receivers_builder_broadcast() {
        let receivers = ReceiversBuilder::new().broadcast().build().unwrap();
        assert_eq!(receivers, Receivers::Broadcast);
    }

    #[test]
    fn test_receivers_builder_none() {
        let receivers = ReceiversBuilder::new().none().build().unwrap();
        assert_eq!(receivers, Receivers::None);
    }

    #[test]
    fn test_receivers_builder_empty() {
        let receivers = ReceiversBuilder::new().build().unwrap();
        assert_eq!(receivers, Receivers::None);
    }

    #[test]
    fn test_receivers_builder_single_node() {
        let receivers = ReceiversBuilder::new().add_node("TCM").build().unwrap();
        match receivers {
            Receivers::Nodes(_, count) => assert_eq!(count, 1),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_multiple_nodes() {
        let receivers = ReceiversBuilder::new()
            .add_node("TCM")
            .add_node("BCM")
            .add_node("ECM")
            .build()
            .unwrap();
        match receivers {
            Receivers::Nodes(_, count) => assert_eq!(count, 3),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_too_many() {
        let mut builder = ReceiversBuilder::new();
        for i in 0..65 {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(_) => {}
            _ => panic!("Expected Signal error"),
        }
    }
}
