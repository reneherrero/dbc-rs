use super::NodesBuilder;
use crate::nodes::{InnerNodes, Node};
use crate::{Error, MAX_NODES, Nodes, Result};

impl NodesBuilder {
    /// Convert std strings to compat Node structs and validate
    fn to_compat_nodes(&self) -> Result<InnerNodes> {
        use crate::compat::{Comment, Name, Vec};
        let mut result: InnerNodes = InnerNodes::new();
        let mut names: Vec<Name, { MAX_NODES }> = Vec::new();
        for (node_name, node_comment) in &self.nodes {
            let compat_name = Name::try_from(node_name.as_str())
                .map_err(|_| Error::Validation(Error::MAX_NAME_SIZE_EXCEEDED))?;
            let _ = names.push(compat_name.clone());

            let compat_comment = if let Some(comment) = node_comment {
                Some(
                    Comment::try_from(comment.as_str())
                        .map_err(|_| Error::Validation("Comment exceeds maximum length"))?,
                )
            } else {
                None
            };

            result
                .push(Node::with_comment(compat_name, compat_comment))
                .map_err(|_| Error::Validation(Error::NODES_TOO_MANY))?;
        }
        Nodes::validate(&names)?;
        Ok(result)
    }

    /// Validates the current builder state without building.
    ///
    /// This is useful for checking if the configuration is valid before building.
    /// Returns a new builder with validated nodes, or an error if validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if validation succeeds, or `Err(Error::Validation)` if:
    /// - Too many nodes are specified (exceeds 256 nodes  limit by default)
    /// - Duplicate node names are found (case-sensitive)
    /// - Node name exceeds maximum length (32 characters by default)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// // Valid configuration
    /// let builder = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("TCM");
    /// assert!(builder.validate().is_ok());
    ///
    /// // Invalid: duplicate nodes
    /// let builder2 = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("ECM"); // Duplicate
    /// assert!(builder2.validate().is_err());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        self.to_compat_nodes()?;
        Ok(self)
    }

    /// Builds the `Nodes` from the builder configuration.
    ///
    /// This validates the nodes and constructs a `Nodes` instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Nodes)` if successful, or `Err(Error::Validation)` if:
    /// - Too many nodes are specified (exceeds 256 nodes limit by default)
    /// - Duplicate node names are found (case-sensitive)
    /// - Node name exceeds maximum length (32 characters)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// // Build with nodes
    /// let nodes = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("TCM")
    ///     .build()?;
    /// assert_eq!(nodes.len(), 2);
    ///
    /// // Build empty
    /// let empty = NodesBuilder::new().build()?;
    /// assert!(empty.is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// // Duplicate nodes
    /// let result = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("ECM") // Duplicate
    ///     .build();
    /// assert!(result.is_err());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<Nodes> {
        let compat_nodes = self.to_compat_nodes()?;
        Ok(Nodes::new(compat_nodes))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::Error;

    #[test]
    fn test_nodes_builder_empty() {
        let nodes = NodesBuilder::new().build().unwrap();
        assert!(nodes.is_empty());
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_nodes_builder_duplicate() {
        let result = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Validation(msg) => assert!(msg.contains(Error::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_nodes_builder_too_many() {
        let mut builder = NodesBuilder::new();
        for i in 0..MAX_NODES {
            let node_str = format!("Node{i}");
            builder = builder.add_node(node_str);
        }
        let node = "NodeLast".to_string();
        let result = builder.add_node(node).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Validation(msg) => {
                assert!(msg.contains(Error::NODES_TOO_MANY));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_nodes_builder_validate() {
        let builder = NodesBuilder::new().add_node("TCM").add_node("BCM");
        let validated = builder.validate().unwrap();
        let nodes = validated.build().unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_nodes_builder_validate_duplicate() {
        let result = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Validation(msg) => assert!(msg.contains(Error::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_nodes_builder_validate_too_many() {
        let mut builder = NodesBuilder::new();
        for i in 0..MAX_NODES {
            let node_str = format!("Node{i}");
            builder = builder.add_node(node_str);
        }

        let result = builder.validate();
        assert!(result.is_ok());

        // Try to adding one past the limit
        builder = result.unwrap();
        let result = builder.add_node("NodeLast").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Validation(msg) => assert!(msg.contains(Error::NODES_TOO_MANY)),
            _ => panic!("Expected Validation error"),
        }
    }
}
