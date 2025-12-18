use super::ReceiversBuilder;
use std::vec::Vec;

impl ReceiversBuilder {
    /// Creates a new `ReceiversBuilder` with default settings (no receivers).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// let builder = ReceiversBuilder::new();
    /// let receivers = builder.build()?;
    /// assert_eq!(receivers.len(), 0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Sets the receiver to none (no explicit receivers).
    ///
    /// This clears any previously set nodes. Per DBC spec, this will
    /// serialize as `Vector__XXX` (no specific receiver).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// let receivers = ReceiversBuilder::new()
    ///     .add_node("TCM")  // This will be cleared
    ///     .none()
    ///     .build()?;
    /// assert_eq!(receivers, dbc_rs::Receivers::None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn none(mut self) -> Self {
        self.nodes.clear();
        self
    }

    /// Adds a single receiver node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node name (anything that implements `AsRef<str>`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// let receivers = ReceiversBuilder::new()
    ///     .add_node("TCM")
    ///     .add_node("BCM")
    ///     .build()?;
    /// assert_eq!(receivers.len(), 2);
    /// assert!(receivers.contains("TCM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        let node = node.as_ref().to_string();
        self.nodes.push(node);
        self
    }

    /// Adds multiple receiver nodes from an iterator.
    ///
    /// # Arguments
    ///
    /// * `nodes` - An iterator of node names (each item must implement `AsRef<str>`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// // From a slice
    /// let receivers = ReceiversBuilder::new()
    ///     .add_nodes(&["TCM", "BCM", "ECM"])
    ///     .build()?;
    /// assert_eq!(receivers.len(), 3);
    ///
    /// // From a vector
    /// let node_vec = vec!["Node1", "Node2"];
    /// let receivers2 = ReceiversBuilder::new()
    ///     .add_nodes(node_vec.iter())
    ///     .build()?;
    /// assert_eq!(receivers2.len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_nodes<I, S>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for node in nodes {
            self = self.add_node(node);
        }

        self
    }

    /// Clears all receiver nodes and resets to default state (none).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// let receivers = ReceiversBuilder::new()
    ///     .add_node("TCM")
    ///     .add_node("BCM")
    ///     .clear()
    ///     .add_node("ECM")
    ///     .build()?;
    /// assert_eq!(receivers.len(), 1);
    /// assert!(receivers.contains("ECM"));
    /// assert!(!receivers.contains("TCM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self
    }
}

impl Default for ReceiversBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Receivers;

    #[test]
    fn test_receivers_builder_none_clears_nodes() {
        let receivers =
            ReceiversBuilder::new().add_node("ECM").add_node("TCM").none().build().unwrap();
        assert_eq!(receivers, Receivers::None);
        assert_eq!(receivers.len(), 0);
    }

    #[test]
    fn test_receivers_builder_add_node_after_none() {
        let receivers = ReceiversBuilder::new().none().add_node("ECM").build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 1),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_add_nodes() {
        let receivers = ReceiversBuilder::new().add_nodes(["ECM", "TCM", "BCM"]).build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 3),
            _ => panic!("Expected Nodes variant"),
        }
        assert!(receivers.contains("ECM"));
        assert!(receivers.contains("TCM"));
        assert!(receivers.contains("BCM"));
    }

    #[test]
    fn test_receivers_builder_add_nodes_iterator() {
        let node_vec = ["Node1", "Node2", "Node3"];
        let receivers = ReceiversBuilder::new().add_nodes(node_vec.iter()).build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 3),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_clear() {
        let receivers = ReceiversBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .clear()
            .add_node("BCM")
            .build()
            .unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert!(receivers.contains("BCM"));
                assert!(!receivers.contains("ECM"));
            }
            _ => panic!("Expected Nodes variant"),
        }
    }
}
