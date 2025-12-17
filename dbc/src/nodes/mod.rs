mod core;
mod parse;
#[cfg(feature = "std")]
mod serialize;

#[cfg(feature = "std")]
mod builder;

use crate::{
    MAX_NAME_SIZE, MAX_NODES,
    compat::{String, Vec},
};
#[cfg(feature = "std")]
pub use builder::NodesBuilder;

/// Represents a collection of node (ECU) names from a DBC file.
///
/// The `BU_` statement in a DBC file lists all nodes (ECUs) on the CAN bus.
/// This struct stores the node names as owned, fixed-capacity strings.
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
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Nodes {
    nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }>,
}
