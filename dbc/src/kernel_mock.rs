//! Mock kernel alloc module for testing kernel feature compatibility
//!
//! This module provides a mock implementation of kernel::alloc APIs
//! to test compilation and identify required changes.
//!
//! Note: This is a simplified mock. The real kernel alloc API uses
//! Result types for fallible operations, but we'll use a simpler
//! approach for initial testing.

#![cfg(feature = "kernel")]

// Mock kernel::alloc module
// In the real kernel, these return Result types, but for testing
// we'll use a simpler approach that mirrors alloc crate APIs
pub mod alloc {
    pub mod string {
        use core::fmt;

        pub struct String {
            inner: crate::alloc::string::String,
        }

        impl String {
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

        impl core::ops::Deref for String {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.inner.as_str()
            }
        }
    }

    pub mod vec {
        pub struct Vec<T> {
            inner: crate::alloc::vec::Vec<T>,
        }

        impl<T> Vec<T> {
            pub fn new() -> Self {
                Self {
                    inner: crate::alloc::vec::Vec::new(),
                }
            }

            pub fn try_push(&mut self, item: T) -> Result<(), ()> {
                self.inner.push(item);
                Ok(())
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
            where
                T: Clone,
            {
                self.inner.extend_from_slice(other);
                Ok(())
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
