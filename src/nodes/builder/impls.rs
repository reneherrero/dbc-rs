use super::NodesBuilder;
use std::vec::Vec;

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

    /// Adds a single node to the list without a comment.
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
        self.nodes.push((node.as_ref().to_string(), None));
        self
    }

    /// Adds a single node to the list with an optional comment.
    ///
    /// # Arguments
    ///
    /// * `node` - The node name (anything that implements `AsRef<str>`)
    /// * `comment` - The comment for this node
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::NodesBuilder;
    ///
    /// let nodes = NodesBuilder::new()
    ///     .add_node_with_comment("ECM", "Engine Control Module")
    ///     .add_node("TCM")
    ///     .build()?;
    /// assert_eq!(nodes.len(), 2);
    /// assert_eq!(nodes.node_comment("ECM"), Some("Engine Control Module"));
    /// assert_eq!(nodes.node_comment("TCM"), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_node_with_comment(
        mut self,
        node: impl AsRef<str>,
        comment: impl AsRef<str>,
    ) -> Self {
        self.nodes.push((
            node.as_ref().to_string(),
            Some(comment.as_ref().to_string()),
        ));
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
}
