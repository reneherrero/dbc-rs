//! Builder for attribute definitions.
//!
//! Provides a fluent API for constructing attribute definitions programmatically.

mod build;
mod impls;

use super::{AttributeObjectType, AttributeValueType};

/// Builder for creating [`AttributeDefinition`](super::AttributeDefinition) instances.
///
/// # Example
///
/// ```rust,no_run
/// use dbc_rs::AttributeDefinitionBuilder;
///
/// let attr_def = AttributeDefinitionBuilder::new()
///     .name("GenMsgCycleTime")
///     .object_type_message()
///     .int_type(0, 10000)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Default, Clone)]
pub struct AttributeDefinitionBuilder {
    pub(crate) name: Option<String>,
    pub(crate) object_type: AttributeObjectType,
    pub(crate) value_type: Option<AttributeValueType>,
}
