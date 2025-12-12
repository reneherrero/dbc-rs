//! A UTF-8–encoded, growable string.

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::compat::vec::Vec;
use crate::error::{Error, Result};
use core::{fmt, hash, str};

#[cfg(feature = "alloc")]
type Inner<const N: usize> = alloc::string::String;
#[cfg(not(feature = "alloc"))]
type Inner<const N: usize> = heapless::String<N>;

/// A UTF-8–encoded, growable string.
///
/// When `heapless` feature is enabled, this is wrapper around `heapless::String`. Otherwise, this
/// is a wrapper around `alloc::string::String`, setting the initial capacity to `N`.
#[derive(Clone, Debug)]
pub struct String<const N: usize>(Inner<N>);

impl<const N: usize> String<N> {
    /// Constructs a new, empty `String` with a capacity of `N` bytes.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert UTF-8 bytes into a `String`.
    #[inline]
    pub fn from_utf8(vec: Vec<u8, N>) -> Result<Self> {
        #[cfg(feature = "alloc")]
        {
            let utf8_str = str::from_utf8(vec.as_slice())
                .map_err(|_| Error::Expected(crate::error::lang::INVALID_UTF8))?;
            Ok(Self(alloc::string::String::from(utf8_str)))
        }
        #[cfg(not(feature = "alloc"))]
        {
            // heapless::String::from_utf8 takes ownership of Vec
            Inner::from_utf8(vec.into_inner())
                .map(Self)
                .map_err(|_| Error::Expected(crate::error::lang::INVALID_UTF8))
        }
    }

    /// Extracts a string slice containing the entire string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the length of this `String`, in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this `String` has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<const N: usize> AsRef<str> for String<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for String<N> {
    type Error = Error;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self> {
        if s.len() > N {
            return Err(Error::Validation(
                crate::error::lang::MAX_NAME_SIZE_EXCEEDED,
            ));
        }
        #[cfg(feature = "alloc")]
        {
            Ok(Self(alloc::string::String::from(s)))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Inner::try_from(s)
                .map(Self)
                .map_err(|_| Error::Validation(crate::error::lang::MAX_NAME_SIZE_EXCEEDED))
        }
    }
}

impl<const N: usize> Default for String<N> {
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

impl<const N: usize> From<Inner<N>> for String<N> {
    #[inline]
    fn from(inner: Inner<N>) -> Self {
        Self(inner)
    }
}

impl<const N: usize> fmt::Display for String<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<const N: usize> hash::Hash for String<N> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<const N1: usize, const N2: usize> PartialEq<String<N2>> for String<N1> {
    #[inline]
    fn eq(&self, other: &String<N2>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<str> for String<N> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> Eq for String<N> {}
