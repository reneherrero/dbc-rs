//! Signal Type definition (SGTYPE_)

use crate::byte_order::ByteOrder;
use crate::compat::String;

/// Represents a Signal Type definition (SGTYPE_)
///
/// Signal Types define reusable signal type definitions with properties that can be
/// referenced by signals using SIG_TYPE_REF_.
///
/// According to the DBC specification, the full format is:
/// `SGTYPE_ type_name : size @ byte_order value_type (factor, offset) [min|max] unit default_value , value_table ;`
#[derive(Debug, Clone, PartialEq)]
pub struct SignalType {
    name: String<{ crate::MAX_NAME_SIZE }>,
    size: u8,
    byte_order: ByteOrder,
    unsigned: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: Option<String<{ crate::MAX_NAME_SIZE }>>,
    default_value: Option<f64>,
    value_table: Option<String<{ crate::MAX_NAME_SIZE }>>,
}

impl SignalType {
    /// Create a new SignalType
    #[cfg_attr(not(feature = "std"), allow(dead_code))] // Only used in std parsing code
    #[allow(clippy::too_many_arguments)] // Internal method, follows spec format
    pub(crate) fn new(
        name: String<{ crate::MAX_NAME_SIZE }>,
        size: u8,
        byte_order: ByteOrder,
        unsigned: bool,
        factor: f64,
        offset: f64,
        min: f64,
        max: f64,
        unit: Option<String<{ crate::MAX_NAME_SIZE }>>,
        default_value: Option<f64>,
        value_table: Option<String<{ crate::MAX_NAME_SIZE }>>,
    ) -> Self {
        Self {
            name,
            size,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            default_value,
            value_table,
        }
    }

    /// Get the signal type name
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the signal type size (in bits)
    #[must_use]
    pub fn size(&self) -> u8 {
        self.size
    }

    /// Get the byte order
    #[must_use]
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    /// Check if the signal type is unsigned
    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        self.unsigned
    }

    /// Get the scaling factor
    #[must_use]
    pub fn factor(&self) -> f64 {
        self.factor
    }

    /// Get the offset value
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// Get the minimum value
    #[must_use]
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Get the maximum value
    #[must_use]
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Get the unit string, if any
    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|u| u.as_ref())
    }

    /// Get the default value, if any
    #[must_use]
    pub fn default_value(&self) -> Option<f64> {
        self.default_value
    }

    /// Get the value table name reference, if any
    #[must_use]
    pub fn value_table(&self) -> Option<&str> {
        self.value_table.as_ref().map(|vt| vt.as_ref())
    }
}

