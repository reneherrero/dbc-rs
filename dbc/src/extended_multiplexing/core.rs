use super::ExtendedMultiplexing;
use crate::{MAX_NAME_SIZE, compat::String, compat::Vec};

impl ExtendedMultiplexing {
    #[allow(dead_code)] // Used by builder/parser
    pub(crate) fn new(
        message_id: u32,
        signal_name: String<{ MAX_NAME_SIZE }>,
        multiplexer_switch: String<{ MAX_NAME_SIZE }>,
        value_ranges: Vec<(u64, u64), 64>,
    ) -> Self {
        Self {
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        }
    }

    #[must_use = "return value should be used"]
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    #[must_use = "return value should be used"]
    pub fn signal_name(&self) -> &str {
        self.signal_name.as_str()
    }

    #[must_use = "return value should be used"]
    pub fn multiplexer_switch(&self) -> &str {
        self.multiplexer_switch.as_str()
    }

    #[must_use = "return value should be used"]
    pub fn value_ranges(&self) -> &[(u64, u64)] {
        self.value_ranges.as_slice()
    }
}
