//! Environment Variable Data (ENVVAR_DATA_/EV_DATA_)
//!
//! Defines binary data storage with specified byte length for environment variables.

/// Environment Variable Data (ENVVAR_DATA_/EV_DATA_)
///
/// Defines binary data storage with specified byte length for environment variables.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct EnvironmentVariableData {
    env_var_name: std::string::String,
    data_size: u32,
}

#[cfg(feature = "std")]
impl EnvironmentVariableData {
    /// Create a new EnvironmentVariableData
    pub(crate) fn new(env_var_name: std::string::String, data_size: u32) -> Self {
        Self {
            env_var_name,
            data_size,
        }
    }

    /// Get the environment variable name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn env_var_name(&self) -> &str {
        &self.env_var_name
    }

    /// Get the data size in bytes
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn data_size(&self) -> u32 {
        self.data_size
    }
}
