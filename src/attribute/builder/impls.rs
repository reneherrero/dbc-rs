//! Builder method implementations for AttributeDefinitionBuilder.

use super::AttributeDefinitionBuilder;
use crate::attribute::{AttributeObjectType, AttributeValueType, EnumValues};
use crate::compat::Name;

impl AttributeDefinitionBuilder {
    /// Creates a new attribute definition builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the attribute name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the object type to Network (global/database level).
    #[must_use]
    pub fn object_type_network(mut self) -> Self {
        self.object_type = AttributeObjectType::Network;
        self
    }

    /// Sets the object type to Node (BU_).
    #[must_use]
    pub fn object_type_node(mut self) -> Self {
        self.object_type = AttributeObjectType::Node;
        self
    }

    /// Sets the object type to Message (BO_).
    #[must_use]
    pub fn object_type_message(mut self) -> Self {
        self.object_type = AttributeObjectType::Message;
        self
    }

    /// Sets the object type to Signal (SG_).
    #[must_use]
    pub fn object_type_signal(mut self) -> Self {
        self.object_type = AttributeObjectType::Signal;
        self
    }

    /// Sets the object type directly.
    #[must_use]
    pub fn object_type(mut self, object_type: AttributeObjectType) -> Self {
        self.object_type = object_type;
        self
    }

    /// Sets the value type to INT with the given range.
    #[must_use]
    pub fn int_type(mut self, min: i64, max: i64) -> Self {
        self.value_type = Some(AttributeValueType::Int { min, max });
        self
    }

    /// Sets the value type to HEX with the given range.
    #[must_use]
    pub fn hex_type(mut self, min: i64, max: i64) -> Self {
        self.value_type = Some(AttributeValueType::Hex { min, max });
        self
    }

    /// Sets the value type to FLOAT with the given range.
    #[must_use]
    pub fn float_type(mut self, min: f64, max: f64) -> Self {
        self.value_type = Some(AttributeValueType::Float { min, max });
        self
    }

    /// Sets the value type to STRING.
    #[must_use]
    pub fn string_type(mut self) -> Self {
        self.value_type = Some(AttributeValueType::String);
        self
    }

    /// Sets the value type to ENUM with the given values.
    ///
    /// # Errors
    ///
    /// Returns an error if any enum value exceeds the maximum name size.
    pub fn enum_type(mut self, values: &[&str]) -> crate::Result<Self> {
        let mut enum_values = EnumValues::new();
        for &value in values {
            let name = Name::try_from(value)
                .map_err(|_| crate::Error::expected(crate::Error::MAX_NAME_SIZE_EXCEEDED))?;
            enum_values.push(name).map_err(|_| {
                crate::Error::expected(crate::Error::ATTRIBUTE_ENUM_VALUES_TOO_MANY)
            })?;
        }
        self.value_type = Some(AttributeValueType::Enum {
            values: enum_values,
        });
        Ok(self)
    }

    /// Sets the value type directly.
    #[must_use]
    pub fn value_type(mut self, value_type: AttributeValueType) -> Self {
        self.value_type = Some(value_type);
        self
    }
}
