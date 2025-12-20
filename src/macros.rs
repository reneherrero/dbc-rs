//! Macros for reducing builder boilerplate.
//!
//! This module provides macros that generate common builder patterns,
//! reducing code duplication and ensuring consistency across all builders.

/// Generates a scalar field setter method.
///
/// Creates a setter method that takes ownership of `self`, sets a field to `Some(value)`,
/// and returns `self` for method chaining.
///
/// # Arguments
///
/// * `$field` - The field name
/// * `$type` - The field type
/// * `$doc` - Documentation string for the setter
///
/// # Examples
///
/// ```ignore
/// impl MessageBuilder {
///     builder_setter!(id, u32, "Sets the CAN message ID");
/// }
///
/// // Expands to:
/// // /// Sets the CAN message ID
/// // #[must_use = "builder method returns modified builder"]
/// // pub fn id(mut self, id: u32) -> Self {
/// //     self.id = Some(id);
/// //     self
/// // }
/// ```
#[macro_export]
macro_rules! builder_setter {
    ($field:ident, $type:ty, $doc:expr) => {
        #[doc = $doc]
        #[must_use = "builder method returns modified builder"]
        pub fn $field(mut self, $field: $type) -> Self {
            self.$field = Some($field);
            self
        }
    };
}

/// Generates a string field setter method.
///
/// Creates a setter method that accepts `impl AsRef<str>`, converts it to `String`,
/// wraps it in `Some`, and returns `self` for method chaining.
///
/// # Arguments
///
/// * `$field` - The field name
/// * `$doc` - Documentation string for the setter
///
/// # Examples
///
/// ```ignore
/// impl MessageBuilder {
///     builder_string_setter!(name, "Sets the message name");
/// }
///
/// // Expands to:
/// // /// Sets the message name
/// // #[must_use = "builder method returns modified builder"]
/// // pub fn name(mut self, name: impl AsRef<str>) -> Self {
/// //     self.name = Some(name.as_ref().to_string());
/// //     self
/// // }
/// ```
#[macro_export]
macro_rules! builder_string_setter {
    ($field:ident, $doc:expr) => {
        #[doc = $doc]
        #[must_use = "builder method returns modified builder"]
        pub fn $field(mut self, $field: impl AsRef<str>) -> Self {
            self.$field = Some($field.as_ref().to_string());
            self
        }
    };
}

/// Extracts and validates a required `Option<T>` field.
///
/// This macro simplifies the common pattern of extracting a required field from an `Option`,
/// returning an error if the field is `None`.
///
/// # Arguments
///
/// * `$field` - The `Option<T>` field to extract
/// * `$error` - The error to return if the field is `None`
///
/// # Examples
///
/// ```ignore
/// fn extract_fields(self) -> Result<(u32, String)> {
///     let id = required_field!(self.id, Error::message(Error::MESSAGE_ID_REQUIRED))?;
///     let name = required_field!(self.name, Error::message(Error::MESSAGE_NAME_EMPTY))?;
///     Ok((id, name))
/// }
/// ```
#[macro_export]
macro_rules! required_field {
    ($field:expr, $error:expr) => {
        $field.ok_or_else(|| $error)
    };
}
