use crate::{Dbc, Error, Message, Result, compat::Vec};

/// Maximum CAN FD payload size (64 bytes)
const MAX_PAYLOAD_SIZE: usize = 64;

impl Dbc {
    /// Encode signal values into a CAN message payload.
    ///
    /// This is the inverse of [`Dbc::decode()`]. It takes a list of signal names
    /// and their physical values, and produces a raw CAN message payload ready
    /// for transmission.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw CAN message ID (without extended flag)
    /// * `signals` - Slice of (signal_name, physical_value) tuples to encode
    /// * `is_extended` - Whether this is an extended (29-bit) CAN ID
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8, 64>)` - The encoded payload, sized according to message DLC
    /// * `Err(Error)` - If message not found, signal not found, or value out of range
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
    ///  SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" *
    /// "#)?;
    ///
    /// // Encode RPM=2000 and Temp=50°C
    /// let payload = dbc.encode(256, &[("RPM", 2000.0), ("Temp", 50.0)], false)?;
    ///
    /// // The payload can now be transmitted over CAN
    /// assert_eq!(payload.len(), 8);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Multiplexing
    ///
    /// For multiplexed messages, include the multiplexer switch signal in the
    /// signal list along with the multiplexed signals you want to encode:
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 300 Sensors : 8 ECM
    ///  SG_ SensorID M : 0|8@1+ (1,0) [0|3] ""
    ///  SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C" *
    ///  SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa" *
    /// "#)?;
    ///
    /// // Encode temperature reading (SensorID=0)
    /// let payload = dbc.encode(300, &[("SensorID", 0.0), ("Temperature", 25.0)], false)?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    pub fn encode(
        &self,
        id: u32,
        signals: &[(&str, f64)],
        is_extended: bool,
    ) -> Result<Vec<u8, MAX_PAYLOAD_SIZE>> {
        // If it's an extended ID, add the extended ID flag
        let id = if is_extended {
            id | Message::EXTENDED_ID_FLAG
        } else {
            id
        };

        // Find message by ID
        let message = self
            .messages()
            .find_by_id(id)
            .ok_or(Error::Encoding(Error::MESSAGE_NOT_FOUND))?;

        // Create zero-initialized payload of size DLC
        let dlc = message.dlc() as usize;
        let mut payload: Vec<u8, MAX_PAYLOAD_SIZE> = Vec::new();
        for _ in 0..dlc {
            payload.push(0).map_err(|_| Error::Encoding(Error::MESSAGE_DLC_TOO_LARGE))?;
        }

        // Encode each signal
        for &(signal_name, physical_value) in signals {
            // Find signal in message
            let signal = message
                .signals()
                .iter()
                .find(|s| s.name() == signal_name)
                .ok_or(Error::Encoding(Error::ENCODING_SIGNAL_NOT_FOUND))?;

            // Encode and insert into payload (get mutable slice for heapless compatibility)
            signal.encode_to(physical_value, payload.as_mut_slice())?;
        }

        Ok(payload)
    }

    /// Encode signal values into a CAN frame using embedded-can types.
    ///
    /// This is a convenience method that encodes signals and returns the result
    /// in a format compatible with embedded-can drivers.
    ///
    /// # Arguments
    ///
    /// * `id` - The CAN ID (Standard or Extended)
    /// * `signals` - Slice of (signal_name, physical_value) tuples to encode
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8, 64>)` - The encoded payload
    /// * `Err(Error)` - If encoding fails
    ///
    /// # Feature
    ///
    /// This method is only available when the `embedded-can` feature is enabled.
    #[cfg(feature = "embedded-can")]
    #[inline]
    pub fn encode_for_id(
        &self,
        id: embedded_can::Id,
        signals: &[(&str, f64)],
    ) -> Result<Vec<u8, MAX_PAYLOAD_SIZE>> {
        match id {
            embedded_can::Id::Standard(std_id) => {
                self.encode(std_id.as_raw() as u32, signals, false)
            }
            embedded_can::Id::Extended(ext_id) => self.encode(ext_id.as_raw(), signals, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_encode_basic() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        // Encode RPM = 2000.0 (raw: 2000 / 0.25 = 8000 = 0x1F40)
        let payload = dbc.encode(256, &[("RPM", 2000.0)], false).unwrap();
        assert_eq!(payload.len(), 8);
        // Little-endian 16-bit: 8000 = 0x1F40 -> [0x40, 0x1F]
        assert_eq!(payload[0], 0x40);
        assert_eq!(payload[1], 0x1F);
    }

    #[test]
    fn test_encode_multiple_signals() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" *
"#,
        )
        .unwrap();

        // Encode RPM = 2000.0 and Temp = 50°C
        // RPM: 2000 / 0.25 = 8000 = 0x1F40
        // Temp: (50 - (-40)) / 1 = 90 = 0x5A
        let payload = dbc.encode(256, &[("RPM", 2000.0), ("Temp", 50.0)], false).unwrap();
        assert_eq!(payload.len(), 8);
        assert_eq!(payload[0], 0x40); // RPM low byte
        assert_eq!(payload[1], 0x1F); // RPM high byte
        assert_eq!(payload[2], 0x5A); // Temp
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" *
 SG_ Throttle : 24|8@1+ (1,0) [0|100] "%" *
"#,
        )
        .unwrap();

        // Original values
        let rpm = 3500.0;
        let temp = 85.0;
        let throttle = 42.0;

        // Encode
        let payload = dbc
            .encode(
                256,
                &[("RPM", rpm), ("Temp", temp), ("Throttle", throttle)],
                false,
            )
            .unwrap();

        // Decode
        let decoded = dbc.decode(256, &payload, false).unwrap();

        // Verify roundtrip
        let find_value = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);
        assert!((find_value("RPM").unwrap() - rpm).abs() < 0.5); // Within factor precision
        assert!((find_value("Temp").unwrap() - temp).abs() < 0.5);
        assert!((find_value("Throttle").unwrap() - throttle).abs() < 0.5);
    }

    #[test]
    fn test_encode_message_not_found() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let result = dbc.encode(512, &[("RPM", 2000.0)], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_signal_not_found() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let result = dbc.encode(256, &[("NonExistent", 100.0)], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_value_out_of_range() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        // Value above max (8000)
        let result = dbc.encode(256, &[("RPM", 9000.0)], false);
        assert!(result.is_err());

        // Value below min (0)
        let result = dbc.encode(256, &[("RPM", -100.0)], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_signed_signal() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ Torque : 0|16@1- (0.01,0) [-327.68|327.67] "Nm" *
"#,
        )
        .unwrap();

        // Encode negative value: -10.0 -> raw = -1000 = 0xFC18 (two's complement)
        let payload = dbc.encode(256, &[("Torque", -10.0)], false).unwrap();
        assert_eq!(payload[0], 0x18); // Low byte
        assert_eq!(payload[1], 0xFC); // High byte (two's complement)

        // Roundtrip verify
        let decoded = dbc.decode(256, &payload, false).unwrap();
        assert!((decoded[0].value - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_encode_big_endian() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ Pressure : 7|16@0+ (0.01,0) [0|655.35] "kPa" *
"#,
        )
        .unwrap();

        // Encode 10.0 kPa -> raw = 1000 = 0x03E8, big-endian at bit 7
        let payload = dbc.encode(256, &[("Pressure", 10.0)], false).unwrap();
        assert_eq!(payload[0], 0x03); // High byte first (big-endian)
        assert_eq!(payload[1], 0xE8); // Low byte

        // Roundtrip verify
        let decoded = dbc.decode(256, &payload, false).unwrap();
        assert!((decoded[0].value - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_encode_extended_can_id() {
        // 0x80000000 + 0x400 = 2147484672
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
        )
        .unwrap();

        // Encode 100.0 km/h with extended ID 0x400
        let payload = dbc.encode(0x400, &[("Speed", 100.0)], true).unwrap();
        assert_eq!(payload[0], 0xE8); // 1000 = 0x03E8
        assert_eq!(payload[1], 0x03);

        // Roundtrip verify
        let decoded = dbc.decode(0x400, &payload, true).unwrap();
        assert!((decoded[0].value - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_encode_multiplexed_signal() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 300 Sensors : 8 ECM
 SG_ SensorID M : 0|8@1+ (1,0) [0|3] ""
 SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C" *
 SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa" *
"#,
        )
        .unwrap();

        // Encode temperature sensor data (SensorID=0)
        let payload = dbc.encode(300, &[("SensorID", 0.0), ("Temperature", 25.0)], false).unwrap();

        // Verify SensorID = 0
        assert_eq!(payload[0], 0x00);

        // Temperature 25°C -> raw = (25 - (-40)) / 0.1 = 650
        // Little-endian: 650 = 0x028A -> [0x8A, 0x02]
        assert_eq!(payload[1], 0x8A);
        assert_eq!(payload[2], 0x02);

        // Decode and verify
        let decoded = dbc.decode(300, &payload, false).unwrap();
        let find_value = |name: &str| decoded.iter().find(|s| s.name == name).map(|s| s.value);
        assert_eq!(find_value("SensorID"), Some(0.0));
        assert!((find_value("Temperature").unwrap() - 25.0).abs() < 0.1);
        // Pressure should not be decoded (SensorID != 1)
        assert!(find_value("Pressure").is_none());
    }

    #[test]
    fn test_encode_preserves_unset_bits() {
        // Test that encoding one signal doesn't affect bits used by other signals
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ SignalA : 0|8@1+ (1,0) [0|255] ""
 SG_ SignalB : 8|8@1+ (1,0) [0|255] ""
"#,
        )
        .unwrap();

        // Encode only SignalA
        let payload = dbc.encode(256, &[("SignalA", 100.0)], false).unwrap();
        assert_eq!(payload[0], 100);
        assert_eq!(payload[1], 0); // SignalB should be 0

        // Encode only SignalB
        let payload = dbc.encode(256, &[("SignalB", 200.0)], false).unwrap();
        assert_eq!(payload[0], 0); // SignalA should be 0
        assert_eq!(payload[1], 200);

        // Encode both
        let payload = dbc.encode(256, &[("SignalA", 100.0), ("SignalB", 200.0)], false).unwrap();
        assert_eq!(payload[0], 100);
        assert_eq!(payload[1], 200);
    }

    #[cfg(feature = "embedded-can")]
    mod embedded_can_tests {
        use super::*;
        use embedded_can::{ExtendedId, Id, StandardId};

        #[test]
        fn test_encode_for_id_standard() {
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
            )
            .unwrap();

            let std_id = Id::Standard(StandardId::new(256).unwrap());
            let payload = dbc.encode_for_id(std_id, &[("RPM", 2000.0)]).unwrap();
            assert_eq!(payload[0], 0x40);
            assert_eq!(payload[1], 0x1F);
        }

        #[test]
        fn test_encode_for_id_extended() {
            // 0x80000000 + 0x400 = 2147484672
            let dbc = Dbc::parse(
                r#"VERSION "1.0"

BU_: ECM

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h" *
"#,
            )
            .unwrap();

            let ext_id = Id::Extended(ExtendedId::new(0x400).unwrap());
            let payload = dbc.encode_for_id(ext_id, &[("Speed", 100.0)]).unwrap();
            assert_eq!(payload[0], 0xE8);
            assert_eq!(payload[1], 0x03);
        }
    }
}
