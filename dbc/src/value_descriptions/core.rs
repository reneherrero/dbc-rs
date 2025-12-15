use super::ValueDescriptions;
use std::{string::String, vec::Vec};

use crate::MAX_VALUE_DESCRIPTIONS;

impl ValueDescriptions {
    /// Create ValueDescriptions from a slice of (value, description) pairs
    pub(crate) fn from_slice(entries: &[(u64, String)]) -> Self {
        let count = entries.len().min(MAX_VALUE_DESCRIPTIONS);
        let vec_entries: Vec<(u64, String)> =
            entries.iter().take(count).map(|(value, desc)| (*value, desc.clone())).collect();
        Self {
            entries: vec_entries,
        }
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
    #[must_use]
    pub fn get(&self, value: u64) -> Option<&str> {
        for (v, desc) in &self.entries {
            if *v == value {
                return Some(desc.as_ref());
            }
        }
        None
    }

    /// Get the number of value descriptions
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if there are any value descriptions
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
    pub fn iter(&self) -> ValueDescriptionsIter<'_> {
        ValueDescriptionsIter {
            entries: &self.entries,
            pos: 0,
        }
    }
}

/// Iterator over value descriptions
pub struct ValueDescriptionsIter<'a> {
    entries: &'a [(u64, String)],
    pos: usize,
}

impl<'a> Iterator for ValueDescriptionsIter<'a> {
    type Item = (u64, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.entries.len() {
            let entry = &self.entries[self.pos];
            let result = (entry.0, entry.1.clone());
            self.pos += 1;
            Some(result)
        } else {
            None
        }
    }
}
