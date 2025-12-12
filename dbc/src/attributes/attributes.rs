/// Object type for attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeObjectType {
    /// Network/global attribute
    Network,
    /// Node (BU_) attribute
    Node,
    /// Message (BO_) attribute
    Message,
    /// Signal (SG_) attribute
    Signal,
    /// Environment variable (EV_) attribute
    EnvironmentVariable,
}

/// Attribute value type
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValueType {
    /// Integer attribute with min and max
    Int(i64, i64),
    /// Hexadecimal attribute with min and max
    Hex(i64, i64),
    /// Float attribute with min and max
    Float(f64, f64),
    /// String attribute
    String,
    /// Enum attribute with possible values
    Enum(std::vec::Vec<std::string::String>),
}

/// Attribute value
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// Integer value
    Int(i64),
    /// Hexadecimal value
    Hex(i64),
    /// Float value
    Float(f64),
    /// String value
    String(std::string::String),
    /// Enum value
    Enum(std::string::String),
}

/// Attribute definition
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDefinition {
    object_type: AttributeObjectType,
    name: std::string::String,
    value_type: AttributeValueType,
}

impl AttributeDefinition {
    pub(crate) fn new(
        object_type: AttributeObjectType,
        name: std::string::String,
        value_type: AttributeValueType,
    ) -> Self {
        Self {
            object_type,
            name,
            value_type,
        }
    }

    #[must_use]
    pub fn object_type(&self) -> AttributeObjectType {
        self.object_type
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn value_type(&self) -> &AttributeValueType {
        &self.value_type
    }
}

/// Attribute assignment to an object
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    name: std::string::String,
    object_type: AttributeObjectType,
    object_name: Option<std::string::String>,
    object_id: Option<u32>,
    value: AttributeValue,
}

impl Attribute {
    pub(crate) fn new(
        name: std::string::String,
        object_type: AttributeObjectType,
        object_name: Option<std::string::String>,
        object_id: Option<u32>,
        value: AttributeValue,
    ) -> Self {
        Self {
            name,
            object_type,
            object_name,
            object_id,
            value,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn object_type(&self) -> AttributeObjectType {
        self.object_type
    }

    #[must_use]
    pub fn object_name(&self) -> Option<&str> {
        self.object_name.as_deref()
    }

    #[must_use]
    pub fn object_id(&self) -> Option<u32> {
        self.object_id
    }

    #[must_use]
    pub fn value(&self) -> &AttributeValue {
        &self.value
    }
}

/// Default value for an attribute
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDefault {
    name: std::string::String,
    value: AttributeValue,
}

impl AttributeDefault {
    pub(crate) fn new(name: std::string::String, value: AttributeValue) -> Self {
        Self { name, value }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn value(&self) -> &AttributeValue {
        &self.value
    }
}
