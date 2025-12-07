//! Mock kernel alloc module for testing kernel feature compatibility
//!
//! This module provides a mock implementation of kernel::alloc APIs
//! that matches the real kernel alloc API pattern.
//!
//! The real kernel uses the standard `alloc` crate with infallible methods
//! (`from_str`, `push_str`, `push`, `extend_from_slice`) and separate
//! fallible reservation methods (`try_reserve`, `try_reserve_exact`).

// Mock kernel::alloc module
// Matches the real kernel alloc API pattern: infallible operations with
// separate fallible reservation methods
pub mod alloc {
    // TryReserveError type matching the real API
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TryReserveError {
        kind: TryReserveErrorKind,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TryReserveErrorKind {
        CapacityOverflow,
        AllocError { layout: core::alloc::Layout },
    }

    impl TryReserveError {
        pub fn kind(&self) -> &TryReserveErrorKind {
            &self.kind
        }
    }

    impl core::fmt::Display for TryReserveError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match &self.kind {
                TryReserveErrorKind::CapacityOverflow => {
                    write!(f, "capacity overflow")
                }
                TryReserveErrorKind::AllocError { .. } => {
                    write!(f, "memory allocation failed")
                }
            }
        }
    }

    pub mod string {
        use core::fmt;

        #[derive(Debug, PartialEq, Eq)]
        pub struct String {
            inner: crate::alloc::string::String,
        }

        impl String {
            /// Create a String from a string slice (infallible, matches real API)
            pub fn from_str(s: &str) -> Self {
                Self {
                    inner: crate::alloc::string::String::from(s),
                }
            }

            pub fn as_str(&self) -> &str {
                self.inner.as_str()
            }

            /// Push a string slice (infallible, matches real API)
            pub fn push_str(&mut self, s: &str) {
                self.inner.push_str(s);
            }

            /// Try to reserve additional capacity (fallible, matches real API)
            pub fn try_reserve(&mut self, additional: usize) -> Result<(), super::TryReserveError> {
                self.inner.try_reserve(additional).map_err(|_| super::TryReserveError {
                    kind: super::TryReserveErrorKind::AllocError {
                        layout: core::alloc::Layout::from_size_align(additional, 1)
                            .unwrap_or_else(|_| unreachable!()),
                    },
                })
            }

            /// Try to reserve exact capacity (fallible, matches real API)
            pub fn try_reserve_exact(
                &mut self,
                additional: usize,
            ) -> Result<(), super::TryReserveError> {
                self.inner.try_reserve_exact(additional).map_err(|_| super::TryReserveError {
                    kind: super::TryReserveErrorKind::AllocError {
                        layout: core::alloc::Layout::from_size_align(additional, 1)
                            .unwrap_or_else(|_| unreachable!()),
                    },
                })
            }
        }

        impl fmt::Display for String {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.inner.fmt(f)
            }
        }

        impl Clone for String {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                }
            }
        }

        impl core::ops::Deref for String {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.inner.as_str()
            }
        }

        impl String {
            pub fn into_boxed_str(self) -> crate::alloc::boxed::Box<str> {
                self.inner.into_boxed_str()
            }
        }

        // Implement From<&str> for conversion (matches real API)
        impl core::convert::From<&str> for String {
            fn from(s: &str) -> Self {
                Self::from_str(s)
            }
        }
    }

    pub mod vec {
        #[derive(Debug, Clone)]
        pub struct Vec<T> {
            inner: crate::alloc::vec::Vec<T>,
        }

        impl<T> Default for Vec<T> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<T> Vec<T> {
            pub fn new() -> Self {
                Self {
                    inner: crate::alloc::vec::Vec::new(),
                }
            }

            /// Push an item (infallible, matches real API)
            pub fn push(&mut self, item: T) {
                self.inner.push(item);
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            /// Extend from slice (infallible, matches real API)
            pub fn extend_from_slice(&mut self, other: &[T])
            where
                T: Clone,
            {
                self.inner.extend_from_slice(other);
            }

            /// Try to reserve additional capacity (fallible, matches real API)
            pub fn try_reserve(&mut self, additional: usize) -> Result<(), super::TryReserveError> {
                self.inner.try_reserve(additional).map_err(|_| super::TryReserveError {
                    kind: super::TryReserveErrorKind::AllocError {
                        layout: core::alloc::Layout::from_size_align(additional, 1)
                            .unwrap_or_else(|_| unreachable!()),
                    },
                })
            }

            /// Try to reserve exact capacity (fallible, matches real API)
            pub fn try_reserve_exact(
                &mut self,
                additional: usize,
            ) -> Result<(), super::TryReserveError> {
                self.inner.try_reserve_exact(additional).map_err(|_| super::TryReserveError {
                    kind: super::TryReserveErrorKind::AllocError {
                        layout: core::alloc::Layout::from_size_align(additional, 1)
                            .unwrap_or_else(|_| unreachable!()),
                    },
                })
            }

            pub fn into_boxed_slice(self) -> crate::alloc::boxed::Box<[T]> {
                self.inner.into_boxed_slice()
            }

            pub fn clear(&mut self) {
                self.inner.clear();
            }

            pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
                self.inner.extend(iter);
            }
        }

        impl<T> IntoIterator for Vec<T> {
            type Item = T;
            type IntoIter = crate::alloc::vec::IntoIter<T>;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl<T> core::iter::FromIterator<T> for Vec<T> {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                Self {
                    inner: crate::alloc::vec::Vec::from_iter(iter),
                }
            }
        }

        impl<T> core::ops::Deref for Vec<T> {
            type Target = [T];
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<T> core::ops::DerefMut for Vec<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }

        impl<T> core::convert::Into<crate::alloc::vec::Vec<T>> for Vec<T> {
            fn into(self) -> crate::alloc::vec::Vec<T> {
                self.inner
            }
        }
    }
}
