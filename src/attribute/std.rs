//! Standard library features for attributes.

use super::{
    AttributeDefinition, AttributeObjectType, AttributeTarget, AttributeValue, AttributeValueType,
};
use std::fmt::{Display, Formatter, Result};

impl Display for AttributeObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Network => Ok(()),
            Self::Node => write!(f, "BU_ "),
            Self::Message => write!(f, "BO_ "),
            Self::Signal => write!(f, "SG_ "),
        }
    }
}

impl Display for AttributeValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Int { min, max } => write!(f, "INT {} {}", min, max),
            Self::Hex { min, max } => write!(f, "HEX {} {}", min, max),
            Self::Float { min, max } => write!(f, "FLOAT {} {}", min, max),
            Self::String => write!(f, "STRING"),
            Self::Enum { values } => {
                write!(f, "ENUM ")?;
                let mut first = true;
                for value in values.as_slice() {
                    if !first {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\"", value.as_str())?;
                    first = false;
                }
                Ok(())
            }
        }
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::String(s) => write!(f, "\"{}\"", s.as_str()),
        }
    }
}

impl Display for AttributeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "BA_DEF_ {}\"{}\" {};",
            self.object_type(),
            self.name(),
            self.value_type()
        )
    }
}

impl Display for AttributeTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Network => Ok(()),
            Self::Node(name) => write!(f, "BU_ {} ", name.as_str()),
            Self::Message(id) => write!(f, "BO_ {} ", id),
            Self::Signal(msg_id, sig_name) => {
                write!(f, "SG_ {} {} ", msg_id, sig_name.as_str())
            }
        }
    }
}

impl AttributeDefinition {
    /// Converts the attribute definition to a DBC format string.
    pub fn to_dbc_string(&self) -> std::string::String {
        format!("{}", self)
    }
}

impl AttributeValue {
    /// Converts the attribute value to a DBC format string.
    pub fn to_dbc_string(&self) -> std::string::String {
        format!("{}", self)
    }
}

impl AttributeTarget {
    /// Converts the attribute target to a DBC format string prefix.
    pub fn to_dbc_string(&self) -> std::string::String {
        format!("{}", self)
    }
}
