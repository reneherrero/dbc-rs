//! High-performance DBC wrapper for blazing fast message lookup and decoding.
//!
//! This module provides [`FastDbc`], a wrapper around [`Dbc`] optimized for:
//! - **O(1) message lookup** via direct array indexing for standard CAN IDs
//! - **Pre-computed decode plans** eliminating runtime bit calculations
//! - **Zero-allocation decoding** with optimized hot paths
//! - **Identity transform detection** skipping factor/offset math when possible
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
//! // Hot path - direct array lookup + pre-computed decode
//! loop {
//!     let (id, payload) = receive_frame();
//!     if let Some(count) = fast.decode_into(id, &payload, &mut values) {
//!         // values[0..count] contains physical values
//!     }
//! }
//! ```

mod decode;
mod hasher;

use crate::{ByteOrder, Dbc, Message, Result};
use decode::{DecodePlan, SignalDecode};
use hasher::FxHashMap;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Maximum standard CAN ID for direct array lookup (11-bit = 2048 values).
const MAX_STANDARD_ID: usize = 2048;

// ============================================================================
// FastDbc
// ============================================================================

/// High-performance DBC wrapper with optimized message lookup and decoding.
///
/// # Performance Optimizations
///
/// - **Direct array lookup**: Standard CAN IDs (0-2047) use direct array indexing
/// - **Pre-computed decode plans**: All bit positions, masks, and flags computed at build time
/// - **Identity transform detection**: Skips factor/offset math when factor=1, offset=0
/// - **Cache-friendly layout**: Decode parameters packed for optimal cache usage
/// - **FxHash for extended IDs**: Fast hash function for non-standard IDs
///
/// Cloning is O(1) due to internal `Arc` usage.
#[derive(Clone)]
pub struct FastDbc {
    inner: Arc<FastDbcInner>,
}

struct FastDbcInner {
    /// The underlying DBC
    dbc: Dbc,
    /// Direct lookup table for standard CAN IDs (0-2047)
    /// Value is index into decode_plans, or usize::MAX if not present
    standard_ids: Box<[usize; MAX_STANDARD_ID]>,
    /// Hash map for extended CAN IDs and IDs >= 2048
    extended_ids: FxHashMap<u32, usize>,
    /// Pre-computed decode plans for each message
    decode_plans: Vec<DecodePlan>,
    /// Maximum signals in any single message
    max_signals: usize,
    /// Total signal count across all messages
    total_signals: usize,
}

impl std::fmt::Debug for FastDbc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastDbc")
            .field("message_count", &self.message_count())
            .field("max_signals", &self.max_signals())
            .field("total_signals", &self.total_signals())
            .finish()
    }
}