#[cfg(feature = "std")]
impl SignalType {
    /// Parse a Signal Type definition (SGTYPE_)
    ///
    /// Format: SGTYPE_ type_name : size @ byte_order value_type (factor, offset) [min|max] unit default_value , value_table ;
    /// Example: SGTYPE_ SignalType1 : 16@1+ (1.0,0) [0|100] "unit" 0 , "ValueTable" ;
    ///
    /// Note: Many fields may be empty or omitted, using defaults:
    /// - byte_order defaults to LittleEndian (1) if omitted
    /// - value_type defaults to unsigned (+) if omitted
    /// - factor defaults to 1.0 if omitted
    /// - offset defaults to 0.0 if omitted
    /// - min/max may be empty
    /// - unit may be empty string
    /// - default_value may be empty
    /// - value_table may be empty string
    pub(crate) fn parse(parser: &mut crate::parser::Parser) -> crate::error::Result<Self> {
        parser.skip_newlines_and_spaces();
        let type_name = parser.parse_identifier()?;
        let type_name = crate::validate_name(type_name)?;
        parser.skip_newlines_and_spaces();
        parser.expect(b":")?;
        parser.skip_newlines_and_spaces();

        // Parse size
        let size = parser.parse_u32()? as u8;
        parser.skip_newlines_and_spaces();

        // Parse @byte_order value_type (optional, defaults to @1+)
        let (byte_order, unsigned) = if parser.expect(b"@").is_ok() {
            // Parse byte order (0 or 1)
            let bo_byte = if parser.expect(b"0").is_ok() {
                b'0'
            } else if parser.expect(b"1").is_ok() {
                b'1'
            } else {
                return Err(crate::error::Error::Expected(
                    "Expected byte order (0 or 1)",
                ));
            };

            let byte_order = match bo_byte {
                b'0' => ByteOrder::BigEndian,
                b'1' => ByteOrder::LittleEndian,
                _ => unreachable!(),
            };

            // Parse value type (+ or -)
            let sign_byte = if parser.expect(b"+").is_ok() {
                b'+'
            } else if parser.expect(b"-").is_ok() {
                b'-'
            } else {
                return Err(crate::error::Error::Expected(
                    "Expected value type (+ or -)",
                ));
            };

            let unsigned = match sign_byte {
                b'+' => true,
                b'-' => false,
                _ => unreachable!(),
            };

            parser.skip_newlines_and_spaces();
            (byte_order, unsigned)
        } else {
            // Defaults: LittleEndian, unsigned
            (ByteOrder::LittleEndian, true)
        };

        // Parse (factor, offset) - may be empty, defaults to (1.0, 0.0)
        let (factor, offset) = if parser.expect(b"(").is_ok() {
            parser.skip_newlines_and_spaces();
            let factor = parser.parse_f64_or_default(1.0)?;
            parser.expect_then_skip(b",")?;
            let offset = parser.parse_f64_or_default(0.0)?;
            parser.skip_newlines_and_spaces();
            parser.expect(b")")?;
            parser.skip_newlines_and_spaces();
            (factor, offset)
        } else {
            (1.0, 0.0)
        };

        // Parse [min|max] - may be empty
        let (min, max) = if parser.expect(b"[").is_ok() {
            parser.skip_newlines_and_spaces();
            let min = parser.parse_f64_or_default(0.0)?;
            parser.expect_then_skip(b"|")?;
            let max = parser.parse_f64_or_default(0.0)?;
            parser.skip_newlines_and_spaces();
            parser.expect(b"]")?;
            parser.skip_newlines_and_spaces();
            (min, max)
        } else {
            (0.0, 0.0)
        };

        // Parse unit: "unit" or ""
        let unit = if parser.expect(b"\"").is_ok() {
            let unit_bytes =
                parser.take_until_quote(false, crate::MAX_NAME_SIZE).map_err(|e| match e {
                    crate::error::Error::MaxStrLength(_) => {
                        crate::error::Error::Signal(crate::error::Error::SIGNAL_PARSE_UNIT_TOO_LONG)
                    }
                    _ => crate::error::Error::Expected("Expected closing quote"),
                })?;
            let unit_str = core::str::from_utf8(unit_bytes)
                .map_err(|_| crate::error::Error::Expected(crate::error::Error::INVALID_UTF8))?;
            let unit_string: String<{ crate::MAX_NAME_SIZE }> = String::try_from(unit_str)
                .map_err(|_| {
                    crate::error::Error::Version(crate::error::Error::MAX_NAME_SIZE_EXCEEDED)
                })?;
            if unit_string.is_empty() {
                None
            } else {
                Some(unit_string)
            }
        } else {
            None
        };

        parser.skip_newlines_and_spaces();

        // Parse default_value - may be empty (defaults to None)
        let default_value = parser.parse_f64().ok();
        parser.skip_newlines_and_spaces();

        // Parse comma before value_table
        let value_table = if parser.expect(b",").is_ok() {
            parser.skip_newlines_and_spaces();
            // Parse value_table name - may be quoted string or identifier
            if parser.expect(b"\"").is_ok() {
                let vt_bytes =
                    parser.take_until_quote(false, crate::MAX_NAME_SIZE).map_err(|e| match e {
                        crate::error::Error::MaxStrLength(_) => crate::error::Error::Signal(
                            crate::error::Error::SIGNAL_PARSE_UNIT_TOO_LONG,
                        ),
                        _ => crate::error::Error::Expected("Expected closing quote"),
                    })?;
                let vt_str = core::str::from_utf8(vt_bytes).map_err(|_| {
                    crate::error::Error::Expected(crate::error::Error::INVALID_UTF8)
                })?;
                if vt_str.is_empty() {
                    None
                } else {
                    let vt_string: String<{ crate::MAX_NAME_SIZE }> = String::try_from(vt_str)
                        .map_err(|_| {
                            crate::error::Error::Version(
                                crate::error::Error::MAX_NAME_SIZE_EXCEEDED,
                            )
                        })?;
                    Some(vt_string)
                }
            } else if let Ok(vt_ident) = parser.parse_identifier() {
                let vt_string: String<{ crate::MAX_NAME_SIZE }> = String::try_from(vt_ident)
                    .map_err(|_| {
                        crate::error::Error::Version(crate::error::Error::MAX_NAME_SIZE_EXCEEDED)
                    })?;
                Some(vt_string)
            } else {
                None
            }
        } else {
            None
        };

        parser.skip_newlines_and_spaces();

        // Semicolon is optional but common
        if parser.starts_with(b";") {
            parser.expect(b";").ok();
        }

        Ok(SignalType::new(
            type_name,
            size,
            byte_order,
            unsigned,
            factor,
            offset,
            min,
            max,
            unit,
            default_value,
            value_table,
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_creation() {
        let name = crate::validate_name("SignalType1").unwrap();
        let signal_type = SignalType::new(
            name,
            16,
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None,
            None,
            None,
        );
        assert_eq!(signal_type.name(), "SignalType1");
        assert_eq!(signal_type.size(), 16);
        assert_eq!(signal_type.byte_order(), ByteOrder::LittleEndian);
        assert!(signal_type.is_unsigned());
        assert_eq!(signal_type.factor(), 1.0);
        assert_eq!(signal_type.offset(), 0.0);
        assert_eq!(signal_type.min(), 0.0);
        assert_eq!(signal_type.max(), 100.0);
    }

    #[test]
    fn test_signal_type_equality() {
        let name1 = crate::validate_name("SignalType1").unwrap();
        let name2 = crate::validate_name("SignalType1").unwrap();
        let st1 = SignalType::new(
            name1,
            16,
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None,
            None,
            None,
        );
        let st2 = SignalType::new(
            name2,
            16,
            ByteOrder::LittleEndian,
            true,
            1.0,
            0.0,
            0.0,
            100.0,
            None,
            None,
            None,
        );
        assert_eq!(st1, st2);
    }

    #[test]
    fn test_parse_minimal() {
        // Minimal format: just name and size (should use defaults)
        let content = b"SignalType1 : 16;";
        let mut parser = crate::Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let signal_type = SignalType::parse(&mut parser).unwrap();
        assert_eq!(signal_type.name(), "SignalType1");
        assert_eq!(signal_type.size(), 16);
        assert_eq!(signal_type.byte_order(), ByteOrder::LittleEndian); // default
        assert!(signal_type.is_unsigned()); // default
        assert_eq!(signal_type.factor(), 1.0); // default
        assert_eq!(signal_type.offset(), 0.0); // default
    }

    #[test]
    fn test_parse_full() {
        // Full format with all fields
        let content = b"SignalType1 : 16@1+ (0.25,10) [0|8000] \"rpm\" 0 , \"GearTable\" ;";
        let mut parser = crate::Parser::new(content).unwrap();
        let signal_type = SignalType::parse(&mut parser).unwrap();
        assert_eq!(signal_type.name(), "SignalType1");
        assert_eq!(signal_type.size(), 16);
        assert_eq!(signal_type.byte_order(), ByteOrder::LittleEndian);
        assert!(signal_type.is_unsigned());
        assert_eq!(signal_type.factor(), 0.25);
        assert_eq!(signal_type.offset(), 10.0);
        assert_eq!(signal_type.min(), 0.0);
        assert_eq!(signal_type.max(), 8000.0);
        assert_eq!(signal_type.unit(), Some("rpm"));
        assert_eq!(signal_type.default_value(), Some(0.0));
        assert_eq!(signal_type.value_table(), Some("GearTable"));
    }

    #[test]
    fn test_parse_big_endian_signed() {
        let content = b"SignalType1 : 8@0- (1,-40) [-40|215] \"degC\" ;";
        let mut parser = crate::Parser::new(content).unwrap();
        let signal_type = SignalType::parse(&mut parser).unwrap();
        assert_eq!(signal_type.byte_order(), ByteOrder::BigEndian);
        assert!(!signal_type.is_unsigned());
        assert_eq!(signal_type.factor(), 1.0);
        assert_eq!(signal_type.offset(), -40.0);
        assert_eq!(signal_type.min(), -40.0);
        assert_eq!(signal_type.max(), 215.0);
        assert_eq!(signal_type.unit(), Some("degC"));
    }

    // Integration tests using Dbc::parse
    #[test]
    fn test_parse_sgtype_definition() {
        let dbc_content = r#"
VERSION "1.0"

BS_:

BU_: ECM

SGTYPE_ SignalType1 : 16;
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse SGTYPE_");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 1);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
    }

    #[test]
    fn test_parse_sgtype_multiple() {
        let dbc_content = r#"
VERSION "1.0"

BS_:

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 32;
SGTYPE_ SignalType3 : 8;
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse multiple SGTYPE_");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 3);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
        assert_eq!(signal_types[1].name(), "SignalType2");
        assert_eq!(signal_types[1].size(), 32);
        assert_eq!(signal_types[2].name(), "SignalType3");
        assert_eq!(signal_types[2].size(), 8);
    }

    #[test]
    fn test_parse_sgtype_without_semicolon() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse SGTYPE_ without semicolon");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 1);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
    }
}
