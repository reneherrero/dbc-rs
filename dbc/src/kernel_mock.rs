//! Mock kernel alloc module for testing kernel feature compatibility
//!
//! This module provides a mock implementation of kernel::alloc APIs
//! to test compilation and identify required changes.
//!
//! Note: This is a simplified mock. The real kernel alloc API uses
//! Result types for fallible operations, but we'll use a simpler
//! approach for initial testing.

// Mock kernel::alloc module
// In the real kernel, these return Result types, but for testing
// we'll use a simpler approach that mirrors alloc crate APIs
pub mod alloc {
    pub mod string {
        use core::fmt;

        #[derive(Debug, PartialEq, Eq)]
        pub struct String {
            inner: crate::alloc::string::String,
        }

        impl String {
            #[allow(clippy::result_unit_err)] // Matches kernel API pattern
            pub fn try_from(s: &str) -> Result<Self, ()> {
                Ok(Self {
                    inner: crate::alloc::string::String::from(s),
                })
            }

            pub fn as_str(&self) -> &str {
                self.inner.as_str()
            }

            pub fn try_push_str(&mut self, s: &str) -> Result<(), ()> {
                self.inner.push_str(s);
                Ok(())
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

        // Implement ToString for conversion
        impl core::convert::From<&str> for String {
            fn from(s: &str) -> Self {
                Self {
                    inner: crate::alloc::string::String::from(s),
                }
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

            #[allow(clippy::result_unit_err)] // Matches kernel API pattern
            pub fn try_push(&mut self, item: T) -> Result<(), ()> {
                self.inner.push(item);
                Ok(())
            }

            pub fn push(&mut self, item: T) {
                self.inner.push(item);
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            #[allow(clippy::result_unit_err)] // Matches kernel API pattern
            pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
            where
                T: Clone,
            {
                self.inner.extend_from_slice(other);
                Ok(())
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
    }
}
