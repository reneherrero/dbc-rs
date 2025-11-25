use crate::{Error, error::messages};
use alloc::{boxed::Box, string::String, vec::Vec};

/// Represents the list of nodes (ECUs) defined in a DBC file.
///
/// Nodes represent the electronic control units (ECUs) that participate
/// in the CAN bus communication. Each message must have a sender that
/// is present in the nodes list.
///
/// # Examples
///
/// ```rust
/// use dbc_rs::Nodes;
///
/// let nodes = Nodes::builder()
///     .add_node("ECM")
///     .add_node("TCM")
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct Nodes {
    nodes: Vec<Box<str>>,
}

impl Nodes {
    /// Create a new builder for constructing a `Nodes`
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Nodes;
    ///
    /// let nodes = Nodes::builder()
    ///     .add_node("ECM")
    ///     .add_node("TCM")
    ///     .add_node("BCM")
    ///     .build()
    ///     .unwrap();
    /// assert!(nodes.contains("ECM"));
    /// assert!(nodes.contains("TCM"));
    /// ```
    pub fn builder() -> NodesBuilder {
        NodesBuilder::new()
    }

    /// This is an internal constructor. For public API usage, use [`Nodes::builder()`] instead.
    #[allow(dead_code)] // Used in tests
    pub(crate) fn new<I, S>(nodes: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let nodes: Vec<Box<str>> = nodes.into_iter().map(|s| s.as_ref().into()).collect();
        Self::validate(&nodes)?;
        Ok(Self { nodes })
    }

    /// Validate node names according to DBC format specifications
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Duplicate node names are found (case-sensitive)
    fn validate(nodes: &[Box<str>]) -> Result<(), Error> {
        // Check for duplicate node names (case-sensitive)
        for (i, node1) in nodes.iter().enumerate() {
            for node2 in nodes.iter().skip(i + 1) {
                if node1.as_ref() == node2.as_ref() {
                    return Err(Error::Nodes(messages::duplicate_node_name(node1.as_ref())));
                }
            }
        }
        Ok(())
    }

    pub(super) fn parse(nodes: &str) -> Result<Self, Error> {
        let nodes: Vec<Box<str>> = nodes[4..].split_whitespace().map(|s| s.into()).collect();
        Self::validate(&nodes)?;
        Ok(Self { nodes })
    }

    /// Get a read-only slice of node names.
    ///
    /// Returns `None` if there are no nodes, otherwise returns `Some(&[Box<str>])`.
    #[inline]
    pub fn nodes(&self) -> Option<&[Box<str>]> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes.as_ref())
        }
    }

    /// Check if a node name exists in the list
    #[inline]
    pub fn contains(&self, node: &str) -> bool {
        self.nodes.iter().any(|n| n.as_ref() == node)
    }

    /// Format nodes as a space-separated string for saving
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        if self.nodes.is_empty() {
            return String::new();
        }
        // Pre-allocate: estimate ~10 chars per node name + spaces
        let capacity = self.nodes.len() * 10;
        let mut result = String::with_capacity(capacity);
        for (i, node) in self.nodes.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(node.as_ref());
        }
        result
    }

    /// Check if the nodes list is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Format nodes in DBC file format (e.g., `BU_: ECM TCM`)
    ///
    /// Useful for debugging and visualization of the nodes in DBC format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Nodes;
    ///
    /// let nodes = Nodes::builder()
    ///     .add_node("ECM")
    ///     .add_node("TCM")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(nodes.to_dbc_string(), "BU_: ECM TCM");
    /// ```
    pub fn to_dbc_string(&self) -> String {
        let mut result = String::from("BU_:");
        let nodes_str = self.to_string();
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}

/// Builder for constructing a `Nodes` with a fluent API
///
/// This builder provides a more ergonomic way to construct `Nodes` instances.
///
/// # Examples
///
/// ```
/// use dbc_rs::Nodes;
///
/// // Add nodes one by one
/// let nodes = Nodes::builder()
///     .add_node("ECM")
///     .add_node("TCM")
///     .add_node("BCM")
///     .build();
///
/// // Add nodes from an iterator
/// let nodes = Nodes::builder()
///     .add_nodes(&["ECM", "TCM", "BCM"])
///     .build();
/// ```
#[derive(Debug)]
pub struct NodesBuilder {
    nodes: Vec<Box<str>>,
}

impl NodesBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a single node
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        self.nodes.push(node.as_ref().into());
        self
    }

    /// Add multiple nodes from an iterator
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.nodes.extend(nodes.into_iter().map(|s| s.as_ref().into()));
        self
    }

    /// Clear all nodes
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self
    }

    /// Validate the current builder state
    ///
    /// This method validates the builder's current state according to DBC format
    /// specifications. Currently checks for duplicate node names.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Duplicate node names are found (case-sensitive)
    #[must_use]
    pub fn validate(&self) -> Result<(), Error> {
        Nodes::validate(&self.nodes)
    }

    /// Build the `Nodes` from the builder
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Duplicate node names are found (case-sensitive)
    #[must_use]
    pub fn build(self) -> Result<Nodes, Error> {
        Nodes::validate(&self.nodes)?;
        Ok(Nodes { nodes: self.nodes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodes_from_valid_line() {
        let line = "BU_: ECM TCM BCM ABS";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(
            node_slice,
            &["ECM".into(), "TCM".into(), "BCM".into(), "ABS".into()]
        );
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = "BU_: ONLYONE";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["ONLYONE".into()]);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = "BU_:   Node1   Node2   ";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["Node1".into(), "Node2".into()]);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = "BU_:";
        let nodes = Nodes::parse(line).unwrap();
        assert!(nodes.nodes().is_none());
    }

    #[test]
    fn test_nodes_new() {
        let nodes = Nodes::new(&["ECM", "TCM", "BCM"]).unwrap();
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
        assert!(!nodes.contains("ABS"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_from_vec() {
        let node_vec = vec!["Node1", "Node2", "Node3"];
        let nodes = Nodes::new(node_vec).unwrap();
        assert!(nodes.contains("Node1"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_from_slice() {
        let node_slice = &["A", "B", "C"][..];
        let nodes = Nodes::new(node_slice).unwrap();
        assert!(nodes.contains("A"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_duplicate() {
        let result = Nodes::new(&["ECM", "TCM", "ECM"]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains("Duplicate node name")),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    fn test_nodes_to_string_single() {
        let nodes = Nodes::new(&["ECM"]).unwrap();
        assert_eq!(nodes.to_string(), "ECM");
    }

    #[test]
    fn test_nodes_to_string_multiple() {
        let nodes = Nodes::new(&["ECM", "TCM", "BCM"]).unwrap();
        assert_eq!(nodes.to_string(), "ECM TCM BCM");
    }

    #[test]
    fn test_nodes_to_dbc_string() {
        let nodes_single = Nodes::new(&["ECM"]).unwrap();
        assert_eq!(nodes_single.to_dbc_string(), "BU_: ECM");

        let nodes_multiple = Nodes::new(&["ECM", "TCM", "BCM"]).unwrap();
        assert_eq!(nodes_multiple.to_dbc_string(), "BU_: ECM TCM BCM");
    }

    #[test]
    fn test_nodes_builder_duplicate() {
        let result = Nodes::builder().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains("Duplicate node name")),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    fn test_nodes_parse_duplicate() {
        let line = "BU_: ECM TCM ECM";
        let result = Nodes::parse(line);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains("Duplicate node name")),
            _ => panic!("Expected Nodes error"),
        }
    }
}
