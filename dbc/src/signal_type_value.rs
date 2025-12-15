//! Signal Type Value Descriptions (SGTYPE_VAL_)

use crate::compat::String;

/// Represents Signal Type Value Descriptions (SGTYPE_VAL_)
///
/// Value descriptions for signal types, similar to VAL_ but for types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct SignalTypeValue {
    type_name: String<{ crate::MAX_NAME_SIZE }>,
    value: u64,
    description: String<{ crate::MAX_NAME_SIZE }>,
}

impl SignalTypeValue {
    /// Create a new SignalTypeValue
    #[cfg_attr(not(feature = "std"), allow(dead_code))] // Only used in std parsing code
    pub(crate) fn new(
        type_name: String<{ crate::MAX_NAME_SIZE }>,
        value: u64,
        description: String<{ crate::MAX_NAME_SIZE }>,
    ) -> Self {
        Self {
            type_name,
            value,
            description,
        }
    }

    /// Get the signal type name
    #[must_use]
    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }

    /// Get the value
    #[must_use]
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Get the description
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

#[cfg(feature = "std")]
impl SignalTypeValue {
    /// Parse Signal Type Value Descriptions (SGTYPE_VAL_)
    ///
    /// Format: SGTYPE_VAL_ type_name value "description" value "description" ... ;
    /// Example: SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" 2 "Two";
    pub(crate) fn parse(
        parser: &mut crate::parser::Parser,
    ) -> crate::error::Result<std::vec::Vec<Self>> {
        parser.skip_newlines_and_spaces();
        let type_name = parser.parse_identifier()?;
        let type_name = crate::validate_name(type_name)?;
        parser.skip_newlines_and_spaces();

        let mut values = std::vec::Vec::new();

        loop {
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b";") {
                parser.expect(b";").ok();
                break;
            }
            if parser.is_empty() {
                break;
            }

            // Parse value
            if let Ok(value) = parser.parse_u32() {
                parser.skip_newlines_and_spaces();
                // Parse description (quoted string)
                let description = if parser.expect(b"\"").is_ok() {
                    let desc_bytes = parser.take_until_quote(false, 1024).ok();
                    if let Some(desc_bytes) = desc_bytes {
                        core::str::from_utf8(desc_bytes)
                            .ok()
                            .and_then(|s| crate::validate_name(s).ok())
                            .unwrap_or_default()
                    } else {
                        crate::compat::String::<{ crate::MAX_NAME_SIZE }>::new()
                    }
                } else {
                    crate::compat::String::<{ crate::MAX_NAME_SIZE }>::new()
                };
                values.push(SignalTypeValue::new(
                    type_name.clone(),
                    value as u64,
                    description,
                ));
            } else {
                // Not a value, might be end of line or semicolon
                if parser.starts_with(b";") {
                    parser.expect(b";").ok();
                }
                break;
            }
        }

        Ok(values)
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_value_creation() {
        let type_name = crate::validate_name("SignalType1").unwrap();
        let description = crate::validate_name("Zero").unwrap();
        let value = SignalTypeValue::new(type_name, 0, description);
        assert_eq!(value.type_name(), "SignalType1");
        assert_eq!(value.value(), 0);
        assert_eq!(value.description(), "Zero");
    }

    #[test]
    fn test_signal_type_value_equality() {
        let type_name1 = crate::validate_name("SignalType1").unwrap();
        let description1 = crate::validate_name("Zero").unwrap();
        let type_name2 = crate::validate_name("SignalType1").unwrap();
        let description2 = crate::validate_name("Zero").unwrap();
        let val1 = SignalTypeValue::new(type_name1, 0, description1);
        let val2 = SignalTypeValue::new(type_name2, 0, description2);
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_parse() {
        let content = b"SignalType1 0 \"Zero\" 1 \"One\" 2 \"Two\";";
        let mut parser = crate::Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let values = SignalTypeValue::parse(&mut parser).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Zero");
        assert_eq!(values[1].value(), 1);
        assert_eq!(values[1].description(), "One");
        assert_eq!(values[2].value(), 2);
        assert_eq!(values[2].description(), "Two");
    }

    // Integration tests using Dbc::parse
    #[test]
    fn test_parse_sgtype_val() {
        let dbc_content = r#"
VERSION "1.0"

BS_:

BU_: ECM

SGTYPE_ SignalType1 : 16;

SGTYPE_VAL_ SignalType1 0 "Value0" 1 "Value1" 2 "Value2" ;
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse SGTYPE_VAL_");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].type_name(), "SignalType1");
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Value0");
        assert_eq!(values[1].value(), 1);
        assert_eq!(values[1].description(), "Value1");
        assert_eq!(values[2].value(), 2);
        assert_eq!(values[2].description(), "Value2");
    }

    #[test]
    fn test_parse_sgtype_val_multiple_types() {
        let dbc_content = r#"
VERSION "1.0"

BS_:

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 8;

SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" ;
SGTYPE_VAL_ SignalType2 0 "Off" 1 "On" 2 "Error" ;
"#;

        let dbc = crate::Dbc::parse(dbc_content).expect("Should parse multiple SGTYPE_VAL_");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 5);

        // Check SignalType1 values
        let type1_values: Vec<&SignalTypeValue> =
            values.iter().filter(|v| v.type_name() == "SignalType1").collect();
        assert_eq!(type1_values.len(), 2);
        assert_eq!(type1_values[0].value(), 0);
        assert_eq!(type1_values[0].description(), "Zero");
        assert_eq!(type1_values[1].value(), 1);
        assert_eq!(type1_values[1].description(), "One");

        // Check SignalType2 values
        let type2_values: Vec<&SignalTypeValue> =
            values.iter().filter(|v| v.type_name() == "SignalType2").collect();
        assert_eq!(type2_values.len(), 3);
        assert_eq!(type2_values[0].value(), 0);
        assert_eq!(type2_values[0].description(), "Off");
        assert_eq!(type2_values[1].value(), 1);
        assert_eq!(type2_values[1].description(), "On");
        assert_eq!(type2_values[2].value(), 2);
        assert_eq!(type2_values[2].description(), "Error");
    }

    #[test]
    fn test_parse_sgtype_val_single_value() {
        let dbc_content = r#"
VERSION "1.0"

BS_:

BU_: ECM

SGTYPE_ SignalType1 : 16;

SGTYPE_VAL_ SignalType1 0 "Zero" ;
"#;

        let dbc =
            crate::Dbc::parse(dbc_content).expect("Should parse SGTYPE_VAL_ with single value");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].type_name(), "SignalType1");
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Zero");
    }
}
