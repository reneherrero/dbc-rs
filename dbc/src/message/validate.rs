use super::Message;
use crate::{ByteOrder, Error, MAX_SIGNALS_PER_MESSAGE, Result, Signal, error::check_max_limit};

impl Message {
    pub(crate) fn validate(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Signal],
    ) -> Result<()> {
        // Check signal count limit per message (DoS protection)
        if let Some(err) = check_max_limit(
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

        // Per DBC spec Section 8.3: DLC can be 0-8 for CAN 2.0, 0-64 for CAN FD
        // DLC = 0 is valid (e.g., for control messages without data payload)
        if dlc > 64 {
            return Err(Error::Validation(Error::MESSAGE_DLC_TOO_LARGE));
        }

        // Validate ID is in a valid range
        let id_valid = id <= Self::MAX_EXTENDED_ID
            || (Self::EXTENDED_ID_FLAG..=Self::MAX_EXTENDED_ID_WITH_FLAG).contains(&id)
            || id == Self::PSEUDO_MESSAGE_ID;
        if !id_valid {
            return Err(Error::Validation(Error::MESSAGE_ID_OUT_OF_RANGE));
        }

        // Validate that all signals fit within the message boundary
        // Each signal must fit within: DLC * 8 bits
        // - Classic CAN (2.0A/2.0B): DLC * 8 <= 64 bits (8 bytes)
        // - CAN FD: DLC * 8 <= 512 bits (64 bytes)
        // This ensures no signal extends beyond the message payload capacity
        //
        // EXCEPTION: Per spec Section 8.6, VECTOR__INDEPENDENT_SIG_MSG (ID 0xC0000000)
        // is a special pseudo-message for "orphan" signals. Skip boundary validation
        // for this pseudo-message since its signals aren't meant to be transmitted.
        if id != Self::PSEUDO_MESSAGE_ID {
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
        }

        // Validate signal overlap detection
        // Check if any two signals overlap in the same message
        // Must account for byte order: little-endian signals extend forward,
        // big-endian signals extend backward from start_bit
        // NOTE: Multiplexed signals (signals with multiplexer_switch_value) are allowed
        // to overlap because they're only active when the multiplexer has a specific value.
        // We skip overlap checking for multiplexed signals.
        // We iterate over pairs without collecting to avoid alloc
        //
        // EXCEPTION: Per spec Section 8.6, VECTOR__INDEPENDENT_SIG_MSG (ID 0xC0000000)
        // is a special pseudo-message for "orphan" signals. Skip overlap validation
        // for this pseudo-message since its signals aren't meant to be transmitted.
        if id != Self::PSEUDO_MESSAGE_ID {
            for (i, sig1) in signals.iter().enumerate() {
                // Skip overlap check if sig1 is multiplexed
                if sig1.multiplexer_switch_value().is_some() {
                    continue;
                }

                let (sig1_lsb, sig1_msb) =
                    Self::bit_range(sig1.start_bit(), sig1.length(), sig1.byte_order());

                for sig2 in signals.iter().skip(i + 1) {
                    // Skip overlap check if sig2 is multiplexed
                    if sig2.multiplexer_switch_value().is_some() {
                        continue;
                    }

                    let (sig2_lsb, sig2_msb) =
                        Self::bit_range(sig2.start_bit(), sig2.length(), sig2.byte_order());

                    // Check if ranges overlap
                    // Two ranges [lsb1, msb1] and [lsb2, msb2] overlap if:
                    // lsb1 <= msb2 && lsb2 <= msb1
                    if sig1_lsb <= sig2_msb && sig2_lsb <= sig1_msb {
                        return Err(Error::Validation(Error::SIGNAL_OVERLAP));
                    }
                }
            }
        }

        Ok(())
    }

    fn bit_range(start_bit: u16, length: u16, byte_order: ByteOrder) -> (u16, u16) {
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
                // To find the physical bit range:
                // 1. Find the index of start_bit in the be_bits sequence
                // 2. MSB (physical) = be_bits[idx]
                // 3. LSB (physical) = be_bits[idx + length - 1]
                // We can calculate this directly:
                // For BE bit N: byte_num = N / 8, bit_in_byte = N % 8
                // Physical bit = byte_num * 8 + (7 - bit_in_byte)
                let byte_num = start / 8;
                let bit_in_byte = start % 8;
                let physical_msb = byte_num * 8 + (7 - bit_in_byte);

                // Calculate LSB: move forward (length - 1) positions in the BE sequence
                // BE bit (start + length - 1) maps to physical bit
                let lsb_be_bit = start + len - 1;
                let lsb_byte_num = lsb_be_bit / 8;
                let lsb_bit_in_byte = lsb_be_bit % 8;
                let physical_lsb = lsb_byte_num * 8 + (7 - lsb_bit_in_byte);

                // Ensure lsb <= msb (they should be in that order for big-endian)
                if physical_lsb <= physical_msb {
                    (physical_lsb, physical_msb)
                } else {
                    (physical_msb, physical_lsb)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_message_big_endian_bit_range_calculation() {
        // Test big-endian bit range calculation
        // BE bit 0 -> physical bit 7
        // BE bit 7 -> physical bit 0
        // BE bit 8 -> physical bit 15
        // BE bit 15 -> physical bit 8
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal starting at BE bit 0, length 8 -> should map to physical bits 0-7
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@1+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        let message = Message::parse(&mut parser, &[signal]).unwrap();
        // The signal should be valid and fit within the message
        assert_eq!(message.signals().len(), 1);
    }

    #[test]
    fn test_message_little_endian_bit_range_calculation() {
        // Test little-endian bit range calculation
        // LE bit N -> physical bit N (straightforward mapping)
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal starting at LE bit 0, length 8 -> should map to physical bits 0-7
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@0+ (1,0) [0|255] \"\"").unwrap())
                .unwrap();

        let message = Message::parse(&mut parser, &[signal]).unwrap();
        // The signal should be valid and fit within the message
        assert_eq!(message.signals().len(), 1);
    }

    #[test]
    fn test_message_multiple_signals_boundary_validation() {
        // Test that signals at message boundaries are validated correctly
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Create signals that exactly fill the message (8 bytes = 64 bits)
        // Signal 1: bits 0-15 (16 bits)
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|16@0+ (1,0) [0|65535] \"\"").unwrap())
                .unwrap();
        // Signal 2: bits 16-31 (16 bits)
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Signal2 : 16|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 3: bits 32-47 (16 bits)
        let signal3 = Signal::parse(
            &mut Parser::new(b"SG_ Signal3 : 32|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 4: bits 48-63 (16 bits) - exactly at boundary
        let signal4 = Signal::parse(
            &mut Parser::new(b"SG_ Signal4 : 48|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2, signal3, signal4]).unwrap();
        assert_eq!(message.signals().len(), 4);
    }
}
