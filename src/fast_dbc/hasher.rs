//! Fast hasher for CAN ID lookups.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// Fast hasher optimized for u32 CAN IDs (FxHash algorithm).
///
/// Uses multiply-xor which is much faster than SipHash for small integer keys.
/// Not cryptographically secure, but perfect for internal hash tables.
#[derive(Default)]
pub(super) struct FxHasher(u64);

impl Hasher for FxHasher {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        const K: u64 = 0x517cc1b727220a95;
        for byte in bytes {
            self.0 = (self.0.rotate_left(5) ^ (*byte as u64)).wrapping_mul(K);
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        const K: u64 = 0x517cc1b727220a95;
        self.0 = (self.0.rotate_left(5) ^ (i as u64)).wrapping_mul(K);
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Type alias for HashMap with FxHasher.
pub(super) type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
