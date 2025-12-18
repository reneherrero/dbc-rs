use crate::{MessageBuilder, NodesBuilder, ValueDescriptionsBuilder, VersionBuilder};
use std::collections::BTreeMap;

/// Builder for constructing `Dbc` instances programmatically.
///
/// This builder allows you to create DBC files without parsing from a string.
/// It requires the `std` feature to be enabled.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{DbcBuilder, NodesBuilder, MessageBuilder, SignalBuilder, VersionBuilder};
///
/// let nodes = NodesBuilder::new()
///     .add_node("ECM")
///     .add_node("TCM");
///
/// let signal = SignalBuilder::new()
///     .name("RPM")
///     .start_bit(0)
///     .length(16);
///
/// let message = MessageBuilder::new()
///     .id(256)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .add_signal(signal);
///
/// let dbc = DbcBuilder::new()
///     .version(VersionBuilder::new().version("1.0"))
///     .nodes(nodes)
///     .add_message(message)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct DbcBuilder {
    version: VersionBuilder,
    nodes: NodesBuilder,
    messages: Vec<MessageBuilder>,
    value_descriptions: BTreeMap<(Option<u32>, String), ValueDescriptionsBuilder>,
}

// Include modules for additional functionality
mod build;
mod impls;
mod methods;
