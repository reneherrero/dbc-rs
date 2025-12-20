//! Defines [`BTreeMap`] and associated types.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "alloc"))]
use crate::error::Error;
use crate::error::Result;

#[cfg(feature = "alloc")]
type Inner<K, V, const N: usize> = alloc::collections::BTreeMap<K, V>;
#[cfg(not(feature = "alloc"))]
type Inner<K, V, const N: usize> = heapless::LinearMap<K, V, N>;

/// An ordered map based on a B-Tree (with `alloc`) or a linear map (with `heapless`).
///
/// When `alloc` feature is enabled, this is a wrapper around `alloc::collections::BTreeMap`.
/// When `heapless` feature is enabled, this is a wrapper around `heapless::LinearMap<K, V, N>`.
///
/// Note: With heapless, iteration order is insertion order, not key order.
#[allow(dead_code)]
pub struct BTreeMap<K, V, const N: usize>(Inner<K, V, N>);

// Manual Clone implementation with proper bounds
impl<K, V, const N: usize> Clone for BTreeMap<K, V, N>
where
    K: Clone + Eq,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Manual Debug implementation with proper bounds
impl<K, V, const N: usize> core::fmt::Debug for BTreeMap<K, V, N>
where
    K: core::fmt::Debug + Eq,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("BTreeMap").field(&self.0).finish()
    }
}

#[allow(dead_code)]
impl<K, V, const N: usize> BTreeMap<K, V, N>
where
    K: Ord + Eq,
{
    /// Constructs a new, empty map.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        #[cfg(feature = "alloc")]
        {
            self.0.contains_key(key)
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0.get(key).is_some()
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `Ok(None)` is returned.
    /// If the map did have this key present, the value is updated, and `Ok(Some(old_value))` is returned.
    /// Returns `Err` if the map is at capacity (heapless only).
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        #[cfg(feature = "alloc")]
        {
            Ok(self.0.insert(key, value))
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0
                .insert(key, value)
                .map_err(|_| Error::Validation(crate::error::Error::MAX_NAME_SIZE_EXCEEDED))
        }
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }
}

#[allow(dead_code)]
impl<K, V, const N: usize> BTreeMap<K, V, N>
where
    K: Eq,
{
    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clears the map, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns an iterator over the map's key-value pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter()
    }

    /// Returns an iterator over the map's keys.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        #[cfg(feature = "alloc")]
        {
            self.0.keys()
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0.iter().map(|(k, _)| k)
        }
    }

    /// Returns an iterator over the map's values.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        #[cfg(feature = "alloc")]
        {
            self.0.values()
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0.iter().map(|(_, v)| v)
        }
    }
}

impl<K, V, const N: usize> Default for BTreeMap<K, V, N>
where
    K: Ord + Eq,
{
    #[inline]
    fn default() -> Self {
        #[cfg(feature = "alloc")]
        {
            Self(Inner::new())
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self(Inner::new())
        }
    }
}

impl<K, V, const N: usize> PartialEq for BTreeMap<K, V, N>
where
    K: Ord + Eq,
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "alloc")]
        {
            self.0 == other.0
        }
        #[cfg(not(feature = "alloc"))]
        {
            // For heapless LinearMap, compare by iterating
            if self.len() != other.len() {
                return false;
            }
            self.iter().all(|(k, v)| other.get(k) == Some(v))
        }
    }
}

impl<K, V, const N: usize> Eq for BTreeMap<K, V, N>
where
    K: Ord + Eq,
    V: Eq,
{
}
