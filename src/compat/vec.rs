//! Defines [`Vec`] and associated types.

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{hash, iter::FromIterator, slice};

use crate::error::{Error, Result};

#[cfg(feature = "alloc")]
type Inner<T, const N: usize> = alloc::vec::Vec<T>;
#[cfg(not(feature = "alloc"))]
type Inner<T, const N: usize> = heapless::Vec<T, N>;

/// A contiguous growable array type.
///
/// When `heapless` feature is enabled, this is wrapper around `heapless::Vec`. Otherwise, this is
/// a wrapper around `alloc::vec::Vec`, setting the initial capacity to `N`.
#[derive(Clone, Debug)]
pub struct Vec<T, const N: usize>(Inner<T, N>);

impl<T, const N: usize> Vec<T, N> {
    /// Constructs a new, empty vector with a capacity of `N`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new vector with a capacity of `N` and fills it with the provided slice.
    #[inline]
    pub fn from_slice(other: &[T]) -> Result<Self>
    where
        T: Clone,
    {
        #[cfg(feature = "alloc")]
        {
            // Optimized: check length upfront, allocate with exact capacity
            if other.len() > N {
                return Err(Error::Validation(
                    crate::error::Error::MAX_NAME_SIZE_EXCEEDED,
                ));
            }
            let mut v = Self(Inner::with_capacity(other.len()));
            v.0.extend_from_slice(other);
            Ok(v)
        }
        #[cfg(not(feature = "alloc"))]
        {
            let mut v = Self::new();
            v.extend_from_slice(other)?;
            Ok(v)
        }
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Extracts a mutable slice containing the entire vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.0
    }

    /// Returns the length of the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clears the vector, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Clones and appends all elements in a slice to the `Vec`.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<()>
    where
        T: Clone,
    {
        #[cfg(feature = "alloc")]
        {
            // Optimized: single upfront capacity check
            let current_len = self.0.len();
            let new_len = current_len.saturating_add(other.len());
            if new_len > N {
                return Err(Error::Validation(
                    crate::error::Error::MAX_NAME_SIZE_EXCEEDED,
                ));
            }
            // Reserve exact capacity if needed (avoids reallocation and over-allocation)
            let current_cap = self.0.capacity();
            if current_cap < new_len {
                // Reserve exactly what we need (new_len - current_cap additional capacity)
                self.0.reserve_exact(new_len - current_cap);
            }
            self.0.extend_from_slice(other);
            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0
                .extend_from_slice(other)
                .map_err(|_| Error::Validation(crate::error::Error::MAX_NAME_SIZE_EXCEEDED))
        }
    }

    /// Appends an `item` to the back of the collection.
    #[inline]
    pub fn push(&mut self, item: T) -> Result<()> {
        #[cfg(feature = "alloc")]
        {
            // Optimized: check length before pushing (avoids capacity check in alloc Vec)
            if self.0.len() >= N {
                return Err(Error::Validation(
                    crate::error::Error::MAX_NAME_SIZE_EXCEEDED,
                ));
            }
            self.0.push(item);
            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.0
                .push(item)
                .map_err(|_| Error::Validation(crate::error::Error::MAX_NAME_SIZE_EXCEEDED))
        }
    }

    /// Returns a reference to an element or subslice, without doing bounds checking.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    /// Returns an iterator over the slice.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the slice.
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.0.iter_mut()
    }

    /// Consumes the Vec and returns the inner representation.
    #[inline]
    #[cfg(not(feature = "alloc"))]
    pub(crate) fn into_inner(self) -> Inner<T, N> {
        self.0
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Vec<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "alloc")]
impl<T, const N: usize> IntoIterator for Vec<T, N> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(not(feature = "alloc"))]
impl<T, const N: usize> IntoIterator for Vec<T, N> {
    type Item = T;
    type IntoIter = heapless::vec::IntoIter<T, N, usize>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const N: usize> FromIterator<T> for Vec<T, N> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        #[cfg(feature = "alloc")]
        {
            let iter = iter.into_iter();
            // Optimized: get size hint once
            let (lower, upper) = iter.size_hint();
            let capacity = upper.unwrap_or(lower).min(N);
            let mut v = alloc::vec::Vec::with_capacity(capacity);
            v.extend(iter);
            Self(v)
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self(Inner::from_iter(iter))
        }
    }
}

impl<T, const N: usize> hash::Hash for Vec<T, N>
where
    T: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T, const N: usize> PartialEq for Vec<T, N>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, const N: usize> Eq for Vec<T, N> where T: Eq {}

impl<T, const N: usize> Default for Vec<T, N> {
    #[inline]
    fn default() -> Self {
        #[cfg(feature = "alloc")]
        {
            Self(Inner::with_capacity(N))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self(Inner::new())
        }
    }
}

impl<T, const N: usize> core::ops::Index<usize> for Vec<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> core::ops::Deref for Vec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
