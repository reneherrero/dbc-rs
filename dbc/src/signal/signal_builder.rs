use crate::{ByteOrder, Error, ReceiversBuilder, Result, error::lang, signal::Signal};

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

#[derive(Debug, Clone)]
pub struct SignalBuilder {
    name: Option<String>,
    start_bit: Option<u16>,
    length: Option<u16>,
    byte_order: Option<ByteOrder>,
    unsigned: Option<bool>,
    factor: Option<f64>,
    offset: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    unit: Option<String>,
    receivers: ReceiversBuilder,
}

impl SignalBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            start_bit: None,
            length: None,
            byte_order: None,
            unsigned: None,
            factor: None,
            offset: None,
            min: None,
            max: None,
            unit: None,
            receivers: ReceiversBuilder::new(),
        }
    }

    #[must_use]
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn start_bit(mut self, start_bit: u16) -> Self {
        self.start_bit = Some(start_bit);
        self
    }

    #[must_use]
    pub fn length(mut self, length: u16) -> Self {
        self.length = Some(length);
        self
    }

    #[must_use]
    pub fn byte_order(mut self, byte_order: ByteOrder) -> Self {
        self.byte_order = Some(byte_order);
        self
    }

    #[must_use]
    pub fn unsigned(mut self, unsigned: bool) -> Self {
        self.unsigned = Some(unsigned);
        self
    }

    #[must_use]
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = Some(factor);
        self
    }

    #[must_use]
    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = Some(offset);
        self
    }

    #[must_use]
    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    #[must_use]
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    #[must_use]
    pub fn unit(mut self, unit: impl AsRef<str>) -> Self {
        self.unit = Some(unit.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn receivers(mut self, receivers: ReceiversBuilder) -> Self {
        self.receivers = receivers;
        self
    }

    fn extract_fields(&self) -> Result<SignalFields> {
        let name = self.name.clone().ok_or(Error::Signal(lang::SIGNAL_NAME_EMPTY))?;
        let start_bit = self.start_bit.ok_or(Error::Signal(lang::SIGNAL_START_BIT_REQUIRED))?;
        let length = self.length.ok_or(Error::Signal(lang::SIGNAL_LENGTH_REQUIRED))?;
        let byte_order = self.byte_order.ok_or(Error::Signal("byte_order is required"))?;
        let unsigned = self.unsigned.ok_or(Error::Signal("unsigned is required"))?;
        let factor = self.factor.ok_or(Error::Signal("factor is required"))?;
        let offset = self.offset.ok_or(Error::Signal("offset is required"))?;
        let min = self.min.ok_or(Error::Signal("min is required"))?;
        let max = self.max.ok_or(Error::Signal("max is required"))?;
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
            return Err(Error::Signal(lang::SIGNAL_PARSE_INVALID_START_BIT));
        }

        // Validate that start_bit + length doesn't exceed CAN FD maximum (512 bits)
        // Note: This is a basic sanity check. Full validation (including name, min/max,
        // message DLC bounds, and overlap detection) happens in build() when the signal
        // is actually constructed, to avoid duplicate validation calls.
        let end_bit = start_bit + length - 1; // -1 because length includes the start bit
        if end_bit >= 512 {
            return Err(Error::Signal(lang::SIGNAL_EXTENDS_BEYOND_MESSAGE));
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
}

impl SignalBuilder {
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

impl Default for SignalBuilder {
    fn default() -> Self {
        Self::new()
    }
}
