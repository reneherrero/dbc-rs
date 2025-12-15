use super::Signal;
use crate::{ByteOrder, Error, Result};

impl Signal {
    /// Decode the signal value from CAN message data bytes.
    ///
    /// Extracts the raw value from bits based on the signal's start bit, length, and byte order,
    /// then applies factor and offset to return the physical/engineering value.
    ///
    /// # Arguments
    ///
    /// * `data` - The CAN message data bytes (up to 8 bytes for classic CAN, 64 for CAN FD)
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` - The physical value (raw * factor + offset)
    /// * `Err(Error)` - If the signal extends beyond the data length
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// // Decode a CAN message with RPM value of 2000 (raw: 8000)
    /// let data = [0x20, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let rpm = signal.decode(&data)?;
    /// assert_eq!(rpm, 2000.0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// Decode signal value from CAN payload - optimized for high-throughput decoding.
    #[inline]
    pub fn decode(&self, data: &[u8]) -> Result<f64> {
        // Cache conversions to usize (common in hot path)
        let start_bit = self.start_bit as usize;
        let length = self.length as usize;
        let end_byte = (start_bit + length - 1) / 8;

        // Bounds check - early return for invalid signals
        if end_byte >= data.len() {
            return Err(Error::Decoding(Error::SIGNAL_EXTENDS_BEYOND_DATA));
        }

        // Extract bits based on byte order
        let raw_value = match self.byte_order {
            ByteOrder::LittleEndian => {
                Self::extract_bits_little_endian_helper(data, start_bit, length)
            }
            ByteOrder::BigEndian => Self::extract_bits_big_endian_helper(data, start_bit, length),
        };

        // Convert to signed/unsigned with optimized sign extension
        let value = if self.unsigned {
            raw_value as i64
        } else {
            // Sign extend for signed values
            // Optimized: compute sign bit mask only once
            let sign_bit_mask = 1u64 << (length - 1);
            if (raw_value & sign_bit_mask) != 0 {
                // Negative value - sign extend using bitwise mask
                let mask = !((1u64 << length) - 1);
                (raw_value | mask) as i64
            } else {
                raw_value as i64
            }
        };

        // Apply factor and offset to get physical value (single mul-add operation)
        Ok((value as f64) * self.factor + self.offset)
    }

    /// Extract bits from data using little-endian byte order.
    /// Inlined for hot path optimization.
    #[inline]
    pub(crate) fn extract_bits_little_endian_helper(
        data: &[u8],
        start_bit: usize,
        length: usize,
    ) -> u64 {
        let mut value: u64 = 0;
        let mut bits_remaining = length;
        let mut current_bit = start_bit;

        while bits_remaining > 0 {
            let byte_idx = current_bit / 8;
            let bit_in_byte = current_bit % 8;
            let bits_to_take = bits_remaining.min(8 - bit_in_byte);

            let byte = data[byte_idx] as u64;
            let mask = ((1u64 << bits_to_take) - 1) << bit_in_byte;
            let extracted = (byte & mask) >> bit_in_byte;

            value |= extracted << (length - bits_remaining);

            bits_remaining -= bits_to_take;
            current_bit += bits_to_take;
        }

        value
    }

    /// Extract bits from data using big-endian byte order.
    /// Inlined for hot path optimization.
    #[inline]
    pub(crate) fn extract_bits_big_endian_helper(
        data: &[u8],
        start_bit: usize,
        length: usize,
    ) -> u64 {
        let mut value: u64 = 0;
        let mut bits_remaining = length;
        let mut current_bit = start_bit;

        while bits_remaining > 0 {
            let byte_idx = current_bit / 8;
            let bit_in_byte = current_bit % 8;
            let bits_to_take = bits_remaining.min(8 - bit_in_byte);

            let byte = data[byte_idx] as u64;
            let mask = ((1u64 << bits_to_take) - 1) << bit_in_byte;
            let extracted = (byte & mask) >> bit_in_byte;

            // For big-endian, we need to reverse the bit order within bytes
            // and place bits in the correct position
            value |= extracted << (length - bits_remaining);

            bits_remaining -= bits_to_take;
            current_bit += bits_to_take;
        }

        value
    }
}
