//! High-performance decoding for Message.

use super::Message;

impl Message {
    /// Decode all signals into the output buffer (physical values).
    ///
    /// This is a zero-allocation decode path for high-speed CAN processing.
    /// Signals are decoded in order and written to the output buffer.
    ///
    /// # Arguments
    /// * `data` - Raw CAN payload bytes
    /// * `out` - Output buffer for physical values (must be at least `signals().len()` long)
    ///
    /// # Returns
    /// Number of signals decoded, or 0 if payload is too short.
    ///
    /// # Example
    /// ```rust,ignore
    /// let msg = dbc.messages().find_by_id(256).unwrap();
    /// let mut values = [0.0f64; 64];
    /// let count = msg.decode_into(&payload, &mut values);
    /// for i in 0..count {
    ///     let signal = msg.signals().at(i).unwrap();
    ///     println!("{}: {}", signal.name(), values[i]);
    /// }
    /// ```
    #[inline]
    pub fn decode_into(&self, data: &[u8], out: &mut [f64]) -> usize {
        // Check minimum payload length
        let min_bytes = self.min_bytes_required() as usize;
        if data.len() < min_bytes {
            return 0;
        }

        let signals = self.signals();
        let count = signals.len().min(out.len());

        for (i, out_val) in out.iter_mut().enumerate().take(count) {
            if let Some(signal) = signals.at(i) {
                // Use decode_raw which returns (raw_value, physical_value)
                if let Ok((_, physical)) = signal.decode_raw(data) {
                    *out_val = physical;
                } else {
                    *out_val = 0.0;
                }
            }
        }

        count
    }

    /// Decode all signals into the output buffer (raw integer values).
    ///
    /// Returns raw values before factor/offset conversion.
    /// Useful for encoding or debugging.
    ///
    /// # Arguments
    /// * `data` - Raw CAN payload bytes
    /// * `out` - Output buffer for raw values (must be at least `signals().len()` long)
    ///
    /// # Returns
    /// Number of signals decoded, or 0 if payload is too short.
    #[inline]
    pub fn decode_raw_into(&self, data: &[u8], out: &mut [i64]) -> usize {
        let min_bytes = self.min_bytes_required() as usize;
        if data.len() < min_bytes {
            return 0;
        }

        let signals = self.signals();
        let count = signals.len().min(out.len());

        for (i, out_val) in out.iter_mut().enumerate().take(count) {
            if let Some(signal) = signals.at(i) {
                if let Ok((raw, _)) = signal.decode_raw(data) {
                    *out_val = raw;
                } else {
                    *out_val = 0;
                }
            }
        }

        count
    }

    /// Decode a single signal by index.
    ///
    /// Returns the physical value or `None` if index is out of bounds or decode fails.
    #[inline]
    pub fn decode_signal(&self, index: usize, data: &[u8]) -> Option<f64> {
        let signal = self.signals().at(index)?;
        signal.decode_raw(data).ok().map(|(_, physical)| physical)
    }

    /// Decode a single signal by index (raw value).
    ///
    /// Returns the raw integer value or `None` if index is out of bounds or decode fails.
    #[inline]
    pub fn decode_signal_raw(&self, index: usize, data: &[u8]) -> Option<i64> {
        let signal = self.signals().at(index)?;
        signal.decode_raw(data).ok().map(|(raw, _)| raw)
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_decode_into_basic() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "C" *
"#,
        )
        .unwrap();

        let msg = dbc.messages().find_by_id(256).unwrap();

        // RPM = 2000 (raw 8000 = 0x1F40), Temp = 50°C (raw 90 = 0x5A)
        let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut values = [0.0f64; 8];

        let count = msg.decode_into(&payload, &mut values);

        assert_eq!(count, 2);
        assert_eq!(values[0], 2000.0); // RPM
        assert_eq!(values[1], 50.0); // Temp
    }

    #[test]
    fn test_decode_raw_into() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#,
        )
        .unwrap();

        let msg = dbc.messages().find_by_id(256).unwrap();

        let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut raw_values = [0i64; 8];

        let count = msg.decode_raw_into(&payload, &mut raw_values);

        assert_eq!(count, 1);
        assert_eq!(raw_values[0], 8000); // Raw before factor
    }

    #[test]
    fn test_decode_into_payload_too_short() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (1,0) [0|65535] "rpm" *
"#,
        )
        .unwrap();

        let msg = dbc.messages().find_by_id(256).unwrap();

        // Payload too short (need 2 bytes for 16-bit signal)
        let payload = [0x40];
        let mut values = [0.0f64; 8];

        let count = msg.decode_into(&payload, &mut values);

        assert_eq!(count, 0);
    }

    #[test]
    fn test_decode_signal_by_index() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "C" *
"#,
        )
        .unwrap();

        let msg = dbc.messages().find_by_id(256).unwrap();
        let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert_eq!(msg.decode_signal(0, &payload), Some(2000.0));
        assert_eq!(msg.decode_signal(1, &payload), Some(50.0));
        assert_eq!(msg.decode_signal(2, &payload), None); // Out of bounds
    }

    #[test]
    fn test_decode_into_buffer_smaller_than_signals() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ Sig1 : 0|8@1+ (1,0) [0|255] "" *
 SG_ Sig2 : 8|8@1+ (1,0) [0|255] "" *
 SG_ Sig3 : 16|8@1+ (1,0) [0|255] "" *
"#,
        )
        .unwrap();

        let msg = dbc.messages().find_by_id(256).unwrap();
        let payload = [0x01, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00];

        // Buffer only has space for 2 values, but message has 3 signals
        let mut values = [0.0f64; 2];
        let count = msg.decode_into(&payload, &mut values);

        assert_eq!(count, 2);
        assert_eq!(values[0], 1.0);
        assert_eq!(values[1], 2.0);
    }
}
