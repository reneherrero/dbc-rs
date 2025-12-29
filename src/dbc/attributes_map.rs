//! Storage wrappers for DBC attributes.
//!
//! Provides storage for:
//! - Attribute definitions (BA_DEF_)
//! - Attribute defaults (BA_DEF_DEF_)
//! - Attribute values (BA_)

use crate::{
    MAX_ATTRIBUTE_DEFINITIONS, MAX_ATTRIBUTE_VALUES,
    attribute::{
        AttributeDefinition, AttributeDefinitions as AttrDefsVec, AttributeObjectType,
        AttributeTarget, AttributeValue,
    },
    compat::{BTreeMap, Name},
};

/// Type alias for attribute defaults map (attribute_name -> default_value)
type DefaultsMap = BTreeMap<Name, AttributeValue, MAX_ATTRIBUTE_DEFINITIONS>;

/// Type alias for attribute values map ((attribute_name, target) -> value)
type ValuesMap = BTreeMap<(Name, AttributeTarget), AttributeValue, MAX_ATTRIBUTE_VALUES>;

/// Storage for attribute definitions (BA_DEF_).
///
/// Attribute definitions specify the name, object type, and value constraints
/// for attributes that can be assigned to DBC objects.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeDefinitionsMap {
    definitions: AttrDefsVec,
}

impl AttributeDefinitionsMap {
    /// Create a new empty attribute definitions map.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            definitions: AttrDefsVec::new(),
        }
    }

    /// Create from a vector of definitions.
    pub(crate) fn from_vec(definitions: AttrDefsVec) -> Self {
        Self { definitions }
    }

    /// Get an iterator over all attribute definitions.
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &AttributeDefinition> {
        self.definitions.as_slice().iter()
    }

    /// Get the number of attribute definitions.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns `true` if there are no attribute definitions.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Get an attribute definition by name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get(&self, name: &str) -> Option<&AttributeDefinition> {
        self.definitions.as_slice().iter().find(|def| def.name() == name)
    }

    /// Get an attribute definition by name and object type.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get_for_type(
        &self,
        name: &str,
        object_type: AttributeObjectType,
    ) -> Option<&AttributeDefinition> {
        self.definitions
            .as_slice()
            .iter()
            .find(|def| def.name() == name && def.object_type() == object_type)
    }
}

/// Storage for attribute default values (BA_DEF_DEF_).
///
/// Default values are used when an attribute is not explicitly assigned
/// to an object.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeDefaultsMap {
    defaults: DefaultsMap,
}

impl AttributeDefaultsMap {
    /// Create a new empty attribute defaults map.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            defaults: DefaultsMap::new(),
        }
    }

    /// Create from a BTreeMap of defaults.
    pub(crate) fn from_map(defaults: DefaultsMap) -> Self {
        Self { defaults }
    }

    /// Get an iterator over all attribute defaults.
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &AttributeValue)> {
        self.defaults.iter().map(|(name, value)| (name.as_str(), value))
    }

    /// Get the number of attribute defaults.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.defaults.len()
    }

    /// Returns `true` if there are no attribute defaults.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.defaults.is_empty()
    }

    /// Get the default value for an attribute by name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get(&self, name: &str) -> Option<&AttributeValue> {
        self.defaults.iter().find(|(n, _)| n.as_str() == name).map(|(_, v)| v)
    }
}

/// Storage for attribute values (BA_).
///
/// Attribute values are the actual values assigned to specific DBC objects.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeValuesMap {
    values: ValuesMap,
}

impl AttributeValuesMap {
    /// Create a new empty attribute values map.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            values: ValuesMap::new(),
        }
    }

    /// Create from a BTreeMap of values.
    pub(crate) fn from_map(values: ValuesMap) -> Self {
        Self { values }
    }

    /// Get an iterator over all attribute values.
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = ((&str, &AttributeTarget), &AttributeValue)> {
        self.values
            .iter()
            .map(|((name, target), value)| ((name.as_str(), target), value))
    }

    /// Get the number of attribute values.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if there are no attribute values.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get an attribute value by name and target.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get(&self, name: &str, target: &AttributeTarget) -> Option<&AttributeValue> {
        self.values
            .iter()
            .find(|((n, t), _)| n.as_str() == name && t == target)
            .map(|(_, v)| v)
    }

    /// Get a network-level attribute value by name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get_network(&self, name: &str) -> Option<&AttributeValue> {
        self.get(name, &AttributeTarget::Network)
    }

    /// Get a node attribute value by node name and attribute name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get_node(&self, node_name: &str, attr_name: &str) -> Option<&AttributeValue> {
        self.values
            .iter()
            .find(|((n, t), _)| {
                n.as_str() == attr_name
                    && matches!(t, AttributeTarget::Node(name) if name.as_str() == node_name)
            })
            .map(|(_, v)| v)
    }

    /// Get a message attribute value by message ID and attribute name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get_message(&self, message_id: u32, attr_name: &str) -> Option<&AttributeValue> {
        self.values
            .iter()
            .find(|((n, t), _)| {
                n.as_str() == attr_name
                    && matches!(t, AttributeTarget::Message(id) if *id == message_id)
            })
            .map(|(_, v)| v)
    }

    /// Get a signal attribute value by message ID, signal name, and attribute name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn get_signal(
        &self,
        message_id: u32,
        signal_name: &str,
        attr_name: &str,
    ) -> Option<&AttributeValue> {
        self.values
            .iter()
            .find(|((n, t), _)| {
                n.as_str() == attr_name
                    && matches!(t, AttributeTarget::Signal(msg_id, sig_name)
                        if *msg_id == message_id && sig_name.as_str() == signal_name)
            })
            .map(|(_, v)| v)
    }
}
