use super::Signal;
use crate::Receivers;
use std::{
    fmt::{Display, Formatter, Result},
    string::String,
};

impl Signal {
    #[must_use = "return value should be used"]
    pub fn to_dbc_string(&self) -> String {
        let mut result = String::with_capacity(100); // Pre-allocate reasonable capacity

        result.push_str(" SG_ ");
        result.push_str(self.name());
        result.push_str(" : ");
        result.push_str(&self.start_bit().to_string());
        result.push('|');
        result.push_str(&self.length().to_string());
        result.push('@');

        // Byte order: 0 for BigEndian (Motorola), 1 for LittleEndian (Intel)
        // Per Vector DBC spec v1.0.1: "Big endian is stored as '0', little endian is stored as '1'"
        match self.byte_order() {
            crate::ByteOrder::BigEndian => result.push('0'), // @0 = Big Endian (Motorola)
            crate::ByteOrder::LittleEndian => result.push('1'), // @1 = Little Endian (Intel)
        }

        // Sign: + for unsigned, - for signed
        if self.is_unsigned() {
            result.push('+');
        } else {
            result.push('-');
        }

        // Factor and offset: (factor,offset)
        result.push_str(" (");
        use core::fmt::Write;
        write!(result, "{}", self.factor()).unwrap();
        result.push(',');
        write!(result, "{}", self.offset()).unwrap();
        result.push(')');

        // Min and max: [min|max]
        result.push_str(" [");
        write!(result, "{}", self.min()).unwrap();
        result.push('|');
        write!(result, "{}", self.max()).unwrap();
        result.push(']');

        // Unit: "unit" or ""
        result.push(' ');
        if let Some(unit) = self.unit() {
            result.push('"');
            result.push_str(unit);
            result.push('"');
        } else {
            result.push_str("\"\"");
        }

        // Per DBC spec Section 9.5: receivers are comma-separated
        // receivers = receiver {',' receiver} ;
        // receiver = node_name | 'Vector__XXX' ;
        match self.receivers() {
            Receivers::Nodes(nodes) => {
                result.push(' ');
                if nodes.is_empty() {
                    // Empty nodes list - use Vector__XXX per spec
                    result.push_str(crate::VECTOR__XXX);
                } else {
                    for (i, node) in self.receivers().iter().enumerate() {
                        if i > 0 {
                            result.push(',');
                        }
                        result.push_str(node);
                    }
                }
            }
            Receivers::None => {
                // No receivers specified - use Vector__XXX per spec
                result.push(' ');
                result.push_str(crate::VECTOR__XXX);
            }
        }

        result
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_dbc_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_signal_to_dbc_string_round_trip() {
        // Test round-trip: parse -> to_dbc_string -> parse
        // Note: DBC spec Section 9.5 says receivers are comma-separated
        // Per spec, '*' is not valid - we use Vector__XXX for no specific receiver
        let test_cases = vec![
            (
                // '*' is parsed as None, serialized as Vector__XXX per spec
                r#"SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *"#,
                " SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" Vector__XXX",
            ),
            (
                // Parser accepts space-separated (tool extension), serializer outputs comma-separated (per spec)
                r#"SG_ Temperature : 16|8@1- (1,-40) [-40|215] "°C" TCM BCM"#,
                " SG_ Temperature : 16|8@1- (1,-40) [-40|215] \"°C\" TCM,BCM",
            ),
            (
                // '*' is parsed as None, serialized as Vector__XXX per spec
                r#"SG_ Flag : 24|1@0+ (1,0) [0|1] "" *"#,
                " SG_ Flag : 24|1@0+ (1,0) [0|1] \"\" Vector__XXX",
            ),
        ];

        for (input_line, expected_output) in test_cases {
            // Parse the signal
            let mut parser = Parser::new(input_line.as_bytes()).unwrap();
            let signal = Signal::parse(&mut parser).unwrap();

            // Convert to DBC string
            let dbc_string = signal.to_dbc_string();
            assert_eq!(dbc_string, expected_output);

            // Round-trip: parse the output
            let mut parser2 = Parser::new(dbc_string.as_bytes()).unwrap();
            // Skip only the leading space, Signal::parse will handle SG_ keyword
            parser2.skip_newlines_and_spaces();
            let signal2 = Signal::parse(&mut parser2).unwrap();

            // Verify round-trip
            assert_eq!(signal.name(), signal2.name());
            assert_eq!(signal.start_bit(), signal2.start_bit());
            assert_eq!(signal.length(), signal2.length());
            assert_eq!(signal.byte_order(), signal2.byte_order());
            assert_eq!(signal.is_unsigned(), signal2.is_unsigned());
            assert_eq!(signal.factor(), signal2.factor());
            assert_eq!(signal.offset(), signal2.offset());
            assert_eq!(signal.min(), signal2.min());
            assert_eq!(signal.max(), signal2.max());
            assert_eq!(signal.unit(), signal2.unit());
        }
    }
}
