use super::{Receivers, ReceiversBuilder};
use crate::{Error, MAX_NAME_SIZE, MAX_NODES, Result};

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
    /// // Specific nodes
    /// let nodes = ReceiversBuilder::new()
    ///     .add_node("TCM")
    ///     .add_node("BCM")
    ///     .build()?;
    ///
    /// // None (default - serializes as Vector__XXX)
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
    pub fn build(self) -> Result<Receivers> {
        if self.nodes.is_empty() {
            Ok(Receivers::new_none())
        } else {
            // Make sure the number of nodes is not greater than the maximum allowed
            // Receivers can have at most MAX_NODES - 1 nodes
            if let Some(err) = crate::error::check_max_limit(
                self.nodes.len(),
                MAX_NODES - 1,
                Error::signal(Error::SIGNAL_RECEIVERS_TOO_MANY),
            ) {
                return Err(err);
            }

            // Make sure the nodes are not duplicated
            // Using O(n²) loop instead of HashSet to avoid allocation overhead
            // (receiver lists are typically small, making this faster in practice)
            for (i, node1) in self.nodes.iter().enumerate() {
                for node2 in self.nodes.iter().skip(i + 1) {
                    if node1 == node2 {
                        return Err(Error::signal(Error::RECEIVERS_DUPLICATE_NAME));
                    }
                }
            }

            // Make sure the node names are not too long and convert to compat::String
            use crate::compat::{String, Vec};
            let mut compat_nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES - 1 }> = Vec::new();
            for node in &self.nodes {
                if let Some(err) = crate::error::check_max_limit(
                    node.len(),
                    MAX_NAME_SIZE,
                    Error::signal(Error::MAX_NAME_SIZE_EXCEEDED),
                ) {
                    return Err(err);
                }
                let compat_str = String::try_from(node.as_str())
                    .map_err(|_| Error::signal(Error::MAX_NAME_SIZE_EXCEEDED))?;
                compat_nodes
                    .push(compat_str)
                    .map_err(|_| Error::signal(Error::SIGNAL_RECEIVERS_TOO_MANY))?;
            }

            Ok(Receivers::new_nodes(compat_nodes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

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
        for i in 0..MAX_NODES {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.add_node(format!("Node{}", MAX_NODES)).build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Signal { msg, .. } => {
                assert_eq!(msg, Error::SIGNAL_RECEIVERS_TOO_MANY);
            }
            _ => panic!("Expected Signal error"),
        }
    }

    #[test]
    fn test_receivers_builder_at_limit() {
        let mut builder = ReceiversBuilder::new();
        // Fill up to MAX_NODES - 1 (the limit)
        for i in 0..(MAX_NODES - 1) {
            let node_str = format!("Node{i}");
            builder = builder.add_node(node_str);
        }
        let receivers = builder.build().unwrap();
        match &receivers {
            Receivers::Nodes(nodes) => assert_eq!(nodes.len(), MAX_NODES - 1),
            _ => panic!("Expected Nodes variant"),
        }
    }
}
