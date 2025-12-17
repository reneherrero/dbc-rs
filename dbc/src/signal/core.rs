use super::Signal;
use crate::{ByteOrder, Error, Receivers, Result};

#[cfg(feature = "std")]
use crate::MAX_NAME_SIZE;
#[cfg(feature = "std")]
use crate::compat::String;

impl Signal {
    pub(crate) fn validate(name: &str, length: u16, min: f64, max: f64) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::Validation(Error::SIGNAL_NAME_EMPTY));
        }

        // Validate length: must be between 1 and 512 bits
        // - Classic CAN (2.0A/2.0B): DLC up to 8 bytes (64 bits)
        // - CAN FD: DLC up to 64 bytes (512 bits)
        // Signal length is validated against message DLC in Message::validate
        // Note: name is parsed before this validation, so we can include it in error messages
        if length == 0 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_SMALL));
        }
        if length > 512 {
            return Err(Error::Validation(Error::SIGNAL_LENGTH_TOO_LARGE));
        }

        // Note: start_bit validation (boundary checks and overlap detection) is done in
        // Message::validate, not here, because:
        // 1. The actual message size depends on DLC (1-64 bytes for CAN FD)
        // 2. Overlap detection requires comparing multiple signals
        // 3. This allows signals to be created independently and validated when added to a message

        // Validate min <= max
        if min > max {
            return Err(Error::Validation(Error::INVALID_RANGE));
        }

        Ok(())
    }

    #[cfg(feature = "std")]
    #[allow(clippy::too_many_arguments)] // Internal method, builder pattern is the public API
    pub(crate) fn new(
        name: String<{ MAX_NAME_SIZE }>,
        start_bit: u16,
        length: u16,
        byte_order: ByteOrder,
        unsigned: bool,
        factor: f64,
        offset: f64,
        min: f64,
        max: f64,
        unit: Option<String<{ MAX_NAME_SIZE }>>,
        receivers: Receivers,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
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
            is_multiplexer_switch: false,
            multiplexer_switch_value: None,
        }
    }

    #[inline]
    #[must_use = "return value should be checked"]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn start_bit(&self) -> u16 {
        self.start_bit
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> u16 {
        self.length
    }

    #[inline]
    #[must_use]
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    #[inline]
    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        self.unsigned
    }

    #[inline]
    #[must_use]
    pub fn factor(&self) -> f64 {
        self.factor
    }

    #[inline]
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    #[inline]
    #[must_use]
    pub fn min(&self) -> f64 {
        self.min
    }

    #[inline]
    #[must_use]
    pub fn max(&self) -> f64 {
        self.max
    }

    #[inline]
    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|u| u.as_ref())
    }

    #[inline]
    #[must_use]
    pub fn receivers(&self) -> &Receivers {
        &self.receivers
    }

    /// Check if this signal is a multiplexer switch (marked with 'M')
    #[inline]
    #[must_use]
    pub fn is_multiplexer_switch(&self) -> bool {
        self.is_multiplexer_switch
    }

    /// Get the multiplexer switch value if this is a multiplexed signal (marked with 'm0', 'm1', etc.)
    /// Returns None if this is a normal signal (not multiplexed)
    #[inline]
    #[must_use]
    pub fn multiplexer_switch_value(&self) -> Option<u64> {
        self.multiplexer_switch_value
    }
}

#[inline]
fn canonical_f64_bits(v: f64) -> u64 {
    // Ensure Hash/Eq are consistent and satisfy the contracts:

    // - Treat -0.0 and 0.0 as equal (and hash identically)

    // - Treat NaN values as equal (and hash identically)

    if v == 0.0 {
        0.0f64.to_bits()
    } else if v.is_nan() {
        f64::NAN.to_bits()
    } else {
        v.to_bits()
    }
}

impl PartialEq for Signal {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.start_bit == other.start_bit
            && self.length == other.length
            && self.byte_order == other.byte_order
            && self.unsigned == other.unsigned
            && canonical_f64_bits(self.factor) == canonical_f64_bits(other.factor)
            && canonical_f64_bits(self.offset) == canonical_f64_bits(other.offset)
            && canonical_f64_bits(self.min) == canonical_f64_bits(other.min)
            && canonical_f64_bits(self.max) == canonical_f64_bits(other.max)
            && self.unit == other.unit
            && self.receivers == other.receivers
            && self.is_multiplexer_switch == other.is_multiplexer_switch
            && self.multiplexer_switch_value == other.multiplexer_switch_value
    }
}

// Custom Eq implementation that handles f64 (treats NaN as equal to NaN, and -0.0 == 0.0)
impl Eq for Signal {}

// Custom Hash implementation that handles f64 (treats NaN consistently)
impl core::hash::Hash for Signal {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.start_bit.hash(state);
        self.length.hash(state);
        self.byte_order.hash(state);
        self.unsigned.hash(state);
        // Handle f64: convert to bits for hashing (NaN will have consistent representation)
        canonical_f64_bits(self.factor).hash(state);
        canonical_f64_bits(self.offset).hash(state);
        canonical_f64_bits(self.min).hash(state);
        canonical_f64_bits(self.max).hash(state);
        self.unit.hash(state);
        self.receivers.hash(state);
        self.is_multiplexer_switch.hash(state);
        self.multiplexer_switch_value.hash(state);
    }
}
