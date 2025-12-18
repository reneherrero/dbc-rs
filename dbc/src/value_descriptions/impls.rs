use super::ValueDescriptions;
use crate::compat::ValueDescEntries;

impl ValueDescriptions {
    /// Create ValueDescriptions from a Vec of (value, description) pairs
    pub(crate) fn new(entries: ValueDescEntries) -> Self {
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

    /// Returns the number of value descriptions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" 2 "Neutral" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     assert_eq!(value_descriptions.len(), 3);
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if there are no value descriptions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     assert!(!value_descriptions.is_empty());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_NAME_SIZE;
    use crate::compat::{String, Vec};

    fn create_test_entries() -> ValueDescEntries {
        let mut entries: Vec<(u64, String<{ MAX_NAME_SIZE }>), 64> = Vec::new();
        let _ = entries.push((0, String::try_from("Park").unwrap()));
        let _ = entries.push((1, String::try_from("Reverse").unwrap()));
        let _ = entries.push((2, String::try_from("Neutral").unwrap()));
        let _ = entries.push((3, String::try_from("Drive").unwrap()));
        entries
    }

    #[test]
    fn test_value_descriptions_get() {
        let vd = ValueDescriptions::new(create_test_entries());

        assert_eq!(vd.get(0), Some("Park"));
        assert_eq!(vd.get(1), Some("Reverse"));
        assert_eq!(vd.get(2), Some("Neutral"));
        assert_eq!(vd.get(3), Some("Drive"));
        assert_eq!(vd.get(4), None);
        assert_eq!(vd.get(100), None);
    }

    #[test]
    fn test_value_descriptions_len() {
        let vd = ValueDescriptions::new(create_test_entries());
        assert_eq!(vd.len(), 4);
    }

    #[test]
    fn test_value_descriptions_is_empty() {
        let vd = ValueDescriptions::new(create_test_entries());
        assert!(!vd.is_empty());

        let empty_entries: ValueDescEntries = Vec::new();
        let empty_vd = ValueDescriptions::new(empty_entries);
        assert!(empty_vd.is_empty());
    }

    #[test]
    fn test_value_descriptions_at() {
        let vd = ValueDescriptions::new(create_test_entries());

        assert_eq!(vd.at(0), Some((0, "Park")));
        assert_eq!(vd.at(1), Some((1, "Reverse")));
        assert_eq!(vd.at(2), Some((2, "Neutral")));
        assert_eq!(vd.at(3), Some((3, "Drive")));
        assert_eq!(vd.at(4), None);
        assert_eq!(vd.at(100), None);
    }

    #[test]
    fn test_value_descriptions_iter() {
        let vd = ValueDescriptions::new(create_test_entries());

        let mut collected: Vec<(u64, &str), 64> = Vec::new();
        for item in vd.iter() {
            let _ = collected.push(item);
        }
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], (0, "Park"));
        assert_eq!(collected[1], (1, "Reverse"));
        assert_eq!(collected[2], (2, "Neutral"));
        assert_eq!(collected[3], (3, "Drive"));
    }

    #[test]
    fn test_value_descriptions_empty_iter() {
        let empty_entries: ValueDescEntries = Vec::new();
        let vd = ValueDescriptions::new(empty_entries);

        let mut collected: Vec<(u64, &str), 64> = Vec::new();
        for item in vd.iter() {
            let _ = collected.push(item);
        }
        assert!(collected.is_empty());
    }
}
