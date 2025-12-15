use crate::{ByteOrder, Error, MAX_SIGNALS_PER_MESSAGE, Result, Signal};

use super::Message;

impl Message {
    #[allow(clippy::similar_names)] // physical_lsb and physical_msb are intentionally similar
    pub(crate) fn bit_range(start_bit: u16, length: u16, byte_order: ByteOrder) -> (u16, u16) {
        let start = start_bit;
        let len = length;

        match byte_order {
            ByteOrder::LittleEndian => {
                // Little-endian: start_bit is LSB, signal extends forward
                // Range: [start_bit, start_bit + length - 1]
                (start, start + len - 1)
            }
            ByteOrder::BigEndian => {
                // Big-endian: start_bit is MSB in big-endian numbering, signal extends backward
                // The big-endian bit numbering follows Vector convention:
                // be_bits = [7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 23, 22, ...]
                // This means: BE bit 0 -> physical bit 7, BE bit 7 -> physical bit 0
                //            BE bit 8 -> physical bit 15, BE bit 15 -> physical bit 8
                // To find the physical bit range, we need to map all BE bits from start_bit
                // to (start_bit + length - 1) to physical bits and find the min/max.
                // For BE bit N: byte_num = N / 8, bit_in_byte = N % 8
                // Physical bit = byte_num * 8 + (7 - bit_in_byte)

                // Calculate physical bits for all BE bits in the signal range
                let mut min_physical = u16::MAX;
                let mut max_physical = 0u16;

                for be_bit in start..start + len {
                    let byte_num = be_bit / 8;
                    let bit_in_byte = be_bit % 8;
                    let physical_bit = byte_num * 8 + (7 - bit_in_byte);

                    if physical_bit < min_physical {
                        min_physical = physical_bit;
                    }
                    if physical_bit > max_physical {
                        max_physical = physical_bit;
                    }
                }

                (min_physical, max_physical)
            }
        }
    }

    #[allow(clippy::similar_names)] // Overlap detection uses intentionally similar variable names (sig1_lsb/sig1_msb, sig2_lsb/sig2_msb)
    pub(crate) fn validate_internal(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Signal],
    ) -> Result<()> {
        // Validate CAN ID range
        // CAN specification allows:
        // - Standard 11-bit IDs: 0x000 to 0x7FF (0-2047)
        // - Extended 29-bit IDs: 0x00000000 to 0x1FFFFFFF (0-536870911)
        // Note: Extended IDs can technically be 0-536870911, but DBC files typically
        // use the convention where IDs 0-2047 are treated as standard and 2048+ as extended.
        // We only validate the maximum range here; the distinction between standard/extended
        // is determined by the ID value in practice.
        const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF; // 536870911

        // Check signal count limit per message (DoS protection)
        if let Some(err) = crate::check_max_limit(
            signals.len(),
            MAX_SIGNALS_PER_MESSAGE,
            Error::Validation(Error::MESSAGE_TOO_MANY_SIGNALS),
        ) {
            return Err(err);
        }

        if name.trim().is_empty() {
            return Err(Error::Validation(Error::MESSAGE_NAME_EMPTY));
        }

        if sender.trim().is_empty() {
            return Err(Error::Validation(Error::MESSAGE_SENDER_EMPTY));
        }

        // Validate DLC (Data Length Code): must be between 1 and 64 bytes
        // - Classic CAN Standard (CAN 2.0A): DLC <= 8 bytes (64 bits) maximum payload
        // - Classic CAN Extended (CAN 2.0B): DLC <= 8 bytes (64 bits) maximum payload
        // - CAN FD (Flexible Data Rate, ISO/Bosch): DLC <= 64 bytes (512 bits) maximum payload
        if dlc == 0 {
            return Err(Error::Validation(Error::MESSAGE_DLC_TOO_SMALL));
        }
        if dlc > 64 {
            return Err(Error::Validation(Error::MESSAGE_DLC_TOO_LARGE));
        }

        // Validate that ID is within valid CAN ID range
        // Extended CAN IDs can be 0x00000000 to 0x1FFFFFFF (0 to 536870911)
        // IDs exceeding 0x1FFFFFFF are invalid
        if id > MAX_EXTENDED_ID {
            return Err(Error::Validation(Error::MESSAGE_ID_OUT_OF_RANGE));
        }

        // Validate that all signals fit within the message boundary
        // Each signal must fit within: DLC * 8 bits
        // - Classic CAN (2.0A/2.0B): DLC * 8 <= 64 bits (8 bytes)
        // - CAN FD: DLC * 8 <= 512 bits (64 bytes)
        // This ensures no signal extends beyond the message payload capacity
        let max_bits = u16::from(dlc) * 8;
        for signal in signals.iter() {
            // Calculate the actual bit range for this signal (accounting for byte order)
            let (lsb, msb) =
                Self::bit_range(signal.start_bit(), signal.length(), signal.byte_order());
            // Check if the signal extends beyond the message boundary
            // The signal's highest bit position must be less than max_bits
            let signal_max_bit = lsb.max(msb);
            if signal_max_bit >= max_bits {
                return Err(Error::Validation(Error::SIGNAL_EXTENDS_BEYOND_MESSAGE));
            }
        }

        // Validate signal overlap detection
        // Check if any two signals overlap in the same message
        // Must account for byte order: little-endian signals extend forward,
        // big-endian signals extend backward from start_bit
        // We iterate over pairs without collecting to avoid alloc
        for (i, sig1) in signals.iter().enumerate() {
            let (sig1_lsb, sig1_msb) =
                Self::bit_range(sig1.start_bit(), sig1.length(), sig1.byte_order());

            for sig2 in signals.iter().skip(i + 1) {
                let (sig2_lsb, sig2_msb) =
                    Self::bit_range(sig2.start_bit(), sig2.length(), sig2.byte_order());

                // Check if ranges overlap
                // Two ranges [lsb1, msb1] and [lsb2, msb2] overlap if:
                // lsb1 <= msb2 && lsb2 <= msb1
                if sig1_lsb <= sig2_msb && sig2_lsb <= sig1_msb {
                    // Allow overlap only if both signals are multiplexed (have multiplexer_switch_value)
                    // Multiplexed signals share bit positions but only one is active at a time
                    let sig1_is_multiplexed = sig1.multiplexer_switch_value().is_some();
                    let sig2_is_multiplexed = sig2.multiplexer_switch_value().is_some();

                    if !(sig1_is_multiplexed && sig2_is_multiplexed) {
                        // At least one signal is not multiplexed, so overlap is not allowed
                        return Err(Error::Validation(Error::SIGNAL_OVERLAP));
                    }
                    // Both are multiplexed, overlap is allowed - continue
                }
            }
        }

        Ok(())
    }
}
