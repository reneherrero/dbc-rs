//! High-performance DBC wrapper for fast message lookup and decoding.
//!
//! This module provides [`FastDbc`], a wrapper around [`Dbc`] that adds:
//! - O(1) message lookup by CAN ID using `HashMap`
//! - Zero-allocation decoding via [`Message::decode_into`]
//! - Helper methods for buffer sizing
//!
//! # Example
//!
//! ```rust,ignore
//! use dbc_rs::{Dbc, FastDbc};
//!
//! let dbc = Dbc::parse(content)?;
//! let fast = FastDbc::new(dbc);
//!
//! // Pre-allocate buffer based on max signals
//! let mut values = vec![0.0f64; fast.max_signals()];
//!
//! // Hot path - O(1) lookup + zero-alloc decode
//! loop {
//!     let (id, payload) = receive_frame();
//!     if let Some(count) = fast.decode_into(id, &payload, &mut values) {
//!         // values[0..count] contains physical values
//!         // Use fast.get(id).unwrap().signals() to get signal metadata
//!     }
//! }
//! ```

use crate::{Dbc, Message};
use std::collections::HashMap;
use std::sync::Arc;

/// High-performance DBC wrapper with O(1) message lookup.
///
/// Wraps a [`Dbc`] and adds a `HashMap` index for fast message lookup by CAN ID.
/// Use this when you need to decode many frames at high speed.
///
/// Cloning is O(1) due to internal `Arc` usage.
#[derive(Debug, Clone)]
pub struct FastDbc {
    /// Shared inner data (cheap to clone)
    inner: Arc<FastDbcInner>,
}

/// Inner data for FastDbc (shared via Arc).
#[derive(Debug)]
struct FastDbcInner {
    /// The underlying DBC
    dbc: Dbc,
    /// O(1) message lookup by CAN ID (standard IDs stored directly, extended with flag)
    index: HashMap<u32, usize>,
    /// Maximum signals in any single message
    max_signals: usize,
    /// Total signal count across all messages
    total_signals: usize,
}

impl FastDbc {
    /// Create a new FastDbc wrapper from a Dbc.
    ///
    /// This builds a HashMap index for O(1) message lookup.
    pub fn new(dbc: Dbc) -> Self {
        let mut index = HashMap::with_capacity(dbc.messages().len());
        let mut max_signals = 0;
        let mut total_signals = 0;

        for (i, msg) in dbc.messages().iter().enumerate() {
            // Use internal ID with extended flag for correct lookup
            index.insert(msg.id_with_flag(), i);
            let sig_count = msg.signals().len();
            max_signals = max_signals.max(sig_count);
            total_signals += sig_count;
        }

        Self {
            inner: Arc::new(FastDbcInner {
                dbc,
                index,
                max_signals,
                total_signals,
            }),
        }
    }

    /// Get a message by standard (11-bit) CAN ID.
    ///
    /// Returns `None` if no message with this ID exists.
    ///
    /// # Performance
    /// O(1) average case.
    #[inline]
    pub fn get(&self, id: u32) -> Option<&Message> {
        self.inner.index.get(&id).and_then(|&idx| self.inner.dbc.messages().at(idx))
    }

    /// Get a message by extended (29-bit) CAN ID.
    ///
    /// Use this for extended CAN IDs.
    #[inline]
    pub fn get_extended(&self, id: u32) -> Option<&Message> {
        let extended_id = id | Message::EXTENDED_ID_FLAG;
        self.inner
            .index
            .get(&extended_id)
            .and_then(|&idx| self.inner.dbc.messages().at(idx))
    }

    /// Get a message by CAN ID, trying with extended flag if standard not found.
    ///
    /// Single lookup optimization: checks if id exists, then tries with extended flag.
    #[inline]
    pub fn get_any(&self, id: u32) -> Option<&Message> {
        // Try standard first, then extended - but use single index access pattern
        self.inner
            .index
            .get(&id)
            .or_else(|| self.inner.index.get(&(id | Message::EXTENDED_ID_FLAG)))
            .and_then(|&idx| self.inner.dbc.messages().at(idx))
    }

    /// Decode a message by standard CAN ID into the output buffer.
    ///
    /// This is the primary high-speed decode path:
    /// - O(1) message lookup
    /// - Zero allocation
    /// - Direct buffer write
    ///
    /// # Arguments
    /// * `id` - Standard (11-bit) CAN ID
    /// * `data` - Raw CAN payload bytes
    /// * `out` - Output buffer for physical values
    ///
    /// # Returns
    /// Number of signals decoded, or `None` if message not found or payload too short.
    #[inline]
    pub fn decode_into(&self, id: u32, data: &[u8], out: &mut [f64]) -> Option<usize> {
        let msg = self.get(id)?;
        let count = msg.decode_into(data, out);
        if count > 0 { Some(count) } else { None }
    }

    /// Decode a message by extended CAN ID into the output buffer.
    #[inline]
    pub fn decode_extended_into(&self, id: u32, data: &[u8], out: &mut [f64]) -> Option<usize> {
        let msg = self.get_extended(id)?;
        let count = msg.decode_into(data, out);
        if count > 0 { Some(count) } else { None }
    }

