use crate::Error;
use alloc::{boxed::Box, string::String, vec::Vec};

#[derive(Debug)]
pub struct Nodes {
    nodes: Vec<Box<str>>,
}

impl Nodes {
    /// Create a new Nodes instance from an iterator of node names
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::Nodes;
    ///
    /// let nodes = Nodes::new(&["ECM", "TCM", "BCM"]);
    /// assert!(nodes.contains("ECM"));
    /// assert!(nodes.contains("TCM"));
    /// ```
    pub fn new<I, S>(nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            nodes: nodes.into_iter().map(|s| s.as_ref().into()).collect(),
        }
    }

    /// Create an empty Nodes instance
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::Nodes;
    ///
    /// let nodes = Nodes::empty();
    /// assert!(nodes.is_empty());
    /// ```
    #[inline]
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn parse(nodes: &str) -> Result<Self, Error> {
        let nodes: Vec<Box<str>> = nodes[4..].split_whitespace().map(|s| s.into()).collect();

        Ok(Self { nodes })
    }

    #[inline]
    pub fn nodes(&self) -> Option<&[Box<str>]> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes.as_ref())
        }
    }

    /// Check if a node name exists in the list
    #[inline]
    pub fn contains(&self, node: &str) -> bool {
        self.nodes.iter().any(|n| n.as_ref() == node)
    }

    /// Format nodes as a space-separated string for saving
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
            result.push_str(node.as_ref());
        }
        result
    }

    /// Check if the nodes list is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Format nodes in DBC file format (e.g., `BU_: ECM TCM`)
    ///
    /// Useful for debugging and visualization of the nodes in DBC format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::Nodes;
    ///
    /// let nodes = Nodes::new(&["ECM", "TCM"]);
    /// assert_eq!(nodes.to_dbc_string(), "BU_: ECM TCM");
    /// ```
    pub fn to_dbc_string(&self) -> String {
        let mut result = String::from("BU_:");
        let nodes_str = self.to_string();
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodes_from_valid_line() {
        let line = "BU_: ECM TCM BCM ABS";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(
            node_slice,
            &["ECM".into(), "TCM".into(), "BCM".into(), "ABS".into()]
        );
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = "BU_: ONLYONE";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["ONLYONE".into()]);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = "BU_:   Node1   Node2   ";
        let nodes = Nodes::parse(line).unwrap();
        let node_slice = nodes.nodes().unwrap();
        assert_eq!(node_slice, &["Node1".into(), "Node2".into()]);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = "BU_:";
        let nodes = Nodes::parse(line).unwrap();
        assert!(nodes.nodes().is_none());
    }

    #[test]
    fn test_nodes_new() {
        let nodes = Nodes::new(&["ECM", "TCM", "BCM"]);
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
        assert!(!nodes.contains("ABS"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_from_vec() {
        let node_vec = vec!["Node1", "Node2", "Node3"];
        let nodes = Nodes::new(node_vec);
        assert!(nodes.contains("Node1"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_new_from_slice() {
        let node_slice = &["A", "B", "C"][..];
        let nodes = Nodes::new(node_slice);
        assert!(nodes.contains("A"));
        assert_eq!(nodes.nodes().unwrap().len(), 3);
    }

    #[test]
    fn test_nodes_empty() {
        let nodes = Nodes::empty();
        assert!(nodes.is_empty());
        assert!(nodes.nodes().is_none());
    }

    #[test]
    fn test_nodes_to_string_empty() {
        let nodes = Nodes::empty();
        assert_eq!(nodes.to_string(), "");
    }

    #[test]
    fn test_nodes_to_string_single() {
        let nodes = Nodes::new(&["ECM"]);
        assert_eq!(nodes.to_string(), "ECM");
    }

    #[test]
    fn test_nodes_to_string_multiple() {
        let nodes = Nodes::new(&["ECM", "TCM", "BCM"]);
        assert_eq!(nodes.to_string(), "ECM TCM BCM");
    }

    #[test]
    fn test_nodes_to_dbc_string() {
        let nodes_empty = Nodes::empty();
        assert_eq!(nodes_empty.to_dbc_string(), "BU_:");

        let nodes_single = Nodes::new(&["ECM"]);
        assert_eq!(nodes_single.to_dbc_string(), "BU_: ECM");

        let nodes_multiple = Nodes::new(&["ECM", "TCM", "BCM"]);
        assert_eq!(nodes_multiple.to_dbc_string(), "BU_: ECM TCM BCM");
    }
}
