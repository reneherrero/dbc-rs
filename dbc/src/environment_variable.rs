//! Environment Variable definition (EV_)
//!
//! Defines environment variables for system simulation and remaining bus simulation tools.

/// Environment Variable definition (EV_)
///
/// Defines environment variables for system simulation and remaining bus simulation tools.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct EnvironmentVariable {
    name: std::string::String,
    var_type: EnvironmentVariableType,
    minimum: f64,
    maximum: f64,
    unit: std::string::String,
    initial_value: f64,
    ev_id: u32,
    access_type: EnvironmentVariableAccessType,
    access_nodes: std::vec::Vec<std::string::String>,
}

/// Environment Variable Type
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(feature = "std")]
pub enum EnvironmentVariableType {
    /// Integer (0)
    Integer,
    /// Float (1)
    Float,
    /// String (2)
    String,
}

/// Environment Variable Access Type
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(feature = "std")]
pub enum EnvironmentVariableAccessType {
    /// DUMMY_NODE_VECTOR0 - Unrestricted
    Unrestricted,
    /// DUMMY_NODE_VECTOR1 - Read only
    ReadOnly,
    /// DUMMY_NODE_VECTOR2 - Write only
    WriteOnly,
    /// DUMMY_NODE_VECTOR3 - Read/Write
    ReadWrite,
    /// DUMMY_NODE_VECTOR8000-8003 - String type (OR-ed with 0x8000)
    StringType(u16),
}

#[cfg(feature = "std")]
impl EnvironmentVariable {
    /// Create a new EnvironmentVariable
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: std::string::String,
        var_type: EnvironmentVariableType,
        minimum: f64,
        maximum: f64,
        unit: std::string::String,
        initial_value: f64,
        ev_id: u32,
        access_type: EnvironmentVariableAccessType,
        access_nodes: std::vec::Vec<std::string::String>,
    ) -> Self {
        Self {
            name,
            var_type,
            minimum,
            maximum,
            unit,
            initial_value,
            ev_id,
            access_type,
            access_nodes,
        }
    }

    /// Get the environment variable name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the variable type
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn var_type(&self) -> EnvironmentVariableType {
        self.var_type
    }

    /// Get the minimum value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn minimum(&self) -> f64 {
        self.minimum
    }

    /// Get the maximum value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn maximum(&self) -> f64 {
        self.maximum
    }

    /// Get the unit
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Get the initial value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn initial_value(&self) -> f64 {
        self.initial_value
    }

    /// Get the ev_id (obsolete)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn ev_id(&self) -> u32 {
        self.ev_id
    }

    /// Get the access type
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn access_type(&self) -> EnvironmentVariableAccessType {
        self.access_type
    }

    /// Get the access nodes
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn access_nodes(&self) -> &[std::string::String] {
        &self.access_nodes
    }
}
