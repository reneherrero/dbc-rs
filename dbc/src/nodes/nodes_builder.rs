use crate::{error::Result, nodes::Nodes};

#[derive(Debug, Default)]
pub struct NodesBuilder {
    nodes: Vec<String>,
}

impl NodesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        self.nodes.push(node.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.nodes.extend(nodes.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    #[must_use]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self
    }

    fn extract_and_validate_nodes(self) -> Result<Vec<String>> {
        let node_strs: Vec<String> = self.nodes.into_iter().map(|s| s.to_string()).collect();
        let node_refs: Vec<&str> = node_strs.iter().map(|s| s.as_str()).collect();
        super::validate_nodes(&node_refs)?;
        Ok(node_strs)
    }

    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        let node_strs = self.extract_and_validate_nodes()?;
        Ok(Self { nodes: node_strs })
    }

    pub fn build(self) -> Result<Nodes> {
        let node_strs = self.extract_and_validate_nodes()?;
        Ok(Nodes { nodes: node_strs })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{Error, error::lang};

    #[test]
    #[ignore]
    fn test_nodes_builder_duplicate() {
        let result = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains(lang::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    #[ignore]
    fn test_nodes_builder_too_many() {
        let mut builder = NodesBuilder::new();
        for i in 0..257 {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => {
                assert!(msg.contains(lang::NODES_TOO_MANY));
            }
            _ => panic!("Expected Nodes error"),
        }
    }
}
