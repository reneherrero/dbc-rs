use crate::{
    ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, MAX_NODES, Nodes, Version,
    compat::{BTreeMap, Comment, Name, Vec},
};

type ExtendedMultiplexings = Vec<ExtendedMultiplexing, { MAX_EXTENDED_MULTIPLEXING }>;
/// Map of node names to their comments (from CM_ BU_ entries)
type NodeCommentsMap = BTreeMap<Name, Comment, MAX_NODES>;

// Index for fast extended multiplexing lookup by (message_id, signal_name)
// Maps to indices into the extended_multiplexing vec
mod ext_mux_index;
use ext_mux_index::ExtMuxIndex;

// Module declarations
mod messages;
mod value_descriptions_map;

// Include modules for additional functionality
#[cfg(feature = "std")]
mod builder;
mod decode;
mod encode;
mod impls;
mod parse;
#[cfg(feature = "std")]
mod std;
mod validate;

// Re-exports
#[cfg(feature = "std")]
pub use builder::DbcBuilder;
pub use decode::DecodedSignal;
use messages::Messages;
use validate::Validate;
use value_descriptions_map::ValueDescriptionsMap;

/// Represents a complete DBC (CAN database) file.
///
/// A `Dbc` contains:
/// - An optional version string
/// - A list of nodes (ECUs)
/// - A collection of messages with their signals
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM TCM
///
/// BO_ 256 EngineData : 8 ECM
///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// println!("Parsed {} messages", dbc.messages().len());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Dbc {
    version: Option<Version>,
    nodes: Nodes,
    messages: Messages,
    value_descriptions: ValueDescriptionsMap,
    extended_multiplexing: ExtendedMultiplexings,
    /// Index for O(1) extended multiplexing lookup by (message_id, signal_name)
    ext_mux_index: ExtMuxIndex,
    /// Database-level comment from CM_ (general comment)
    comment: Option<Comment>,
    /// Node comments from CM_ BU_ entries
    node_comments: NodeCommentsMap,
}
