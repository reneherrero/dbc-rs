use crate::{Error, MAX_NAME_SIZE, MAX_NODES, Nodes, Result};
use std::{string::String, vec::Vec};

/// Builder for creating `Nodes` programmatically.
///
/// This builder allows you to construct node lists when building DBC files
/// programmatically. It validates that node names are unique and within limits.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::NodesBuilder;
///
/// let nodes = NodesBuilder::new()
///     .add_node("ECM")
///     .add_node("TCM")
///     .add_node("BCM")
///     .build()?;
///
/// assert_eq!(nodes.len(), 3);
/// assert!(nodes.contains("ECM"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Validation
///
/// The builder validates:
/// - Maximum of 256 nodes (DoS protection)
/// - All node names must be unique (case-sensitive)
/// - Maximum 32 characters per node name
/// - Maximum number of nodes (implementation limit for DoS protection)
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug)]
pub struct NodesBuilder {
    nodes: Vec<String>,
}

impl NodesBuilder {
    /// Creates a new `NodesBuilder` with an empty node list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// let builder = NodesBuilder::new();
    /// let nodes = builder.build()?;
    /// assert!(nodes.is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a single node to the list.
    ///
    /// # Arguments
    ///
    /// * `node` - The node name (anything that implements `AsRef<str>`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// let nodes = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("TCM")
    ///     .build()?;
    /// assert_eq!(nodes.len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        self.nodes.push(node.as_ref().to_string());
        self
    }

    /// Adds multiple nodes from an iterator.
    ///
    /// # Arguments
    ///
    /// * `nodes` - An iterator of node names (each item must implement `AsRef<str>`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// // From a slice
    /// let nodes = NodesBuilder::new()
    ///     .add_nodes(&["ECM", "TCM", "BCM"])
    ///     .build()?;
    /// assert_eq!(nodes.len(), 3);
    ///
    /// // From a vector
    /// let node_vec = vec!["Node1", "Node2"];
    /// let nodes2 = NodesBuilder::new()
    ///     .add_nodes(node_vec.iter())
    ///     .build()?;
    /// assert_eq!(nodes2.len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for node in nodes {
            self = self.add_node(node.as_ref());
        }

        self
    }

    /// Clears all nodes from the builder.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// let nodes = NodesBuilder::new()
    ///     .add_node("ECM")
    ///     .add_node("TCM")
    ///     .clear()
    ///     .add_node("BCM")
    ///     .build()?;
    /// assert_eq!(nodes.len(), 1);
    /// assert!(nodes.contains("BCM"));
    /// assert!(!nodes.contains("ECM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self
    }

    fn extract_and_validate_nodes(self) -> Result<Vec<String>> {
        Nodes::validate(&self.nodes)?;
        Ok(self.nodes)
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
        let nodes = self.extract_and_validate_nodes()?;
        Ok(Self { nodes })
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
        let nodes = self.extract_and_validate_nodes()?;
        // Convert std::vec::Vec<String> to compat::Vec<String<MAX_NAME_SIZE>, MAX_NODES>
        use crate::compat::{String, Vec};
        let mut result: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }> = Vec::new();
        for node_str in nodes {
            let compat_str = String::try_from(node_str.as_str())
                .map_err(|_| Error::Validation(Error::MAX_NAME_SIZE_EXCEEDED))?;
            result.push(compat_str).map_err(|_| Error::Validation(Error::NODES_TOO_MANY))?;
        }
        Ok(Nodes::new(result))
    }
}

impl Default for NodesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::Error;

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
    fn test_nodes_builder_add_nodes() {
        let nodes = NodesBuilder::new().add_nodes(["ECM", "TCM", "BCM"]).build().unwrap();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
    }

    #[test]
    fn test_nodes_builder_add_nodes_iterator() {
        let node_vec = ["Node1", "Node2", "Node3"];
        let nodes = NodesBuilder::new().add_nodes(node_vec.iter()).build().unwrap();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains("Node1"));
    }

    #[test]
    fn test_nodes_builder_clear() {
        let nodes = NodesBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .clear()
            .add_node("BCM")
            .build()
            .unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains("BCM"));
        assert!(!nodes.contains("ECM"));
        assert!(!nodes.contains("TCM"));
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

    #[test]
    fn test_nodes_builder_empty() {
        let nodes = NodesBuilder::new().build().unwrap();
        assert!(nodes.is_empty());
        assert_eq!(nodes.len(), 0);
    }
}
