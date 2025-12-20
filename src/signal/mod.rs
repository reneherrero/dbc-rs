mod decode;
mod encode;
mod impls;
mod parse;
#[cfg(feature = "std")]
mod std;
mod validate;

#[cfg(feature = "std")]
mod builder;

use crate::{ByteOrder, Receivers, compat::Name};
#[cfg(feature = "std")]
pub use builder::SignalBuilder;

/// Position info: (start_bit, length, byte_order, unsigned)
type Position = (u16, u16, ByteOrder, bool);
/// Scaling: (factor, offset)
type Scaling = (f64, f64);
/// Range: (min, max)
type Range = (f64, f64);

/// Represents a CAN signal within a message.
///
/// A `Signal` contains:
/// - A name
/// - Start bit position and length
/// - Byte order (big-endian or little-endian)
/// - Signed/unsigned flag
/// - Factor and offset for physical value conversion
/// - Min/max range
/// - Optional unit string
/// - Receivers (nodes that receive this signal)
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 Engine : 8 ECM
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
/// "#)?;
///
/// let message = dbc.messages().at(0).unwrap();
/// let signal = message.signals().at(0).unwrap();
/// println!("Signal: {} (bits: {}-{})", signal.name(), signal.start_bit(), signal.start_bit() + signal.length() - 1);
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Signal {
    name: Name,
    start_bit: u16,
    length: u16,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<Name>,
    receivers: Receivers,
    /// True if this is a multiplexer switch signal (marked with 'M')
    is_multiplexer_switch: bool,
    /// If this is a multiplexed signal (marked with 'm0', 'm1', etc.), this contains the switch value
    /// None means this is a normal signal (not multiplexed)
    multiplexer_switch_value: Option<u64>,
}
