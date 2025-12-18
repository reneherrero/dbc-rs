use super::{NodeNames, Nodes};

impl Nodes {
    pub(crate) fn new(nodes: NodeNames) -> Self {
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
        self.nodes.iter().map(|s| s.as_str())
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
    /// assert_eq!(dbc.nodes().at(0), Some("ECM"));
    /// assert_eq!(dbc.nodes().at(1), Some("TCM"));
    /// assert_eq!(dbc.nodes().at(2), Some("BCM"));
    /// assert_eq!(dbc.nodes().at(3), None); // Out of bounds
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn at(&self, index: usize) -> Option<&str> {
        self.nodes.get(index).map(|s| s.as_str())
    }
}
