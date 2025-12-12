/// A value table in a DBC file
///
/// Value tables map numeric values to human-readable text descriptions.
/// They can be referenced by signals for enumerated values.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueTable {
    name: std::string::String,
    entries: std::vec::Vec<(u64, std::string::String)>,
}

impl ValueTable {
    pub(crate) fn new(
        name: std::string::String,
        entries: std::vec::Vec<(u64, std::string::String)>,
    ) -> Self {
        Self { name, entries }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn entries(&self) -> &[(u64, std::string::String)] {
        self.entries.as_slice()
    }

    /// Get the description for a specific value
    #[must_use]
    pub fn get(&self, value: u64) -> Option<&str> {
        self.entries.iter().find(|(v, _)| *v == value).map(|(_, desc)| desc.as_str())
    }
}
