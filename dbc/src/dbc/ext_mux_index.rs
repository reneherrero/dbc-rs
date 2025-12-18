//! Index for fast extended multiplexing lookup by (message_id, signal_name).
//!
//! This provides O(log n) lookup instead of O(n) filtering on every decode call.

use crate::compat::{BTreeMap, Name, Vec};
use crate::{ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING};

/// Maximum number of unique (message_id, signal_name) pairs we can index.
const MAX_INDEX_ENTRIES: usize = MAX_EXTENDED_MULTIPLEXING;

/// Maximum number of extended multiplexing entries per signal.
/// A signal might have multiple SG_MUL_VAL_ entries (e.g., one per switch).
const MAX_ENTRIES_PER_SIGNAL: usize = 16;

/// Index key: (message_id, signal_name)
type IndexKey = (u32, Name);

/// Index value: indices into the extended_multiplexing vec
type IndexValue = Vec<usize, MAX_ENTRIES_PER_SIGNAL>;

/// Map type using the compat layer
type IndexMap = BTreeMap<IndexKey, IndexValue, MAX_INDEX_ENTRIES>;

/// Index for fast extended multiplexing lookup.
///
/// Maps `(message_id, signal_name)` to a list of indices into the
/// `extended_multiplexing` vec. This allows O(log n) lookup instead of
/// O(n) filtering on every decode call.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtMuxIndex {
    index: Option<IndexMap>,
}

impl ExtMuxIndex {
    /// Build an index from extended multiplexing entries.
    pub fn build(entries: &[ExtendedMultiplexing]) -> Self {
        if entries.is_empty() {
            return Self { index: None };
        }

        let mut index = IndexMap::new();

        for (i, entry) in entries.iter().enumerate() {
            // Try to get signal_name as Name
            let signal_name: Name = match Name::try_from(entry.signal_name()) {
                Ok(name) => name,
                Err(_) => continue, // Skip if name is too long
            };

            let key = (entry.message_id(), signal_name.clone());

            // Get existing indices or create new
            if let Some(indices) = index.get(&key) {
                // Clone, add, and re-insert (BTreeMap doesn't have get_mut in compat)
                let mut new_indices = indices.clone();
                let _ = new_indices.push(i);
                let _ = index.insert(key, new_indices);
            } else {
                let mut indices = IndexValue::new();
                let _ = indices.push(i);
                let _ = index.insert(key, indices);
            }
        }

        Self { index: Some(index) }
    }

    /// Create an empty index.
    pub fn empty() -> Self {
        Self { index: None }
    }

    /// Look up extended multiplexing entries for a (message_id, signal_name) pair.
    ///
    /// Returns indices into the extended_multiplexing vec, or None if not found.
    #[inline]
    pub fn get(&self, message_id: u32, signal_name: &str) -> Option<&[usize]> {
        let index = self.index.as_ref()?;

        // Create lookup key
        let signal_name_owned: Name = Name::try_from(signal_name).ok()?;
        let key = (message_id, signal_name_owned);

        index.get(&key).map(|v| v.as_slice())
    }
}

impl Default for ExtMuxIndex {
    fn default() -> Self {
        Self::empty()
    }
}

// Tests require alloc for vec![] macro
#[cfg(all(test, feature = "alloc"))]
mod tests {
    extern crate alloc;
    use alloc::vec;

    use super::*;
    use crate::ExtendedMultiplexing;

    fn make_ext_mux(
        msg_id: u32,
        signal: &str,
        switch: &str,
        ranges: &[(u64, u64)],
    ) -> ExtendedMultiplexing {
        let signal_name = Name::try_from(signal).unwrap();
        let switch_name = Name::try_from(switch).unwrap();
        let mut value_ranges = Vec::new();
        for &r in ranges {
            let _ = value_ranges.push(r);
        }
        ExtendedMultiplexing::new(msg_id, signal_name, switch_name, value_ranges)
    }

    #[test]
    fn test_empty_index() {
        let index = ExtMuxIndex::build(&[]);
        assert!(index.get(100, "Signal").is_none());
    }

    #[test]
    fn test_single_entry() {
        let entries = vec![make_ext_mux(100, "Signal_A", "Mux1", &[(0, 10)])];
        let index = ExtMuxIndex::build(&entries);

        // Should find the entry
        let result = index.get(100, "Signal_A");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &[0]);

        // Should not find non-existent entries
        assert!(index.get(100, "Signal_B").is_none());
        assert!(index.get(200, "Signal_A").is_none());
    }

    #[test]
    fn test_multiple_entries_same_signal() {
        // Same signal with multiple switches
        let entries = vec![
            make_ext_mux(100, "Signal_A", "Mux1", &[(0, 10)]),
            make_ext_mux(100, "Signal_A", "Mux2", &[(5, 15)]),
        ];
        let index = ExtMuxIndex::build(&entries);

        let result = index.get(100, "Signal_A");
        assert!(result.is_some());
        let indices = result.unwrap();
        assert_eq!(indices.len(), 2);
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn test_multiple_signals() {
        let entries = vec![
            make_ext_mux(100, "Signal_A", "Mux1", &[(0, 10)]),
            make_ext_mux(100, "Signal_B", "Mux1", &[(5, 15)]),
            make_ext_mux(200, "Signal_A", "Mux1", &[(0, 5)]),
        ];
        let index = ExtMuxIndex::build(&entries);

        assert_eq!(index.get(100, "Signal_A").unwrap(), &[0]);
        assert_eq!(index.get(100, "Signal_B").unwrap(), &[1]);
        assert_eq!(index.get(200, "Signal_A").unwrap(), &[2]);
        assert!(index.get(200, "Signal_B").is_none());
    }
}
