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
            return Err(ParseError::Version(messages::NODES_TOO_MANY));
        }

        // Check for duplicate node names (case-sensitive)
        for (i, node1) in nodes.iter().enumerate() {
            for node2 in nodes.iter().skip(i + 1) {
                if *node1 == *node2 {
                    return Err(ParseError::Version(lang::NODES_DUPLICATE_NAME));
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
                        return Err(ParseError::Version(messages::NODES_TOO_MANY));
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

    /// Get an iterator over the nodes
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM BCM")?;
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

    /// Check if a node is in the list
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM")?;
    /// assert!(dbc.nodes().contains("ECM"));
    /// assert!(!dbc.nodes().contains("BCM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(&self, node: &str) -> bool {
        self.iter().any(|n| n == node)
    }

    /// Get the number of nodes
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM BCM")?;
    /// assert_eq!(dbc.nodes().len(), 3);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if there are no nodes
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"")?;
    /// assert!(dbc.nodes().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get a node by index, or None if index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM")?;
    /// if let Some(node) = dbc.nodes().at(0) {
    ///     assert_eq!(node, "ECM");
    /// }
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

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
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
        let node_vec: alloc::vec::Vec<&str> = nodes.iter().collect();
        assert_eq!(node_vec, &["ECM", "TCM", "BCM", "ABS"]);
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = b"BU_: ONLYONE";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let node_vec: alloc::vec::Vec<&str> = nodes.iter().collect();
        assert_eq!(node_vec, &["ONLYONE"]);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = b"BU_:   Node1   Node2   ";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let node_vec: alloc::vec::Vec<&str> = nodes.iter().collect();
        assert_eq!(node_vec, &["Node1", "Node2"]);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = b"BU_:";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_new() {
        use crate::nodes::NodesBuilder;
        let nodes = NodesBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .add_node("BCM")
            .build()
            .unwrap();
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
        assert!(!nodes.contains("ABS"));
        assert_eq!(nodes.iter().count(), 3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_new_from_vec() {
        use crate::nodes::NodesBuilder;
        let nodes = NodesBuilder::new()
            .add_node("Node1")
            .add_node("Node2")
            .add_node("Node3")
            .build()
            .unwrap();
        assert!(nodes.contains("Node1"));
        assert_eq!(nodes.iter().count(), 3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_new_from_slice() {
        use crate::nodes::NodesBuilder;
        let nodes = NodesBuilder::new().add_node("A").add_node("B").add_node("C").build().unwrap();
        assert!(nodes.contains("A"));
        assert_eq!(nodes.iter().count(), 3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_new_duplicate() {
        use crate::nodes::NodesBuilder;
        let result = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("ECM").build();
        assert!(result.is_err());
        use crate::Error;
        match result.unwrap_err() {
            Error::Nodes(msg) => {
                assert!(msg.contains(lang::NODES_DUPLICATE_NAME))
            }
            _ => panic!("Expected Error::Nodes with NODES_DUPLICATE_NAME"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_to_string_single() {
        use crate::nodes::NodesBuilder;
        let nodes = NodesBuilder::new().add_node("ECM").build().unwrap();
        assert_eq!(alloc::format!("{}", nodes), "ECM");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_to_string_multiple() {
        use crate::nodes::NodesBuilder;
        let nodes = NodesBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .add_node("BCM")
            .build()
            .unwrap();
        assert_eq!(alloc::format!("{}", nodes), "ECM TCM BCM");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_to_dbc_string() {
        use crate::nodes::NodesBuilder;
        let nodes_single = NodesBuilder::new().add_node("ECM").build().unwrap();
        assert_eq!(nodes_single.to_dbc_string(), "BU_: ECM");

        let nodes_multiple = NodesBuilder::new()
            .add_node("ECM")
            .add_node("TCM")
            .add_node("BCM")
            .build()
            .unwrap();
        assert_eq!(nodes_multiple.to_dbc_string(), "BU_: ECM TCM BCM");
    }

    #[test]
    fn test_nodes_parse_duplicate() {
        let line = b"BU_: ECM TCM ECM";
        let mut parser = Parser::new(line).unwrap();
        let result = Nodes::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => assert!(msg == lang::NODES_DUPLICATE_NAME),
            _ => panic!("Expected ParseError::Version"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_too_many() {
        use crate::nodes::NodesBuilder;
        // Create a builder with 257 nodes (exceeds limit of 256)
        let mut builder = NodesBuilder::new();
        for i in 0..257 {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.build();
        assert!(result.is_err());
        use crate::Error;
        match result.unwrap_err() {
            Error::Nodes(msg) => {
                assert!(msg.contains(lang::NODES_TOO_MANY));
            }
            _ => panic!("Expected Error::Nodes"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_nodes_at_limit() {
        use crate::nodes::NodesBuilder;
        // Create a builder with exactly 256 nodes (at the limit)
        let mut builder = NodesBuilder::new();
        for i in 0..256 {
            builder = builder.add_node(format!("Node{i}"));
        }
        let result = builder.build();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 256);
    }
}
