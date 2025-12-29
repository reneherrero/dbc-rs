use super::BitTiming;

impl BitTiming {
    /// Creates a new empty BitTiming (most common case).
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            baudrate: None,
            btr1: None,
            btr2: None,
        }
    }

    /// Creates a new BitTiming with baudrate only.
    #[must_use]
    pub const fn with_baudrate(baudrate: u32) -> Self {
        Self {
            baudrate: Some(baudrate),
            btr1: None,
            btr2: None,
        }
    }

    /// Creates a new BitTiming with all parameters.
    #[must_use]
    pub const fn with_btr(baudrate: u32, btr1: u32, btr2: u32) -> Self {
        Self {
            baudrate: Some(baudrate),
            btr1: Some(btr1),
            btr2: Some(btr2),
        }
    }

    /// Returns the baudrate in kbps, if specified.
    #[must_use]
    pub const fn baudrate(&self) -> Option<u32> {
        self.baudrate
    }

    /// Returns the BTR1 (Bus Timing Register 1) value, if specified.
    #[must_use]
    pub const fn btr1(&self) -> Option<u32> {
        self.btr1
    }

    /// Returns the BTR2 (Bus Timing Register 2) value, if specified.
    #[must_use]
    pub const fn btr2(&self) -> Option<u32> {
        self.btr2
    }

    /// Returns true if the bit timing section is empty (no values).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.baudrate.is_none()
    }
}
