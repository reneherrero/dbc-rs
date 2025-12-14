use crate::compat::{String, Vec};
use crate::error::lang;
use crate::{BU_, Error, MAX_NAME_SIZE, MAX_NODES, Parser, Result};

/// Represents a collection of node (ECU) names from a DBC file.
///
/// The `BU_` statement in a DBC file lists all nodes (ECUs) on the CAN bus.
/// This struct stores the node names as borrowed references.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.0"
///
/// BU_: ECM TCM BCM
///
/// BO_ 256 Engine : 8 ECM
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
/// "#)?;
///
/// // Access nodes
/// assert_eq!(dbc.nodes().len(), 3);
/// assert!(dbc.nodes().contains("ECM"));
/// assert!(dbc.nodes().contains("TCM"));
///
/// // Iterate over nodes
/// for node in dbc.nodes().iter() {
///     println!("Node: {}", node);
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Empty Nodes
///
/// A DBC file may have an empty node list (`BU_:` with no nodes):
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.0"
///
/// BU_:
///
/// BO_ 256 Engine : 8 ECM
/// "#)?;
///
/// assert!(dbc.nodes().is_empty());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # DBC Format
///
/// In DBC files, nodes are specified on the `BU_` line:
/// - Format: `BU_: Node1 Node2 Node3 ...`
/// - Node names are space-separated
/// - Maximum of 256 nodes (DoS protection)
/// - Duplicate node names are not allowed (case-sensitive)
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Nodes {
    nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }>,
}

impl Nodes {
    // Shared validation function
    pub(crate) fn validate(nodes: &[impl AsRef<str>]) -> Result<()> {
        // Check for too many nodes (DoS protection)
        if let Some(err) = crate::check_max_limit(
            nodes.len(),
            MAX_NODES,
            Error::Validation(lang::NODES_TOO_MANY),
        ) {
            return Err(err);
        }

        // Check for duplicate node names (case-sensitive)
        for (i, node1) in nodes.iter().enumerate() {
            for node2 in nodes.iter().skip(i + 1) {
                if node1.as_ref() == node2.as_ref() {
                    return Err(Error::Validation(lang::NODES_DUPLICATE_NAME));
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    pub(crate) fn new(nodes: &[impl AsRef<str>]) -> Self {
        // Validation should have been done prior (by builder)
        let nodes_vec: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }> =
            nodes.iter().take(MAX_NODES).map(|n| n.as_ref().to_string().into()).collect();
        Self { nodes: nodes_vec }
    }

    #[must_use = "parse result should be checked"]
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Nodes parsing must always start with "BU_" keyword
        parser
            .expect(BU_.as_bytes())
            .map_err(|_| Error::Expected("Expected BU_ keyword"))?;

        // Expect ":" after "BU_" (no whitespace between BU_ and :)
        parser.expect_with_msg(b":", "Expected colon after BU_")?;

        // Skip optional whitespace after ":"
        parser.skip_newlines_and_spaces();

        // Parse node names into Vec
        let mut node_names: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }> = Vec::new();

        loop {
            // Skip whitespace before each node name
            parser.skip_whitespace_optional();

            // Try to parse an identifier (node name)
            // parse_identifier() will fail if we're at EOF
            match parser.parse_identifier() {
                Ok(node) => {
                    if let Some(err) = crate::check_max_limit(
                        node_names.len(),
                        MAX_NODES - 1,
                        Error::Nodes(lang::NODES_TOO_MANY),
                    ) {
                        return Err(err);
                    }
                    let node_str = crate::validate_name(node)?;
                    node_names.push(node_str).map_err(|_| Error::Nodes(lang::NODES_TOO_MANY))?;
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if node_names.is_empty() {
            return Ok(Nodes { nodes: Vec::new() });
        }

        // Validate before construction
        Self::validate(node_names.as_slice()).map_err(|e| {
            crate::error::map_val_error(e, Error::Nodes, || Error::Nodes(lang::NODES_ERROR_PREFIX))
        })?;
        // Construct directly (validation already done)
        Ok(Self { nodes: node_names })
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn at(&self, index: usize) -> Option<&str> {
        self.nodes.get(index).map(|s| s.as_str())
    }

    /// Converts the nodes to their DBC file representation.
    ///
    /// Returns a string in the format: `BU_: Node1 Node2 Node3 ...`
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
    /// let dbc_string = dbc.nodes().to_dbc_string();
    /// assert_eq!(dbc_string, "BU_: ECM TCM BCM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Empty Nodes
    ///
    /// Empty node lists are represented as `BU_:`:
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_:
    /// "#)?;
    ///
    /// assert_eq!(dbc.nodes().to_dbc_string(), "BU_:");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Feature Requirements
    ///
    /// This method requires the `std` feature to be enabled.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> std::string::String {
        let mut result = format!("{}:", BU_);
        let nodes_str = format!("{}", self);
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Nodes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.nodes.is_empty() {
            return Ok(());
        }
        for (i, node) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{Error, Parser, error::lang};

    #[test]
    fn test_nodes_from_valid_line() {
        let line = b"BU_: ECM TCM BCM ABS";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("ECM"));
        assert_eq!(iter.next(), Some("TCM"));
        assert_eq!(iter.next(), Some("BCM"));
        assert_eq!(iter.next(), Some("ABS"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = b"BU_: ONLYONE";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("ONLYONE"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = b"BU_:   Node1   Node2   ";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("Node1"));
        assert_eq!(iter.next(), Some("Node2"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = b"BU_:";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        assert!(nodes.is_empty());
    }

    // Note: Builder tests have been moved to nodes_builder.rs
    // This module only tests Nodes parsing and direct API usage

    #[test]
    fn test_nodes_parse_duplicate() {
        let line = b"BU_: ECM TCM ECM";
        let mut parser = Parser::new(line).unwrap();
        let result = Nodes::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg == lang::NODES_DUPLICATE_NAME),
            _ => panic!("Expected Error::Nodes"),
        }
    }
}
