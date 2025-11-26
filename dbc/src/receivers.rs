use alloc::boxed::Box;
use alloc::vec::Vec;

/// Receiver specification for a signal.
///
/// Defines which nodes on the CAN bus should receive and process this signal.
///
/// # Limits
///
/// For security reasons (`DoS` protection), the maximum number of receiver nodes
/// per signal is **64**. Attempting to parse a signal with more than 64 receiver
/// nodes will result in a validation error.
#[derive(Debug, Clone, PartialEq)]
pub enum Receivers {
    /// Signal is broadcast to all nodes.
    Broadcast,
    /// Signal is sent to specific nodes.
    ///
    /// The vector contains at most 64 node names (`DoS` protection limit).
    Nodes(Vec<Box<str>>),
    /// Signal has no receivers specified.
    None,
}