    /// Decode raw values by standard CAN ID.
    #[inline]
    pub fn decode_raw_into(&self, id: u32, data: &[u8], out: &mut [i64]) -> Option<usize> {
        let msg = self.get(id)?;
        let count = msg.decode_raw_into(data, out);
        if count > 0 { Some(count) } else { None }
    }

    /// Get the maximum number of signals in any single message.
    ///
    /// Use this to pre-allocate decode buffers.
    #[inline]
    pub fn max_signals(&self) -> usize {
        self.inner.max_signals
    }

    /// Get the total number of signals across all messages.
    #[inline]
    pub fn total_signals(&self) -> usize {
        self.inner.total_signals
    }

    /// Get the number of messages.
    #[inline]
    pub fn message_count(&self) -> usize {
        self.inner.dbc.messages().len()
    }

    /// Check if a message with this standard CAN ID exists.
    #[inline]
    pub fn contains(&self, id: u32) -> bool {
        self.inner.index.contains_key(&id)
    }

    /// Check if a message with this extended CAN ID exists.
    #[inline]
    pub fn contains_extended(&self, id: u32) -> bool {
        self.inner.index.contains_key(&(id | Message::EXTENDED_ID_FLAG))
    }

    /// Get the underlying Dbc.
    #[inline]
    pub fn dbc(&self) -> &Dbc {
        &self.inner.dbc
    }

    /// Consume and return the underlying Dbc.
    ///
    /// Returns the Dbc if this is the only reference, otherwise clones it.
    #[inline]
    pub fn into_dbc(self) -> Dbc {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner.dbc,
            Err(arc) => arc.dbc.clone(),
        }
    }

    /// Iterator over all CAN IDs (with extended flag where applicable).
    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.inner.index.keys().copied()
    }
}

impl From<Dbc> for FastDbc {
    fn from(dbc: Dbc) -> Self {
        Self::new(dbc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_dbc_basic() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "C" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        assert_eq!(fast.message_count(), 1);
        assert_eq!(fast.max_signals(), 2);
        assert_eq!(fast.total_signals(), 2);
        assert!(fast.contains(256));
        assert!(!fast.contains(512));

        let msg = fast.get(256).unwrap();
        assert_eq!(msg.name(), "Engine");
    }

    #[test]
    fn test_fast_dbc_decode_into() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "C" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        // RPM = 2000 (raw 8000), Temp = 50°C (raw 90)
        let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut values = vec![0.0f64; fast.max_signals()];

        let count = fast.decode_into(256, &payload, &mut values).unwrap();

        assert_eq!(count, 2);
        assert_eq!(values[0], 2000.0);
        assert_eq!(values[1], 50.0);
    }

    #[test]
    fn test_fast_dbc_message_not_found() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (1,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);
        let payload = [0x00; 8];
        let mut values = [0.0f64; 8];

        assert!(fast.decode_into(512, &payload, &mut values).is_none());
    }

    #[test]
    fn test_fast_dbc_extended_id() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
        )
        .unwrap();
        // 2147484672 = 0x80000400 = extended ID 0x400

        let fast = FastDbc::new(dbc);

        // Should NOT find by standard ID
        assert!(!fast.contains(0x400));
        assert!(fast.get(0x400).is_none());

        // Should find by extended ID
        assert!(fast.contains_extended(0x400));
        let msg = fast.get_extended(0x400).unwrap();
        assert_eq!(msg.name(), "ExtendedMsg");

        // Decode
        let payload = [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut values = [0.0f64; 8];

        let count = fast.decode_extended_into(0x400, &payload, &mut values).unwrap();
        assert_eq!(count, 1);
        assert_eq!(values[0], 100.0); // 1000 * 0.1
    }

    #[test]
    fn test_fast_dbc_multiple_messages() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Msg1 : 8 ECM
 SG_ Sig1 : 0|8@1+ (1,0) [0|255] "" *
 SG_ Sig2 : 8|8@1+ (1,0) [0|255] "" *

BO_ 512 Msg2 : 8 ECM
 SG_ SigA : 0|16@1+ (1,0) [0|65535] "" *

BO_ 768 Msg3 : 8 ECM
 SG_ SigX : 0|8@1+ (1,0) [0|255] "" *
 SG_ SigY : 8|8@1+ (1,0) [0|255] "" *
 SG_ SigZ : 16|8@1+ (1,0) [0|255] "" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        assert_eq!(fast.message_count(), 3);
        assert_eq!(fast.max_signals(), 3); // Msg3 has most
        assert_eq!(fast.total_signals(), 6);

        assert!(fast.contains(256));
        assert!(fast.contains(512));
        assert!(fast.contains(768));
    }

    #[test]
    fn test_fast_dbc_from_trait() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        let fast: FastDbc = dbc.into();
        assert_eq!(fast.message_count(), 1);
    }

    #[test]
    fn test_fast_dbc_into_dbc() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);
        let dbc_back = fast.into_dbc();

        assert_eq!(dbc_back.messages().len(), 1);
    }
}
