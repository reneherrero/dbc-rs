use super::SignalBuilder;
use crate::{ByteOrder, Error, ReceiversBuilder, Result, Signal, required_field};

type SignalFields = (
    String,
    u16,
    u16,
    ByteOrder,
    bool,
    f64,
    f64,
    f64,
    f64,
    Option<String>,
    ReceiversBuilder,
);

impl SignalBuilder {
    fn extract_fields(&self) -> Result<SignalFields> {
        let name = required_field!(self.name.clone(), Error::signal(Error::SIGNAL_NAME_EMPTY))?;
        let start_bit = required_field!(
            self.start_bit,
            Error::signal(Error::SIGNAL_START_BIT_REQUIRED)
        )?;
        let length = required_field!(self.length, Error::signal(Error::SIGNAL_LENGTH_REQUIRED))?;
        let byte_order = required_field!(
            self.byte_order,
            Error::signal(Error::SIGNAL_BYTE_ORDER_REQUIRED)
        )?;
        let unsigned = required_field!(
            self.unsigned,
            Error::signal(Error::SIGNAL_UNSIGNED_REQUIRED)
        )?;
        let factor = required_field!(self.factor, Error::signal(Error::SIGNAL_FACTOR_REQUIRED))?;
        let offset = required_field!(self.offset, Error::signal(Error::SIGNAL_OFFSET_REQUIRED))?;
        let min = required_field!(self.min, Error::signal(Error::SIGNAL_MIN_REQUIRED))?;
        let max = required_field!(self.max, Error::signal(Error::SIGNAL_MAX_REQUIRED))?;
        Ok((
            name,
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            self.unit.clone(),
            self.receivers.clone(),
        ))
    }

    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        let (
            name,
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            receivers,
        ) = self.extract_fields()?;

        // Validate start_bit: must be between 0 and 511 (CAN FD maximum is 512 bits)
        if start_bit > 511 {
            return Err(Error::signal(Error::SIGNAL_PARSE_INVALID_START_BIT));
        }

        // Validate that start_bit + length doesn't exceed CAN FD maximum (512 bits)
        // Note: This is a basic sanity check. Full validation (including name, min/max,
        // message DLC bounds, and overlap detection) happens in build() when the signal
        // is actually constructed, to avoid duplicate validation calls.
        let end_bit = start_bit + length - 1; // -1 because length includes the start bit
        if end_bit >= 512 {
            return Err(Error::signal(Error::SIGNAL_EXTENDS_BEYOND_MESSAGE));
        }
        Ok(Self {
            name: Some(name),
            start_bit: Some(start_bit),
            length: Some(length),
            byte_order: Some(byte_order),
            unsigned: Some(unsigned),
            factor: Some(factor),
            offset: Some(offset),
            min: Some(min),
            max: Some(max),
            unit,
            receivers,
        })
    }

    /// Builds and validates the `Signal`.
    ///
    /// Consumes the builder and returns a fully constructed and validated [`Signal`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any required field is missing (name, start_bit, length, byte_order, unsigned, factor, offset, min, max)
    /// - Name exceeds maximum length (32 characters)
    /// - Signal length is invalid (zero or exceeds 64 bits)
    /// - Min value exceeds max value
    /// - Receivers fail to build
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{SignalBuilder, ByteOrder, ReceiversBuilder};
    ///
    /// let signal = SignalBuilder::new()
    ///     .name("EngineSpeed")
    ///     .start_bit(0)
    ///     .length(16)
    ///     .byte_order(ByteOrder::LittleEndian)
    ///     .unsigned(true)
    ///     .factor(0.25)
    ///     .offset(0.0)
    ///     .min(0.0)
    ///     .max(8000.0)
    ///     .unit("rpm")
    ///     .receivers(ReceiversBuilder::new().add_node("TCM"))
    ///     .build()?;
    ///
    /// assert_eq!(signal.name(), "EngineSpeed");
    /// assert_eq!(signal.factor(), 0.25);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<Signal> {
        let (
            name,
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            receivers,
        ) = self.extract_fields()?;
        // Build receivers first (receivers is already ReceiversBuilder)
        let built_receivers = receivers.build()?;
        // Validate before construction
        Signal::validate(&name, length, min, max)?;
        // Use Cow::Owned for owned strings (no leak needed)
        Ok(Signal::new(
            name.into(),
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit.map(|u| u.into()),
            built_receivers,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_signal() -> SignalBuilder {
        SignalBuilder::new()
            .name("TestSignal")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
    }

    #[test]
    fn test_signal_builder_new() {
        let builder = SignalBuilder::new();
        // Default builder should be empty
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_signal_builder_default() {
        let builder = SignalBuilder::default();
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_signal_builder_missing_name() {
        let result = SignalBuilder::new()
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_start_bit() {
        let result = SignalBuilder::new()
            .name("Test")
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_length() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_byte_order() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_unsigned() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_factor() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_offset() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_min() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_missing_max() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_validate_valid() {
        let builder = minimal_signal();
        let result = builder.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_builder_validate_start_bit_too_large() {
        let builder = SignalBuilder::new()
            .name("Test")
            .start_bit(512) // > 511
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_validate_extends_beyond_512_bits() {
        let builder = SignalBuilder::new()
            .name("Test")
            .start_bit(505)
            .length(16) // 505 + 16 - 1 = 520 >= 512
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_validate_at_boundary() {
        // 504 + 8 - 1 = 511 < 512, should pass
        let builder = SignalBuilder::new()
            .name("Test")
            .start_bit(504)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none());

        let result = builder.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_builder_invalid_min_max_range() {
        // min > max should fail
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(100.0)
            .max(0.0) // min > max
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_zero_length() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(0) // Invalid
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_signal_builder_length_too_large() {
        let result = SignalBuilder::new()
            .name("Test")
            .start_bit(0)
            .length(513) // > 512
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
            .build();

        assert!(result.is_err());
    }
}
