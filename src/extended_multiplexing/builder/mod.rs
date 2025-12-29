/// Builder for creating `ExtendedMultiplexing` programmatically.
///
/// This builder allows you to construct extended multiplexing entries when building DBC files
/// programmatically. Extended multiplexing (SG_MUL_VAL_) entries define which multiplexer
/// switch values activate specific multiplexed signals.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::ExtendedMultiplexingBuilder;
///
/// // Build an extended multiplexing entry
/// let ext_mux = ExtendedMultiplexingBuilder::new()
///     .message_id(500)
///     .signal_name("Signal_A")
///     .multiplexer_switch("Mux1")
///     .add_value_range(0, 5)
///     .add_value_range(10, 15)
///     .build()?;
///
/// assert_eq!(ext_mux.message_id(), 500);
/// assert_eq!(ext_mux.signal_name(), "Signal_A");
/// assert_eq!(ext_mux.multiplexer_switch(), "Mux1");
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct ExtendedMultiplexingBuilder {
    message_id: Option<u32>,
    signal_name: Option<String>,
    multiplexer_switch: Option<String>,
    value_ranges: std::vec::Vec<(u64, u64)>,
}

mod build;
mod impls;