impl FastDbc {
    /// Load a DBC file from disk and wrap it for fast access.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let dbc = Dbc::from_file(path)?;
        Ok(Self::new(dbc))
    }

    /// Create a new FastDbc wrapper from a Dbc.
    ///
    /// This pre-computes all decode parameters for maximum runtime performance.
    pub fn new(dbc: Dbc) -> Self {
        let mut standard_ids = Box::new([usize::MAX; MAX_STANDARD_ID]);
        let mut extended_ids: FxHashMap<u32, usize> =
            HashMap::with_capacity_and_hasher(16, Default::default());
        let mut decode_plans = Vec::with_capacity(dbc.messages().len());
        let mut max_signals = 0;
        let mut total_signals = 0;

        for (msg_idx, msg) in dbc.messages().iter().enumerate() {
            let plan_idx = decode_plans.len();

            // Build decode plan
            let signals: Vec<SignalDecode> =
                msg.signals().iter().map(SignalDecode::from_signal).collect();

            let sig_count = signals.len();
            max_signals = max_signals.max(sig_count);
            total_signals += sig_count;

            decode_plans.push(DecodePlan {
                message_index: msg_idx,
                min_bytes: msg.min_bytes_required(),
                signals,
            });

            // Index by ID
            let id = msg.id_with_flag();
            if !msg.is_extended() && id < MAX_STANDARD_ID as u32 {
                standard_ids[id as usize] = plan_idx;
            } else {
                extended_ids.insert(id, plan_idx);
            }
        }

        Self {
            inner: Arc::new(FastDbcInner {
                dbc,
                standard_ids,
                extended_ids,
                decode_plans,
                max_signals,
                total_signals,
            }),
        }
    }

    // ========================================================================
    // Message Lookup
    // ========================================================================

    /// Get decode plan index for a standard CAN ID.
    #[inline(always)]
    fn get_plan_index(&self, id: u32) -> Option<usize> {
        if id < MAX_STANDARD_ID as u32 {
            // Direct array lookup - fastest path
            let idx = self.inner.standard_ids[id as usize];
            if idx != usize::MAX { Some(idx) } else { None }
        } else {
            // Fall back to hash map for large IDs
            self.inner.extended_ids.get(&id).copied()
        }
    }

    /// Get decode plan index for an extended CAN ID.
    #[inline(always)]
    fn get_plan_index_extended(&self, id: u32) -> Option<usize> {
        let extended_id = id | Message::EXTENDED_ID_FLAG;
        self.inner.extended_ids.get(&extended_id).copied()
    }

    /// Get a message by standard (11-bit) CAN ID.
    #[inline]
    pub fn get(&self, id: u32) -> Option<&Message> {
        self.get_plan_index(id)
            .map(|idx| &self.inner.decode_plans[idx])
            .and_then(|plan| self.inner.dbc.messages().at(plan.message_index))
    }

    /// Get a message by extended (29-bit) CAN ID.
    #[inline]
    pub fn get_extended(&self, id: u32) -> Option<&Message> {
        self.get_plan_index_extended(id)
            .map(|idx| &self.inner.decode_plans[idx])
            .and_then(|plan| self.inner.dbc.messages().at(plan.message_index))
    }

    /// Get a message by CAN ID, trying extended if standard not found.
    #[inline]
    pub fn get_any(&self, id: u32) -> Option<&Message> {
        self.get(id).or_else(|| self.get_extended(id))
    }

    // ========================================================================
    // High-Speed Decode
    // ========================================================================

    /// Decode a message by standard CAN ID into the output buffer.
    ///
    /// This is the primary high-speed decode path:
    /// - O(1) message lookup via direct array indexing
    /// - Pre-computed decode parameters
    /// - Identity transform detection (skips math when factor=1, offset=0)
    /// - Zero allocation
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
        let plan_idx = self.get_plan_index(id)?;
        let plan = &self.inner.decode_plans[plan_idx];

        if data.len() < plan.min_bytes as usize {
            return None;
        }

        Some(self.decode_with_plan(plan, data, out))
    }

    /// Decode a message by extended CAN ID into the output buffer.
    #[inline]
    pub fn decode_extended_into(&self, id: u32, data: &[u8], out: &mut [f64]) -> Option<usize> {
        let plan_idx = self.get_plan_index_extended(id)?;
        let plan = &self.inner.decode_plans[plan_idx];

        if data.len() < plan.min_bytes as usize {
            return None;
        }

        Some(self.decode_with_plan(plan, data, out))
    }

    /// Decode raw values by standard CAN ID.
    #[inline]
    pub fn decode_raw_into(&self, id: u32, data: &[u8], out: &mut [i64]) -> Option<usize> {
        let plan_idx = self.get_plan_index(id)?;
        let plan = &self.inner.decode_plans[plan_idx];

        if data.len() < plan.min_bytes as usize {
            return None;
        }

        Some(self.decode_raw_with_plan(plan, data, out))
    }

    // ========================================================================
    // Internal Decode Implementation
    // ========================================================================

    /// Decode using pre-computed plan.
    #[inline(always)]
    fn decode_with_plan(&self, plan: &DecodePlan, data: &[u8], out: &mut [f64]) -> usize {
        let mut count = 0;
        for (out_val, sig) in out.iter_mut().zip(plan.signals.iter()) {
            let raw = self.extract_raw(*sig, data);
            *out_val = self.apply_scaling(*sig, raw);
            count += 1;
        }
        count
    }

    /// Decode raw values using pre-computed plan.
    #[inline(always)]
    fn decode_raw_with_plan(&self, plan: &DecodePlan, data: &[u8], out: &mut [i64]) -> usize {
        let mut count = 0;
        for (out_val, sig) in out.iter_mut().zip(plan.signals.iter()) {
            *out_val = self.extract_raw(*sig, data);
            count += 1;
        }
        count
    }

    /// Extract raw signed value from data.
    #[inline(always)]
    fn extract_raw(&self, sig: SignalDecode, data: &[u8]) -> i64 {
        let byte_order = if sig.is_little_endian() {
            ByteOrder::LittleEndian
        } else {
            ByteOrder::BigEndian
        };

        let start_bit = sig.byte_start as usize * 8 + sig.bit_offset as usize;
        let raw_bits = byte_order.extract_bits(data, start_bit, sig.length as usize);

        if sig.is_unsigned() {
            raw_bits as i64
        } else {
            Self::sign_extend(raw_bits, sig.length as usize)
        }
    }

    /// Apply factor and offset scaling.
    #[inline(always)]
    fn apply_scaling(&self, sig: SignalDecode, raw: i64) -> f64 {
        if sig.is_identity() {
            raw as f64
        } else {
            (raw as f64) * sig.factor + sig.offset
        }
    }

    /// Sign-extend a value.
    #[inline(always)]
    fn sign_extend(value: u64, bits: usize) -> i64 {
        let sign_bit = 1u64 << (bits - 1);
        if (value & sign_bit) != 0 {
            let mask = !((1u64 << bits) - 1);
            (value | mask) as i64
        } else {
            value as i64
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

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
        self.inner.decode_plans.len()
    }

    /// Check if a message with this standard CAN ID exists.
    #[inline]
    pub fn contains(&self, id: u32) -> bool {
        self.get_plan_index(id).is_some()
    }

    /// Check if a message with this extended CAN ID exists.
    #[inline]
    pub fn contains_extended(&self, id: u32) -> bool {
        self.get_plan_index_extended(id).is_some()
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

    /// Iterator over all CAN IDs.
    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        // Standard IDs from direct lookup table
        let standard = self
            .inner
            .standard_ids
            .iter()
            .enumerate()
            .filter(|(_, idx)| **idx != usize::MAX)
            .map(|(id, _)| id as u32);

        // Extended IDs from hash map
        let extended = self.inner.extended_ids.keys().copied();

        standard.chain(extended)
    }
}

