use super::ValueDescriptions;
use std::{string::String, vec::Vec};

impl ValueDescriptions {
    /// Create ValueDescriptions from a Vec of (value, description) pairs
    pub(crate) fn new(entries: Vec<(u64, String)>) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self { entries }
    }

    /// Get the description for a numeric value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     if let Some(desc) = value_descriptions.get(0) {
    ///         println!("Value 0: {}", desc);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn get(&self, value: u64) -> Option<&str> {
        for (v, desc) in &self.entries {
            if *v == value {
                return Some(desc.as_ref());
            }
        }
        None
    }

    /// Get the number of value descriptions
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if there are any value descriptions
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a value description by index.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index of the value description
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     if let Some((value, description)) = value_descriptions.at(0) {
    ///         println!("Value {}: {}", value, description);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn at(&self, index: usize) -> Option<(u64, &str)> {
        self.entries.get(index).map(|(value, desc)| (*value, desc.as_str()))
    }

    /// Iterate over all value descriptions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     for (value, description) in value_descriptions.iter() {
    ///         println!("{} -> {}", value, description);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = (u64, &str)> + '_ {
        self.entries.iter().map(|(value, desc)| (*value, desc.as_str()))
    }
}
