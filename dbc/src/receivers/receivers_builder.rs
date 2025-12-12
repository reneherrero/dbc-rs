use std::{collections::HashSet, string::String};

use crate::error::lang;
use crate::{Error, MAX_NAME_SIZE, MAX_RECEIVER_NODES, Result};

use super::Receivers;

/// Builder for creating `Receivers` programmatically.
///
/// This builder allows you to construct receiver configurations for signals
/// when building DBC files programmatically.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{ReceiversBuilder, SignalBuilder, ByteOrder};
///
/// // Broadcast receiver
/// let broadcast = ReceiversBuilder::new().broadcast().build()?;
///
/// // Specific nodes
/// let specific = ReceiversBuilder::new()
///     .add_node("TCM")
///     .add_node("BCM")
///     .build()?;
///
/// // No receivers
/// let none = ReceiversBuilder::new().none().build()?;
///
/// // Use with signal builder
/// let signal = SignalBuilder::new()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .byte_order(ByteOrder::BigEndian)
///     .unsigned(true)
///     .factor(0.25)
///     .offset(0.0)
///     .min(0.0)
///     .max(8000.0)
///     .receivers(ReceiversBuilder::new().add_node("TCM").add_node("BCM"))
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct ReceiversBuilder {
    is_broadcast: bool,
    nodes: Vec<String>,
}

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
        Self {
            is_broadcast: false,
            nodes: Vec::new(),
        }
    }

    /// Sets the receiver to broadcast (`*` in DBC format).
    ///
    /// This clears any previously set nodes and sets the receiver to broadcast mode.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// let receivers = ReceiversBuilder::new()
    ///     .add_node("TCM")  // This will be cleared
    ///     .broadcast()
    ///     .build()?;
    /// assert_eq!(receivers, dbc_rs::Receivers::Broadcast);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn broadcast(mut self) -> Self {
        self.is_broadcast = true;
        self.nodes.clear();
        self
    }

    /// Sets the receiver to none (no explicit receivers).
    ///
    /// This clears any previously set nodes and sets the receiver to none mode.
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
    #[must_use]
    pub fn none(mut self) -> Self {
        self.is_broadcast = false;
        self.nodes.clear();
        self
    }

    /// Adds a single receiver node.
    ///
    /// This automatically clears broadcast and none modes, switching to specific nodes mode.
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
    pub fn add_node(mut self, node: impl AsRef<str>) -> Self {
        let node = node.as_ref().to_string();
        self.is_broadcast = false;
        self.nodes.push(node);

        self
    }

    /// Adds multiple receiver nodes from an iterator.
    ///
    /// This automatically clears broadcast and none modes, switching to specific nodes mode.
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
    #[must_use]
    pub fn clear(mut self) -> Self {
        self.nodes.clear();
        self.is_broadcast = false;
        self
    }
}

impl ReceiversBuilder {
    /// Builds the `Receivers` from the builder configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Receivers)` if successful, or `Err(Error::Signal)` if:
    /// - More than 64 receiver nodes are specified (exceeds maximum limit)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// // Broadcast
    /// let broadcast = ReceiversBuilder::new().broadcast().build()?;
    ///
    /// // Specific nodes
    /// let nodes = ReceiversBuilder::new()
    ///     .add_node("TCM")
    ///     .add_node("BCM")
    ///     .build()?;
    ///
    /// // None (default)
    /// let none = ReceiversBuilder::new().build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// ```rust,no_run
    /// use dbc_rs::ReceiversBuilder;
    ///
    /// // Too many nodes (limit is 64)
    /// let mut builder = ReceiversBuilder::new();
    /// for i in 0..65 {
    ///     builder = builder.add_node(format!("Node{i}"));
    /// }
    /// assert!(builder.build().is_err());
    /// ```
    pub fn build(&self) -> Result<Receivers> {
        if self.is_broadcast {
            Ok(Receivers::new_broadcast())
        } else if self.nodes.is_empty() {
            Ok(Receivers::new_none())
        } else {
            // Make sure the number of nodes is not greater than the maximum allowed
            if let Some(err) = crate::check_max_limit(
                self.nodes.len(),
                MAX_RECEIVER_NODES,
                Error::Signal(lang::SIGNAL_RECEIVERS_TOO_MANY),
            ) {
                return Err(err);
            }

            // Make sure the nodes are not duplicated
            let mut seen = HashSet::new();
            if !self.nodes.iter().all(|item| seen.insert(item)) {
                return Err(Error::Signal(lang::RECEIVERS_DUPLICATE_NAME));
            }

            // Make sure the node names are not too long and convert to compat::String
            use crate::compat::{String, Vec};
            let mut compat_nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_RECEIVER_NODES }> =
                Vec::new();
            for node in &self.nodes {
                if let Some(err) = crate::check_max_limit(
                    node.len(),
                    MAX_NAME_SIZE,
                    Error::Signal(lang::MAX_NAME_SIZE_EXCEEDED),
                ) {
                    return Err(err);
                }
                let compat_str = String::try_from(node.as_str())
                    .map_err(|_| Error::Signal(lang::MAX_NAME_SIZE_EXCEEDED))?;
                compat_nodes
                    .push(compat_str)
                    .map_err(|_| Error::Signal(lang::SIGNAL_RECEIVERS_TOO_MANY))?;
            }

            Ok(Receivers::new_nodes(compat_nodes.as_slice()))
        }
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
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 1),
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
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 3),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_too_many() {
        let mut builder = ReceiversBuilder::new();
        for i in 0..MAX_RECEIVER_NODES {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.add_node(format!("Node{}", MAX_RECEIVER_NODES)).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal(msg) => {
                assert_eq!(msg, lang::SIGNAL_RECEIVERS_TOO_MANY);
            }
            _ => panic!("Expected Signal error"),
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

    #[test]
    fn test_receivers_builder_broadcast_clears_nodes() {
        let receivers = ReceiversBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .broadcast()
            .build()
            .unwrap();
        assert_eq!(receivers, Receivers::Broadcast);
        assert_eq!(receivers.len(), 0);
    }

    #[test]
    fn test_receivers_builder_none_clears_nodes() {
        let receivers =
            ReceiversBuilder::new().add_node("ECM").add_node("TCM").none().build().unwrap();
        assert_eq!(receivers, Receivers::None);
        assert_eq!(receivers.len(), 0);
    }

    #[test]
    fn test_receivers_builder_add_node_clears_broadcast() {
        let receivers = ReceiversBuilder::new().broadcast().add_node("ECM").build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 1),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_add_node_clears_none() {
        let receivers = ReceiversBuilder::new().none().add_node("ECM").build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), 1),
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_receivers_builder_at_limit() {
        let mut builder = ReceiversBuilder::new();
        for i in 0..MAX_RECEIVER_NODES {
            let node_str = format!("Node{i}");
            builder = builder.add_node(node_str);
        }
        let receivers = builder.build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), MAX_RECEIVER_NODES),
            _ => panic!("Expected Nodes variant"),
        }
    }
}
