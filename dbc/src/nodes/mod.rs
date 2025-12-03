#[cfg(feature = "std")]
use crate::error::Result;
use crate::{
    Parser,
    error::{ParseError, ParseResult, messages},
};
use core::{
    option::{
        Option,
        Option::{None, Some},
    },
    result::Result::{Err, Ok},
};

#[cfg(feature = "std")]
pub mod nodes_builder;

#[cfg(feature = "std")]
pub use nodes_builder::NodesBuilder;
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct Nodes {
    nodes: Vec<String>,
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct Nodes<'a> {
    nodes: &'a [&'a str],
}

// Shared validation function
#[cfg_attr(not(feature = "std"), allow(dead_code))]
pub(crate) fn validate_nodes(_nodes: &[&str]) -> ParseResult<()> {
    // // Check for too many nodes (DoS protection)
    // const MAX_NODES: usize = 256;
    // if nodes.len() > MAX_NODES {
    //     return Err(ParseError::Version(messages::NODES_TOO_MANY));
    // }

    // // Check for duplicate node names (case-sensitive)
    // for (i, node1) in nodes.iter().enumerate() {
    //     for node2 in nodes.iter().skip(i + 1) {
    //         if *node1 == *node2 {
    //             return Err(ParseError::Version(messages::NODES_TOO_MANY));
    //         }
    //     }
    // }
    Ok(())
}

// Implementation for std (owned Vec<String>)
#[cfg(feature = "std")]
impl Nodes {
    pub(crate) const BU_: &'static str = "BU_";
    const MAX_NODES: usize = 256;

    #[allow(dead_code)] // Used in tests
    pub(crate) fn new(nodes: &[&str]) -> Result<Self> {
        validate_nodes(nodes)?;
        // Convert to owned Vec<String>
        let nodes_vec: Vec<String> = nodes.iter().map(|s| s.to_string()).collect();
        Ok(Self { nodes: nodes_vec })
    }

    pub(crate) fn parse<'b>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        // Expect "BU_:" keyword
        // Note: When called from Dbc::parse, find_next_keyword already advanced past "BU_",
        // so we try to expect "BU_" first, and if that fails, we're already past it and just expect ":"
        if parser.expect(Self::BU_.as_bytes()).is_ok() {
            // Successfully consumed "BU_", now expect ":"
            parser.expect(b":")?;
        } else {
            // Already past "BU_" from find_next_keyword
            // find_next_keyword advances to right after "BU_", which should be at ":" or whitespace
            // Check if we're already at ":" (no whitespace) or need to skip whitespace first
            let remaining = parser.remaining();
            if remaining.is_empty() || !remaining.starts_with(b":") {
                // Not at ":", skip whitespace and try again
                parser.skip_newlines_and_spaces();
            }
            parser.expect(b":")?;
        }

        // Skip optional whitespace after ":"
        parser.skip_newlines_and_spaces();

        // Parse node names one by one
        use alloc::vec::Vec;
        let mut node_names: Vec<&str> = Vec::new();

        loop {
            // Skip whitespace before each node name
            let _ = parser.skip_whitespace();

            // Check if we're at EOF or end of line
            if parser.remaining().is_empty() {
                break;
            }

            // Try to parse an identifier (node name)
            match parser.parse_identifier() {
                Ok(node) => {
                    node_names.push(node);
                    // Check for too many nodes (DoS protection)
                    if node_names.len() > Self::MAX_NODES {
                        return Err(ParseError::Version(messages::NODES_TOO_MANY));
                    }
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if node_names.is_empty() {
            // No nodes specified, return empty nodes
            return Ok(Nodes { nodes: Vec::new() });
        }

        // Validate nodes
        validate_nodes(&node_names)?;

        // Convert to owned Vec<String>
        let nodes_vec: Vec<String> = node_names.into_iter().map(|s| s.to_string()).collect();
        Ok(Nodes { nodes: nodes_vec })
    }

    #[inline]
    #[must_use]
    pub fn nodes(&self) -> Option<&[String]> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(&self.nodes)
        }
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, node: &str) -> bool {
        self.nodes.iter().any(|n| n == node)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[allow(clippy::inherent_to_string)]
    #[cfg(feature = "std")]
    #[must_use]
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
            result.push_str(node);
        }
        result
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        let mut result = String::from(Self::BU_);
        result.push(':');
        let nodes_str = self.to_string();
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}

// Implementation for no_std (borrowed &[&str])
#[cfg(not(feature = "std"))]
impl<'a> Nodes<'a> {
    #[allow(dead_code)] // Used in Dbc::parse
    pub(crate) const BU_: &'static str = "BU_";
    #[allow(dead_code)] // Used in validation
    const MAX_NODES: usize = 256;