impl From<Dbc> for FastDbc {
    fn from(dbc: Dbc) -> Self {
        Self::new(dbc)
    }
}

// ============================================================================
// Tests
// ============================================================================

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
    fn test_fast_dbc_identity_transform() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RawValue : 0|16@1+ (1,0) [0|65535] "" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        // Raw value 12345
        let payload = [0x39, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut values = [0.0f64; 1];

        fast.decode_into(256, &payload, &mut values);

        assert_eq!(values[0], 12345.0);
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
    fn test_fast_dbc_large_id() {
        // Test ID >= 2048 (uses hash map)
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 3000 LargeId : 8 ECM
 SG_ Value : 0|16@1+ (1,0) [0|65535] "" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        assert!(fast.contains(3000));
        assert!(!fast.contains(256));

        let payload = [0x39, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut values = [0.0f64; 1];

        let count = fast.decode_into(3000, &payload, &mut values).unwrap();
        assert_eq!(count, 1);
        assert_eq!(values[0], 12345.0);
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

    #[test]
    fn test_fast_dbc_ids_iterator() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 100 Msg1 : 8 ECM
BO_ 200 Msg2 : 8 ECM
BO_ 3000 LargeId : 8 ECM
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);
        let ids: Vec<u32> = fast.ids().collect();

        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&100));
        assert!(ids.contains(&200));
        assert!(ids.contains(&3000));
    }

    #[test]
    fn test_fast_dbc_decode_raw_into() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let fast = FastDbc::new(dbc);

        let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut raw_values = [0i64; 8];

        let count = fast.decode_raw_into(256, &payload, &mut raw_values).unwrap();

        assert_eq!(count, 1);
        assert_eq!(raw_values[0], 8000); // Raw before factor
    }
}
