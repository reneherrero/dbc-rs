mod impls;
mod parse;
#[cfg(feature = "std")]
mod std;
mod validate;

#[cfg(feature = "std")]
mod builder;

use crate::{
    MAX_NODES,
    compat::{Comment, Name, Vec},
};
#[cfg(feature = "std")]
pub use builder::NodesBuilder;

/// Represents a single node (ECU) with its name and optional comment.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Node {
    name: Name,
    comment: Option<Comment>,
}

impl Node {
    /// Creates a new node with the given name and no comment.
    #[inline]
    pub(crate) fn new(name: Name) -> Self {
        Self {
            name,
            comment: None,
        }
    }

    /// Creates a new node with the given name and comment.
    #[inline]
    #[cfg(feature = "std")]
    pub(crate) fn with_comment(name: Name, comment: Option<Comment>) -> Self {
        Self { name, comment }
    }

    /// Returns the node name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the node comment, if any.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_str())
    }

    /// Sets the comment for this node.
    #[inline]
    pub(crate) fn set_comment(&mut self, comment: Comment) {
        self.comment = Some(comment);
    }
}

type InnerNodes = Vec<Node, { MAX_NODES }>;

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
/// - All node names must be unique (case-sensitive)
/// - Empty node list is valid (`BU_:`)
/// - Maximum 32 characters per node name by default
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Nodes {
    nodes: InnerNodes,
}
