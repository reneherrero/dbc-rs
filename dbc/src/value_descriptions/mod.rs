mod core;

#[cfg(feature = "std")]
mod builder;

use crate::compat::ValueDescEntries;
#[cfg(feature = "std")]
pub use builder::ValueDescriptionsBuilder;

/// Value descriptions for a signal.
///
/// Maps numeric signal values to human-readable text descriptions.
/// For example, a gear position signal might map:
/// - 0 -> "Park"
/// - 1 -> "Reverse"
/// - 2 -> "Neutral"
/// - 3 -> "Drive"
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 100 EngineData : 8 ECM
///  SG_ GearPosition : 0|8@1+ (1,0) [0|5] "" *
///
/// VAL_ 100 GearPosition 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" ;
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// let message = dbc.messages().iter().find(|m| m.id() == 100).unwrap();
/// let signal = message.signals().find("GearPosition").unwrap();
///
/// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
///     if let Some(description) = value_descriptions.get(0) {
///         println!("Value 0 means: {}", description);
///     }
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueDescriptions {
    entries: ValueDescEntries,
}
