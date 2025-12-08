use crate::{
    Parser,
    error::{ParseError, ParseResult, messages},
};

/// Iterator over nodes in a Nodes collection
struct NodesIter<'a, 'b> {
    nodes: &'b [Option<&'a str>],
    count: usize,
    pos: usize,
}

impl<'a, 'b> Iterator for NodesIter<'a, 'b> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.count {
            let result = self.nodes[self.pos];
            self.pos += 1;
            if let Some(node) = result {
                return Some(node);
            }
        }
        None
    }
}

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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Nodes<'a> {
    nodes: [Option<&'a str>; crate::MAX_NODES],
    count: usize,
}

impl<'a> Default for Nodes<'a> {
    fn default() -> Self {
        Self {
            nodes: [const { None }; crate::MAX_NODES],
            count: 0,
        }
    }
}

impl<'a> Nodes<'a> {
    // Shared validation function
    pub(crate) fn validate_nodes(nodes: &[&str]) -> ParseResult<()> {
        use crate::error::lang;
        // Check for too many nodes (DoS protection)
        if nodes.len() > crate::MAX_NODES {
            return Err(ParseError::Nodes(messages::NODES_TOO_MANY));
        }

        // Check for duplicate node names (case-sensitive)
        for (i, node1) in nodes.iter().enumerate() {
            for node2 in nodes.iter().skip(i + 1) {
                if *node1 == *node2 {
                    return Err(ParseError::Nodes(lang::NODES_DUPLICATE_NAME));
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)] // Only used by builders (std-only)
    pub(crate) fn new(nodes: &[&'a str]) -> Self {
        // Validation should have been done prior (by builder)
        let mut node_array: [Option<&'a str>; crate::MAX_NODES] =
            [const { None }; crate::MAX_NODES];
        let count = nodes.len().min(crate::MAX_NODES);
        for (i, node) in nodes.iter().take(crate::MAX_NODES).enumerate() {
            node_array[i] = Some(*node);
        }
        Self {
            nodes: node_array,
            count,
        }
    }

    #[must_use = "parse result should be checked"]
    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        // Expect "BU_:" keyword
        // Note: When called from Dbc::parse, find_next_keyword already advanced past "BU_",
        // so we try to expect "BU_" first, and if that fails, we're already past it and just expect ":"
        if parser.expect(crate::BU_.as_bytes()).is_ok() {
            // Successfully consumed "BU_", now expect ":"
            parser
                .expect(b":")
                .map_err(|_| ParseError::Expected("Expected colon after BU_"))?;
        } else {
            // Already past "BU_" from find_next_keyword
            // find_next_keyword advances to right after "BU_", which should be at ":" or whitespace
            // Try to expect ":" - if it fails, skip whitespace and try again
            if parser.expect(b":").is_err() {
                // Not at ":", skip whitespace and try again
                parser.skip_newlines_and_spaces();
                parser
                    .expect(b":")
                    .map_err(|_| ParseError::Expected("Expected colon after BU_"))?;
            }
        }

        // Skip optional whitespace after ":"
        parser.skip_newlines_and_spaces();

        // Parse node names into fixed-size array
        let mut node_names: [Option<&'b str>; crate::MAX_NODES] =
            [const { None }; crate::MAX_NODES];
        let mut count = 0;

        loop {
            // Skip whitespace before each node name
            let _ = parser.skip_whitespace();

            // Try to parse an identifier (node name)
            // parse_identifier() will fail if we're at EOF
            match parser.parse_identifier() {
                Ok(node) => {
                    if count >= crate::MAX_NODES {
                        return Err(ParseError::Nodes(messages::NODES_TOO_MANY));
                    }
                    node_names[count] = Some(node);
                    count += 1;
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if count == 0 {
            return Ok(Nodes {
                nodes: [const { None }; crate::MAX_NODES],
                count: 0,
            });
        }

        // Collect valid node names into a slice for validation and construction
        // We use a stack-allocated array to avoid allocation
        let mut node_refs: [&'b str; crate::MAX_NODES] = [""; crate::MAX_NODES];
        for i in 0..count {
            if let Some(node) = node_names[i] {
                node_refs[i] = node;
            }
        }

        // Validate before construction
        Self::validate_nodes(&node_refs[..count])?;
        // Construct directly (validation already done)
        let mut node_array: [Option<&'a str>; crate::MAX_NODES] =
            [const { None }; crate::MAX_NODES];
        for (i, node) in node_refs.iter().take(count).enumerate() {
            node_array[i] = Some(*node);
        }
        Ok(Self {
            nodes: node_array,
            count,
        })
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
    pub fn iter(&self) -> impl Iterator<Item = &'a str> + '_ {
        NodesIter {
            nodes: &self.nodes,
            count: self.count,
            pos: 0,
        }
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
        self.count
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
        self.count == 0
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
    pub fn at(&self, index: usize) -> Option<&'a str> {
        if index >= self.count {
            return None;
        }
        self.nodes[index]
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
    /// This method requires the `alloc` feature to be enabled.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> alloc::string::String {
        use alloc::string::String;
        let mut result = String::from(crate::BU_);
        result.push(':');
        let nodes_str = alloc::format!("{}", self);
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<'a> core::fmt::Display for Nodes<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.count == 0 {
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
    use crate::{
        Parser,
        error::{ParseError, lang},
    };

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
            ParseError::Nodes(msg) => assert!(msg == lang::NODES_DUPLICATE_NAME),
            _ => panic!("Expected ParseError::Nodes"),
        }
    }

    // Note: Builder limit tests have been moved to nodes_builder.rs
}
