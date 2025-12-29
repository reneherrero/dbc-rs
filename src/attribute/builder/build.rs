//! Build method implementation for AttributeDefinitionBuilder.

use super::AttributeDefinitionBuilder;
use crate::attribute::AttributeDefinition;
use crate::compat::Name;
use crate::{Error, Result};

impl AttributeDefinitionBuilder {
    /// Builds the attribute definition.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The name is not set
    /// - The value type is not set
    /// - The name exceeds the maximum size
    pub fn build(self) -> Result<AttributeDefinition> {
        // Validate name
        let name_str = self.name.ok_or_else(|| Error::expected(Error::ATTRIBUTE_NAME_REQUIRED))?;

        let name = Name::try_from(name_str.as_str())
            .map_err(|_| Error::expected(Error::MAX_NAME_SIZE_EXCEEDED))?;

        // Validate value type
        let value_type = self
            .value_type
            .ok_or_else(|| Error::expected(Error::ATTRIBUTE_VALUE_TYPE_REQUIRED))?;

        Ok(AttributeDefinition::new(name, self.object_type, value_type))
    }
}
