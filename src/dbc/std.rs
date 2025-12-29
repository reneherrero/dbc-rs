use super::Dbc;
use crate::Result;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;

impl Dbc {
    /// Load and parse a DBC file from disk.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::from_file("path/to/file.dbc")?;
    /// println!("Loaded {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&content)
    }
    /// Serialize this DBC to a DBC format string
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let dbc_string = dbc.to_dbc_string();
    /// // The string can be written to a file or used elsewhere
    /// assert!(dbc_string.contains("VERSION"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn to_dbc_string(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let signal_count: usize = self.messages().iter().map(|m| m.signals().len()).sum();
        let estimated_capacity = 200 + (self.messages.len() * 50) + (signal_count * 100);
        let mut result = String::with_capacity(estimated_capacity);

        // VERSION line
        if let Some(version) = &self.version {
            result.push_str(&version.to_dbc_string());
            result.push_str("\n\n");
        }

        // BS_ line (bit timing) - only output if present
        if let Some(ref bit_timing) = self.bit_timing {
            result.push_str(&bit_timing.to_string());
            result.push_str("\n\n");
        } else {
            // Empty BS_: is always required per DBC spec
            result.push_str("BS_:\n\n");
        }

        // BU_ line
        result.push_str(&self.nodes.to_dbc_string());
        result.push('\n');

        // BO_ and SG_ lines for each message
        for message in self.messages().iter() {
            result.push('\n');
            result.push_str(&message.to_string_full());
        }

        // CM_ lines (comments section)
        // General database comment
        if let Some(comment) = self.comment() {
            result.push_str("\nCM_ \"");
            result.push_str(comment);
            result.push_str("\";\n");
        }

        // Node comments
        for node in self.nodes.iter_nodes() {
            if let Some(comment) = node.comment() {
                result.push_str("CM_ BU_ ");
                result.push_str(node.name());
                result.push_str(" \"");
                result.push_str(comment);
                result.push_str("\";\n");
            }
        }

        // Message and signal comments
        for message in self.messages().iter() {
            if let Some(comment) = message.comment() {
                result.push_str("CM_ BO_ ");
                result.push_str(&message.id().to_string());
                result.push_str(" \"");
                result.push_str(comment);
                result.push_str("\";\n");
            }

            for signal in message.signals().iter() {
                if let Some(comment) = signal.comment() {
                    result.push_str("CM_ SG_ ");
                    result.push_str(&message.id().to_string());
                    result.push(' ');
                    result.push_str(signal.name());
                    result.push_str(" \"");
                    result.push_str(comment);
                    result.push_str("\";\n");
                }
            }
        }

        result
    }
}

impl Display for Dbc {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_to_dbc_string() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#,
        )
        .unwrap();

        let dbc_string = dbc.to_dbc_string();
        assert!(dbc_string.contains("VERSION"));
        assert!(dbc_string.contains("BU_"));
        assert!(dbc_string.contains("BO_"));
        assert!(dbc_string.contains("SG_"));
    }

    #[test]
    fn test_display() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        let display_str = format!("{}", dbc);
        assert!(display_str.contains("VERSION"));
    }

    #[test]
    fn test_save_round_trip() {
        let original = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@0+ (0.1,0) [0|1000] "bar"
"#;

        let dbc = Dbc::parse(original).unwrap();
        let saved = dbc.to_dbc_string();
        let dbc2 = Dbc::parse(&saved).unwrap();

        // Verify round-trip: parsed data should match
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            dbc2.version().map(|v| v.to_string())
        );
        assert_eq!(dbc.messages().len(), dbc2.messages().len());

        for (msg1, msg2) in dbc.messages().iter().zip(dbc2.messages().iter()) {
            assert_eq!(msg1.id(), msg2.id());
            assert_eq!(msg1.name(), msg2.name());
            assert_eq!(msg1.dlc(), msg2.dlc());
            assert_eq!(msg1.sender(), msg2.sender());
            assert_eq!(msg1.signals().len(), msg2.signals().len());

            for (sig1, sig2) in msg1.signals().iter().zip(msg2.signals().iter()) {
                assert_eq!(sig1.name(), sig2.name());
                assert_eq!(sig1.start_bit(), sig2.start_bit());
                assert_eq!(sig1.length(), sig2.length());
                assert_eq!(sig1.byte_order(), sig2.byte_order());
                assert_eq!(sig1.is_unsigned(), sig2.is_unsigned());
                assert_eq!(sig1.factor(), sig2.factor());
                assert_eq!(sig1.offset(), sig2.offset());
                assert_eq!(sig1.min(), sig2.min());
                assert_eq!(sig1.max(), sig2.max());
                assert_eq!(sig1.unit(), sig2.unit());
                assert_eq!(sig1.receivers(), sig2.receivers());
            }
        }
    }

    #[test]
    fn test_save_basic() {
        // Use parsing instead of builders
        // Note: '*' is parsed as None per spec compliance, output is 'Vector__XXX'
        let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();

        let saved = dbc.to_dbc_string();
        assert!(saved.contains("VERSION \"1.0\""));
        assert!(saved.contains("BU_: ECM"));
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        // Per DBC spec Section 9.5: '*' is not valid, we output 'Vector__XXX' instead
        assert!(saved.contains("SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" Vector__XXX"));
    }

    #[test]
    fn test_save_multiple_messages() {
        // Use parsing instead of builders
        let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        let saved = dbc.to_dbc_string();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    #[test]
    fn test_bit_timing_empty() {
        // Empty BS_: should still be present in output
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        // bit_timing should be None when empty
        assert!(dbc.bit_timing().is_none());

        // Output should still have BS_:
        let saved = dbc.to_dbc_string();
        assert!(saved.contains("BS_:"));
    }

    #[test]
    fn test_bit_timing_with_values() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BS_: 500000 : 1,2

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        // bit_timing should be Some with values
        let bt = dbc.bit_timing().expect("bit timing should be present");
        assert_eq!(bt.baudrate(), Some(500000));
        assert_eq!(bt.btr1(), Some(1));
        assert_eq!(bt.btr2(), Some(2));

        // Output should have full BS_ line
        let saved = dbc.to_dbc_string();
        assert!(saved.contains("BS_: 500000 : 1,2"));
    }

    #[test]
    fn test_bit_timing_baudrate_only() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BS_: 500000

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();

        // bit_timing should be Some with baudrate only
        let bt = dbc.bit_timing().expect("bit timing should be present");
        assert_eq!(bt.baudrate(), Some(500000));
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);

        // Output should have BS_ with baudrate
        let saved = dbc.to_dbc_string();
        assert!(saved.contains("BS_: 500000"));
    }
}
