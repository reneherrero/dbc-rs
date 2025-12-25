use super::{InnerNodes, Node, Nodes};

impl Nodes {
    pub(crate) fn new(nodes: InnerNodes) -> Self {
        // Validation should have been done prior (by builder)
        Self { nodes }
    }

    /// Returns an iterator over the node names.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    /// "#)?;
    ///
    /// // Iterate over nodes
    /// let mut iter = dbc.nodes().iter();
    /// assert_eq!(iter.next(), Some("ECM"));
    /// assert_eq!(iter.next(), Some("TCM"));
    /// assert_eq!(iter.next(), Some("BCM"));
    /// assert_eq!(iter.next(), None);
    ///
    /// // Or use in a loop
    /// for node in dbc.nodes().iter() {
    ///     println!("Node: {}", node);
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.nodes.iter().map(|node| node.name())
    }

    /// Returns an iterator over the node structs.
    ///
    /// This provides access to both the node name and its comment.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///
    /// CM_ BU_ ECM "Engine Control Module";
    /// "#)?;
    ///
    /// for node in dbc.nodes().iter_nodes() {
    ///     println!("Node: {}", node.name());
    ///     if let Some(comment) = node.comment() {
    ///         println!("  Comment: {}", comment);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node> + '_ {
        self.nodes.iter()
    }

    /// Checks if a node name is in the list.
    ///
    /// The check is case-sensitive.
    ///
    /// # Arguments
    ///
    /// * `node` - The node name to check
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM
    /// "#)?;
    ///
    /// assert!(dbc.nodes().contains("ECM"));
    /// assert!(dbc.nodes().contains("TCM"));
    /// assert!(!dbc.nodes().contains("BCM"));
    /// assert!(!dbc.nodes().contains("ecm")); // Case-sensitive
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn contains(&self, node: &str) -> bool {
        self.iter().any(|n| n == node)
    }

    /// Returns the number of nodes in the collection.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    /// "#)?;
    ///
    /// assert_eq!(dbc.nodes().len(), 3);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if there are no nodes in the collection.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// // Empty node list
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_:
    /// "#)?;
    /// assert!(dbc.nodes().is_empty());
    ///
    /// // With nodes
    /// let dbc2 = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    /// "#)?;
    /// assert!(!dbc2.nodes().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Gets a node name by index.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index of the node
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    /// "#)?;
    ///
    /// assert_eq!(dbc.nodes().at(0), Some("ECM"));
    /// assert_eq!(dbc.nodes().at(1), Some("TCM"));
    /// assert_eq!(dbc.nodes().at(2), Some("BCM"));
    /// assert_eq!(dbc.nodes().at(3), None); // Out of bounds
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn at(&self, index: usize) -> Option<&str> {
        self.nodes.get(index).map(|node| node.name())
    }

    /// Gets a node by index.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index of the node
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    /// "#)?;
    ///
    /// let node = dbc.nodes().get(0).unwrap();
    /// assert_eq!(node.name(), "ECM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    /// Returns the comment for a specific node, if present.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///
    /// CM_ BU_ ECM "Engine Control Module";
    /// "#)?;
    ///
    /// assert_eq!(dbc.nodes().node_comment("ECM"), Some("Engine Control Module"));
    /// assert_eq!(dbc.nodes().node_comment("TCM"), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn node_comment(&self, node_name: &str) -> Option<&str> {
        self.nodes
            .iter()
            .find(|node| node.name() == node_name)
            .and_then(|node| node.comment())
    }

    /// Sets the comment for a node by name.
    ///
    /// Returns `true` if the node was found and the comment was set,
    /// `false` if the node was not found.
    pub(crate) fn set_node_comment(
        &mut self,
        node_name: &str,
        comment: crate::compat::Comment,
    ) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.name() == node_name) {
            node.set_comment(comment);
            true
        } else {
            false
        }
    }
}
