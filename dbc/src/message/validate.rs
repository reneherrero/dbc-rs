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
        match byte_order {
            ByteOrder::LittleEndian => {
                // Little-endian: start_bit is LSB, signal extends forward (to higher bit positions)
                // Range: [start_bit, start_bit + length - 1]
                (start_bit, start_bit + length - 1)
            }
            ByteOrder::BigEndian => {
                // Big-endian (Motorola): start_bit is MSB position in Vector convention
                // Vector bit numbering: byte N contains bits [N*8+7, N*8+6, ..., N*8+0]
                // where bit N*8+7 is the MSB of byte N
                //
                // For a BE signal:
                // - First byte: uses bits from (start_bit % 8) down to 0
                // - Subsequent bytes: uses bits from 7 down to 0 (or partial for last byte)
                //
                // Example: start_bit=7, length=16
                //   Byte 0: bits 7,6,5,4,3,2,1,0 (8 bits) -> linear bits 7-0
                //   Byte 1: bits 7,6,5,4,3,2,1,0 (8 bits) -> linear bits 15-8
                //   Linear range: [0, 15]
                //
                // Example: start_bit=23, length=16
                //   Byte 2: bits 7,6,5,4,3,2,1,0 (8 bits) -> linear bits 23-16
                //   Byte 3: bits 7,6,5,4,3,2,1,0 (8 bits) -> linear bits 31-24
                //   Linear range: [16, 31]

                let start_byte = start_bit / 8;
                let msb_bit_in_byte = start_bit % 8; // 0-7, where 7 is MSB of byte
                let bits_in_first_byte = msb_bit_in_byte + 1;

                if length <= bits_in_first_byte {
                    // Signal fits within one byte
                    // Uses bits from msb_bit_in_byte down to (msb_bit_in_byte - length + 1)
                    let lsb = start_bit - (length - 1);
                    (lsb, start_bit)
                } else {
                    // Signal spans multiple bytes
                    let remaining = length - bits_in_first_byte;
                    let additional_full_bytes = remaining / 8;
                    let bits_in_last_byte = remaining % 8;

                    // Calculate last byte index
                    let last_byte = if bits_in_last_byte > 0 {
                        start_byte + 1 + additional_full_bytes
                    } else {
                        // All remaining bits fit in full bytes
                        start_byte + additional_full_bytes
                    };

                    // For multi-byte BE signals:
                    // - Min bit is bit 0 of the first byte (we go all the way down)
                    // - Max bit is bit 7 of the last byte (subsequent bytes start at bit 7)
                    let min_bit = start_byte * 8;
                    let max_bit = last_byte * 8 + 7;

                    (min_bit, max_bit)
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
        // For BE, start_bit is the MSB position
        // A typical BE signal starts at bit 7 (MSB of byte 0)
        let data = b"BO_ 256 EngineData : 8 ECM";
        let mut parser = Parser::new(data).unwrap();

        // Signal starting at BE bit 7 (MSB of byte 0), length 8 -> bytes 0
        let signal =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 7|8@0+ (1,0) [0|255] \"\"").unwrap())
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
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|8@1+ (1,0) [0|255] \"\"").unwrap())
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
        // Using little-endian (@1+) where start_bit is LSB position
        // Signal 1: bits 0-15 (16 bits)
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 0|16@1+ (1,0) [0|65535] \"\"").unwrap())
                .unwrap();
        // Signal 2: bits 16-31 (16 bits)
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Signal2 : 16|16@1+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 3: bits 32-47 (16 bits)
        let signal3 = Signal::parse(
            &mut Parser::new(b"SG_ Signal3 : 32|16@1+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();
        // Signal 4: bits 48-63 (16 bits) - exactly at boundary
        let signal4 = Signal::parse(
            &mut Parser::new(b"SG_ Signal4 : 48|16@1+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();

        let message = Message::parse(&mut parser, &[signal1, signal2, signal3, signal4]).unwrap();
        assert_eq!(message.signals().len(), 4);
    }

    #[test]
    fn test_big_endian_non_overlapping_signals() {
        // Test that non-overlapping big-endian signals are correctly detected as non-overlapping
        // This was a regression where BE signals in different bytes were incorrectly flagged as overlapping
        let data = b"BO_ 768 WheelSpeeds : 8 ECU";
        let mut parser = Parser::new(data).unwrap();

        // Signal 1: BE, start_bit=7 (byte 0 MSB), length=16 -> bytes 0-1
        let signal1 = Signal::parse(
            &mut Parser::new(b"SG_ WheelSpeed_FL : 7|16@0+ (0.01,0) [0|300] \"km/h\"").unwrap(),
        )
        .unwrap();

        // Signal 2: BE, start_bit=23 (byte 2 MSB), length=16 -> bytes 2-3
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ WheelSpeed_FR : 23|16@0+ (0.01,0) [0|300] \"km/h\"").unwrap(),
        )
        .unwrap();

        // These signals should NOT overlap (bytes 0-1 vs bytes 2-3)
        let message = Message::parse(&mut parser, &[signal1, signal2]).unwrap();
        assert_eq!(message.signals().len(), 2);
    }

    #[test]
    fn test_big_endian_overlapping_signals_detected() {
        // Test that overlapping big-endian signals ARE correctly detected
        let data = b"BO_ 256 Test : 8 ECU";
        let mut parser = Parser::new(data).unwrap();

        // Signal 1: BE, start_bit=7, length=16 -> bytes 0-1
        let signal1 =
            Signal::parse(&mut Parser::new(b"SG_ Signal1 : 7|16@0+ (1,0) [0|65535] \"\"").unwrap())
                .unwrap();

        // Signal 2: BE, start_bit=15, length=16 -> bytes 1-2 (overlaps with signal1 in byte 1)
        let signal2 = Signal::parse(
            &mut Parser::new(b"SG_ Signal2 : 15|16@0+ (1,0) [0|65535] \"\"").unwrap(),
        )
        .unwrap();

        // These signals SHOULD overlap (both use byte 1)
        let result = Message::parse(&mut parser, &[signal1, signal2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bit_range_big_endian_single_byte() {
        // Single byte BE signal: start_bit=5, length=4 -> bits 5,4,3,2
        let (min, max) = Message::bit_range(5, 4, ByteOrder::BigEndian);
        assert_eq!(min, 2);
        assert_eq!(max, 5);
    }

    #[test]
    fn test_bit_range_big_endian_multi_byte() {
        // Multi-byte BE signal: start_bit=7, length=16 -> bytes 0-1, bits 0-15
        let (min, max) = Message::bit_range(7, 16, ByteOrder::BigEndian);
        assert_eq!(min, 0);
        assert_eq!(max, 15);

        // Multi-byte BE signal: start_bit=23, length=16 -> bytes 2-3, bits 16-31
        let (min, max) = Message::bit_range(23, 16, ByteOrder::BigEndian);
        assert_eq!(min, 16);
        assert_eq!(max, 31);
    }

    #[test]
    fn test_bit_range_little_endian() {
        // LE signal: start_bit=0, length=16 -> bits 0-15
        let (min, max) = Message::bit_range(0, 16, ByteOrder::LittleEndian);
        assert_eq!(min, 0);
        assert_eq!(max, 15);

        // LE signal: start_bit=16, length=16 -> bits 16-31
        let (min, max) = Message::bit_range(16, 16, ByteOrder::LittleEndian);
        assert_eq!(min, 16);
        assert_eq!(max, 31);
    }
}
