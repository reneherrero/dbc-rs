use crate::ValueDescriptions;
use std::{
    collections::{BTreeMap, btree_map::Iter},
    string::String,
};

/// Encapsulates the value descriptions map for a DBC
///
/// Value descriptions map signal values to human-readable text descriptions.
/// They can be message-specific (keyed by message_id and signal_name) or global
/// (keyed by None and signal_name, applying to all signals with that name).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ValueDescriptionsList {
    value_descriptions: BTreeMap<(Option<u32>, String), ValueDescriptions>,
}

impl ValueDescriptionsList {
    /// Create ValueDescriptionsList from a BTreeMap
    pub(crate) fn from_map(
        value_descriptions: BTreeMap<(Option<u32>, String), ValueDescriptions>,
    ) -> Self {
        Self { value_descriptions }
    }

    /// Get an iterator over all value descriptions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 100 Engine : 8 ECM
    ///  SG_ Gear : 0|8@1+ (1,0) [0|5] "" *
    ///
    /// VAL_ 100 Gear 0 "Park" 1 "Drive" ;"#)?;
    /// for ((message_id, signal_name), value_descriptions) in dbc.value_descriptions().iter() {
    ///     println!("Message {:?}, Signal {}: {} entries", message_id, signal_name, value_descriptions.len());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = ((Option<u32>, &str), &ValueDescriptions)> + '_ {
        ValueDescriptionsListIter {
            entries: self.value_descriptions.iter(),
        }
    }

    /// Get the number of value description entries
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 100 Engine : 8 ECM
    ///  SG_ Gear : 0|8@1+ (1,0) [0|5] "" *
    ///
    /// VAL_ 100 Gear 0 "Park" 1 "Drive" ;"#)?;
    /// assert_eq!(dbc.value_descriptions().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.value_descriptions.len()
    }

    /// Returns `true` if there are no value descriptions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM")?;
    /// assert!(dbc.value_descriptions().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value_descriptions.is_empty()
    }

    /// Get value descriptions for a specific signal
    ///
    /// This method first tries to find a message-specific value description,
    /// then falls back to a global value description (if message_id is None in the map).
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID
    /// * `signal_name` - The signal name
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 100 Engine : 8 ECM
    ///  SG_ Gear : 0|8@1+ (1,0) [0|5] "" *
    ///
    /// VAL_ 100 Gear 0 "Park" 1 "Drive" ;"#)?;
    /// if let Some(value_descriptions) = dbc.value_descriptions().for_signal(100, "Gear") {
    ///     assert_eq!(value_descriptions.get(0), Some("Park"));
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn for_signal(&self, message_id: u32, signal_name: &str) -> Option<&ValueDescriptions> {
        // First try to find a specific entry for this message_id
        // Then fall back to a global entry (None message_id) that applies to all messages
        // Priority: message-specific > global
        // Note: We can't use get() directly because signal_name is &str but key uses &'a str
        // So we iterate and match by string content
        self.value_descriptions
            .iter()
            .find(|((id, name), _)| {
                name.as_str() == signal_name
                    && match id {
                        Some(specific_id) => *specific_id == message_id,
                        None => false, // Check global entries separately
                    }
            })
            .map(|(_, v)| v)
            .or_else(|| {
                // Fall back to global entry (None message_id)
                self.value_descriptions
                    .iter()
                    .find(|((id, name), _)| id.is_none() && name.as_str() == signal_name)
                    .map(|(_, v)| v)
            })
    }
}

/// Iterator over value descriptions in a ValueDescriptionsList
struct ValueDescriptionsListIter<'a> {
    entries: Iter<'a, (Option<u32>, String), ValueDescriptions>,
}

impl<'a> Iterator for ValueDescriptionsListIter<'a> {
    type Item = ((Option<u32>, &'a str), &'a ValueDescriptions);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(|(k, v)| ((k.0, k.1.as_str()), v))
    }
}
