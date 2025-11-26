use alloc::boxed::Box;
use alloc::vec::Vec;

/// Receiver specification for a signal.
///
/// Defines which nodes on the CAN bus should receive and process this signal.
#[derive(Debug, Clone, PartialEq)]
pub enum Receivers {
    /// Signal is broadcast to all nodes.
    Broadcast,
    /// Signal is sent to specific nodes.
    Nodes(Vec<Box<str>>),
    /// Signal has no receivers specified.
    None,
}
