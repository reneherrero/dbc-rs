#[cfg(feature = "std")]
use super::Dbc;

impl Dbc {
    /// Serialize this DBC to a DBC format string
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let dbc_string = dbc.to_dbc_string();
    /// // The string can be written to a file or used elsewhere
    /// assert!(dbc_string.contains("VERSION"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let signal_count: usize = self.messages().iter().map(|m| m.signals().len()).sum();
        let estimated_capacity = 200 + (self.messages().len() * 50) + (signal_count * 100);
        let mut result = String::with_capacity(estimated_capacity);

        // VERSION line
        if let Some(version) = self.version() {
            result.push_str(&version.to_dbc_string());
            result.push_str("\n\n");
        }

        // BU_ line
        result.push_str(&self.nodes().to_dbc_string());
        result.push('\n');

        // BO_ and SG_ lines for each message
        for message in self.messages().iter() {
            result.push('\n');
            result.push_str(&message.to_string_full());
        }

        result
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Dbc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
    }
}
