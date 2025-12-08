#[cfg(any(feature = "alloc", feature = "kernel"))]
use crate::compat::{Box, String, Vec, str_to_string};
use crate::{error::Error, error::Result, nodes::Nodes};

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
/// - No duplicate node names (case-sensitive)
///
/// # Feature Requirements
///
/// This builder requires the `alloc` feature to be enabled.
#[derive(Debug, Default)]
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
        Self::default()
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
    #[must_use]
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        self.nodes.push(str_to_string(node));
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
    #[must_use]
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.nodes.extend(nodes.into_iter().map(str_to_string));
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
    #[must_use]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self
    }

    fn extract_and_validate_nodes(self) -> Result<Vec<String>> {
        let node_strs: Vec<String> = crate::compat::strings_from_iter(self.nodes);
        let node_refs: Vec<&str> = node_strs.iter().map(|s| s.as_str()).collect();
        super::Nodes::validate_nodes(&node_refs).map_err(|e| match e {
            crate::error::ParseError::Nodes(msg) => Error::nodes(msg),
            _ => Error::from(e),
        })?;
        Ok(node_strs)
    }

    /// Validates the current builder state without building.
    ///
    /// This is useful for checking if the configuration is valid before building.
    /// Returns a new builder with validated nodes, or an error if validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if validation succeeds, or `Err(Error::Nodes)` if:
    /// - More than 256 nodes are specified (exceeds maximum limit)
    /// - Duplicate node names are found (case-sensitive)
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
        let node_strs = self.extract_and_validate_nodes()?;
        Ok(Self { nodes: node_strs })
    }

    /// Builds the `Nodes` from the builder configuration.
    ///
    /// This validates the nodes and constructs a `Nodes` instance with static lifetime.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Nodes)` if successful, or `Err(Error::Nodes)` if:
    /// - More than 256 nodes are specified (exceeds maximum limit)
    /// - Duplicate node names are found (case-sensitive)
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
    ///
    /// // Too many nodes (limit is 256)
    /// let mut builder = NodesBuilder::new();
    /// for i in 0..257 {
    ///     builder = builder.add_node(format!("Node{i}"));
    /// }
    /// assert!(builder.build().is_err());
    /// ```
    pub fn build(self) -> Result<Nodes<'static>> {
        let node_strs = self.extract_and_validate_nodes()?;
        // Convert owned Strings to static references by leaking Box<str>
        // This is acceptable for builder pattern where the caller owns the data
        let mut node_refs: Vec<&'static str> = Vec::new();
        for s in node_strs {
            let boxed: Box<str> = s.into_boxed_str();
            node_refs.push(Box::leak(boxed));
        }
        // Validate before construction
        super::Nodes::validate_nodes(&node_refs).map_err(|e| match e {
            crate::error::ParseError::Nodes(msg) => Error::nodes(msg),
            _ => Error::from(e),
        })?;
        Ok(Nodes::new(&node_refs))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{error::Error, error::lang};
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use alloc::format;

    #[test]
    fn test_nodes_builder_duplicate() {
        let result = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains(lang::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
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
        let builder = NodesBuilder::new().add_node("ECM").add_node("TCM");
        let validated = builder.validate().unwrap();
        let nodes = validated.build().unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_nodes_builder_validate_duplicate() {
        let builder = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM");
        let result = builder.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains(lang::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    fn test_nodes_builder_validate_too_many() {
        let mut builder = NodesBuilder::new();
        for i in 0..257 {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains(lang::NODES_TOO_MANY)),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    fn test_nodes_builder_empty() {
        let nodes = NodesBuilder::new().build().unwrap();
        assert!(nodes.is_empty());
        assert_eq!(nodes.len(), 0);
    }
}