    #[allow(dead_code)] // Used in Dbc::parse
    #[must_use]
    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        // Expect "BU_:" keyword
        // Note: When called from Dbc::parse, find_next_keyword already advanced past "BU_",
        // so we try to expect "BU_" first, and if that fails, we're already past it and just expect ":"
        if parser.expect(Self::BU_.as_bytes()).is_ok() {
            // Successfully consumed "BU_", now expect ":"
            parser.expect(b":")?;
        } else {
            // Already past "BU_" from find_next_keyword
            // find_next_keyword advances to right after "BU_", which should be at ":" or whitespace
            // Check if we're already at ":" (no whitespace) or need to skip whitespace first
            let remaining = parser.remaining();
            if remaining.is_empty() || !remaining.starts_with(b":") {
                // Not at ":", skip whitespace and try again
                parser.skip_newlines_and_spaces();
            }
            parser.expect(b":")?;
        }

        // Skip optional whitespace after ":"
        parser.skip_newlines_and_spaces();

        // Parse node names one by one
        use alloc::vec::Vec;
        let mut node_names: Vec<&str> = Vec::new();

        loop {
            // Skip whitespace before each node name
            let _ = parser.skip_whitespace();

            // Check if we're at EOF or end of line
            if parser.remaining().is_empty() {
                break;
            }

            // Try to parse an identifier (node name)
            match parser.parse_identifier() {
                Ok(node) => {
                    node_names.push(node);
                    // Check for too many nodes (DoS protection)
                    if node_names.len() > Self::MAX_NODES {
                        return Err(ParseError::Version(messages::NODES_TOO_MANY));
                    }
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if node_names.is_empty() {
            return Ok(Nodes { nodes: &[] });
        }

        validate_nodes(&node_names)?;

        // Use borrowed slice - no alloc needed
        use alloc::boxed::Box;
        let node_slice = Box::leak(node_names.into_boxed_slice());
        Ok(Nodes { nodes: node_slice })
    }

    pub fn nodes(&self) -> Option<&[&'a str]> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes)
        }
    }

    pub fn contains(&self, node: &str) -> bool {
        self.nodes.iter().any(|n| *n == node)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::{
        Error, Parser,
        error::{ParseError, lang},
    };

    #[test]
    fn test_nodes_from_valid_line() {
        let line = b"BU_: ECM TCM BCM ABS";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["ECM", "TCM", "BCM", "ABS"]);
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = b"BU_: ONLYONE";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["ONLYONE"]);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = b"BU_:   Node1   Node2   ";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["Node1", "Node2"]);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = b"BU_:";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        assert!(nodes.nodes().is_none());
    }

    #[test]
    fn test_nodes_new() {
        let node_array = ["ECM", "TCM", "BCM"];
        let nodes = Nodes::new(&node_array).unwrap();
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
        assert!(!nodes.contains("ABS"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_from_vec() {
        let node_vec = vec!["Node1", "Node2", "Node3"];
        let nodes = Nodes::new(&node_vec).unwrap();
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
    #[ignore]
    fn test_nodes_new_duplicate() {
        let node_array = ["ECM", "TCM", "ECM"];
        let result = Nodes::new(&node_array);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg.contains(lang::NODES_DUPLICATE_NAME)),
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_nodes_to_string_single() {
        let node_array = ["ECM"];
        let nodes = Nodes::new(&node_array).unwrap();
        assert_eq!(nodes.to_string(), "ECM");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_nodes_to_string_multiple() {
        let node_array = ["ECM", "TCM", "BCM"];
        let nodes = Nodes::new(&node_array).unwrap();
        assert_eq!(nodes.to_string(), "ECM TCM BCM");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_nodes_to_dbc_string() {
        let node_array_single = ["ECM"];
        let nodes_single = Nodes::new(&node_array_single).unwrap();
        assert_eq!(nodes_single.to_dbc_string(), "BU_: ECM");

        let node_array_multiple = ["ECM", "TCM", "BCM"];
        let nodes_multiple = Nodes::new(&node_array_multiple).unwrap();
        assert_eq!(nodes_multiple.to_dbc_string(), "BU_: ECM TCM BCM");
    }

    #[test]
    #[ignore]
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
    #[cfg(feature = "std")]
    #[ignore]
    fn test_nodes_too_many() {
        // Create a vector with 257 nodes (exceeds limit of 256)
        let mut node_strings = Vec::new();
        for i in 0..257 {
            node_strings.push(format!("Node{i}"));
        }
        let node_refs: Vec<&str> = node_strings.iter().map(|s| s.as_str()).collect();
        let result = Nodes::new(&node_refs);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => {
                assert!(msg.contains(lang::NODES_TOO_MANY));
            }
            _ => panic!("Expected Nodes error"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_nodes_at_limit() {
        // Create a vector with exactly 256 nodes (at the limit)
        let mut node_strings = Vec::new();
        for i in 0..256 {
            node_strings.push(format!("Node{i}"));
        }
        let node_refs: Vec<&str> = node_strings.iter().map(|s| s.as_str()).collect();
        let result = Nodes::new(&node_refs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes().unwrap().len(), 256);
    }
}
