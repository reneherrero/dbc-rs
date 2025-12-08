#[cfg(any(feature = "alloc", feature = "kernel"))]
use crate::compat::{Box, String, str_to_string};
use crate::{
    ByteOrder, Receivers, ReceiversBuilder,
    error::{Error, Result},
    signal::Signal,
};

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
    Receivers<'static>,
);

#[derive(Debug, Clone)]
pub struct SignalBuilder {
    name: Option<String>,
    start_bit: Option<u16>,
    length: Option<u16>,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<String>,
    receivers: Receivers<'static>,
}

impl Default for SignalBuilder {
    fn default() -> Self {
        Self {
            name: None,
            start_bit: None,
            length: None,
            byte_order: ByteOrder::BigEndian,
            unsigned: true,
            factor: 1.0,
            offset: 0.0,
            min: 0.0,
            max: 0.0,
            unit: None,
            receivers: ReceiversBuilder::new().broadcast().build().unwrap(),
        }
    }
}

impl SignalBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(str_to_string(name));
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
        self.byte_order = byte_order;
        self
    }

    #[must_use]
    pub fn unsigned(mut self, unsigned: bool) -> Self {
        self.unsigned = unsigned;
        self
    }

    #[must_use]
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    #[must_use]
    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    #[must_use]
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    #[must_use]
    pub fn unit(mut self, unit: impl AsRef<str>) -> Self {
        self.unit = Some(str_to_string(unit));
        self
    }

    #[must_use]
    pub fn no_unit(mut self) -> Self {
        self.unit = None;
        self
    }

    #[must_use]
    pub fn receivers(mut self, receivers: Receivers<'static>) -> Self {
        self.receivers = receivers;
        self
    }

    fn extract_fields(self) -> Result<SignalFields> {
        let name = self.name.ok_or(Error::signal(crate::error::lang::SIGNAL_NAME_EMPTY))?;
        let start_bit = self
            .start_bit
            .ok_or(Error::signal(crate::error::lang::SIGNAL_START_BIT_REQUIRED))?;
        let length =
            self.length.ok_or(Error::signal(crate::error::lang::SIGNAL_LENGTH_REQUIRED))?;
        Ok((
            name,
            start_bit,
            length,
            self.byte_order,
            self.unsigned,
            self.factor,
            self.offset,
            self.min,
            self.max,
            self.unit,
            self.receivers,
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
            return Err(Error::signal(
                crate::error::lang::SIGNAL_PARSE_INVALID_START_BIT,
            ));
        }

        // Validate that start_bit + length doesn't exceed CAN FD maximum (512 bits)
        // Note: This is a basic sanity check. Full validation (including message DLC bounds
        // and overlap detection) happens when the signal is added to a message.
        let end_bit = start_bit + length - 1; // -1 because length includes the start bit
        if end_bit >= 512 {
            return Err(Error::signal(
                crate::error::lang::SIGNAL_EXTENDS_BEYOND_MESSAGE,
            ));
        }

        Signal::validate(&name, length, min, max).map_err(Error::from)?;
        Ok(Self {
            name: Some(name),
            start_bit: Some(start_bit),
            length: Some(length),
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            receivers,
        })
    }

    pub fn build(self) -> Result<Signal<'static>> {
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
        // Convert owned strings to static references by leaking Box<str>
        let name_boxed: Box<str> = name.into_boxed_str();
        let name_static: &'static str = Box::leak(name_boxed);
        let unit_static: Option<&'static str> = if let Some(u) = unit {
            let boxed: Box<str> = u.into_boxed_str();
            Some(Box::leak(boxed))
        } else {
            None
        };
        // Validate before construction
        Signal::validate(name_static, length, min, max).map_err(|e| match e {
            crate::error::ParseError::Signal(msg) => Error::signal(msg),
            _ => Error::ParseError(e),
        })?;
        Ok(Signal::new(
            name_static,
            start_bit,
            length,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit_static,
            receivers,
        ))
    }
}
