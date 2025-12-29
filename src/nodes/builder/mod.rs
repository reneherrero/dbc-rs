use std::{string::String, vec::Vec};

/// Builder for creating `Nodes` programmatically.
///
/// This builder allows you to construct node lists when building DBC files
/// programmatically. It validates that node names are unique and within limits.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::NodesBuilder;
///
/// let nodes = NodesBuilder::new()
///     .add_node("ECM")
///     .add_node("TCM")
///     .add_node("BCM")
///     .build()?;
///
/// assert_eq!(nodes.len(), 3);
/// assert!(nodes.contains("ECM"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Adding Comments
///
/// You can add comments to nodes using `add_node_with_comment`:
///
/// ```rust,no_run
/// use dbc_rs::NodesBuilder;
///
/// let nodes = NodesBuilder::new()
///     .add_node_with_comment("ECM", "Engine Control Module")
///     .add_node("TCM")
///     .build()?;
///
/// assert_eq!(nodes.node_comment("ECM"), Some("Engine Control Module"));
/// assert_eq!(nodes.node_comment("TCM"), None);
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Validation
///
/// The builder validates:
/// - Maximum of 256 nodes (DoS protection)
/// - All node names must be unique (case-sensitive)
/// - Maximum 32 characters per node name
/// - Maximum number of nodes (implementation limit for DoS protection)
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct NodesBuilder {
    nodes: Vec<(String, Option<String>)>,
}

mod build;
mod impls;
