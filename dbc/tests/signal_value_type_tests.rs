//! Tests for Signal Extended Value Types (SIG_VALTYPE_)
//! 
//! Tests parsing of SIG_VALTYPE_ entries and decoding of float32/float64 signals.

#[cfg(feature = "std")]
mod std {
    use dbc_rs::{Dbc, SignalExtendedValueType};

    #[test]
    fn test_parse_sig_valtype_float32() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@1+ (1,0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");

        // Verify signal value type is stored
        let value_type = dbc.get_signal_value_type(256, "Temperature");
        assert_eq!(value_type, Some(SignalExtendedValueType::Float32));
    }

    #[test]
    fn test_parse_sig_valtype_float64() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Pressure : 0|64@1+ (1,0) [0|1000] "kPa" *

SIG_VALTYPE_ 256 Pressure : 2 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");

        // Verify signal value type is stored
        let value_type = dbc.get_signal_value_type(256, "Pressure");
        assert_eq!(value_type, Some(SignalExtendedValueType::Float64));
    }

    #[test]
    fn test_parse_sig_valtype_integer() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

SIG_VALTYPE_ 256 RPM : 0 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");

        // Verify signal value type is stored
        let value_type = dbc.get_signal_value_type(256, "RPM");
        assert_eq!(value_type, Some(SignalExtendedValueType::Integer));
    }

    #[test]
    fn test_parse_sig_valtype_multiple_signals() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@1+ (1,0) [0|100] "°C" *
 SG_ Pressure : 32|32@1+ (1,0) [0|1000] "kPa" *

SIG_VALTYPE_ 256 Temperature : 1 ;
SIG_VALTYPE_ 256 Pressure : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");

        // Verify both signals have float32 type
        assert_eq!(
            dbc.get_signal_value_type(256, "Temperature"),
            Some(SignalExtendedValueType::Float32)
        );
        assert_eq!(
            dbc.get_signal_value_type(256, "Pressure"),
            Some(SignalExtendedValueType::Float32)
        );
    }

    #[test]
    fn test_decode_float32_little_endian() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@1+ (1,0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Temperature").unwrap();

        // Test value: 3.14159 (IEEE 754 float32: 0x40490FDB)
        // Little-endian bytes: [0xDB, 0x0F, 0x49, 0x40]
        let data = [0xDB, 0x0F, 0x49, 0x40, 0x00, 0x00, 0x00, 0x00];

        let value_type = dbc.get_signal_value_type(256, "Temperature");
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();

        // Should be approximately 3.14159
        assert!((decoded - 3.14159).abs() < 0.00001);
    }

    #[test]
    #[ignore = "Big-endian float extraction needs verification"]
    fn test_decode_float32_big_endian() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@0+ (1,0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Temperature").unwrap();

        // For big-endian, the bit extraction works differently
        // Test with a simple value: 1.0 (IEEE 754 float32: 0x3F800000)
        // Big-endian byte order: [0x3F, 0x80, 0x00, 0x00]
        let data = [0x3F, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let value_type = dbc.get_signal_value_type(256, "Temperature");
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();

        // Should be approximately 1.0
        assert!((decoded - 1.0).abs() < 0.00001);
    }

    #[test]
    fn test_decode_float32_with_factor_and_offset() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@1+ (2.0,10.0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Temperature").unwrap();

        // Test value: 1.5 (IEEE 754 float32: 0x3FC00000)
        // Little-endian bytes: [0x00, 0x00, 0xC0, 0x3F]
        let data = [0x00, 0x00, 0xC0, 0x3F, 0x00, 0x00, 0x00, 0x00];

        let value_type = dbc.get_signal_value_type(256, "Temperature");
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();

        // Should be: 1.5 * 2.0 + 10.0 = 13.0
        assert!((decoded - 13.0).abs() < 0.00001);
    }

    #[test]
    fn test_decode_float64_little_endian() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Pressure : 0|64@1+ (1,0) [0|1000] "kPa" *

SIG_VALTYPE_ 256 Pressure : 2 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Pressure").unwrap();

        // Test value: 3.141592653589793 (IEEE 754 float64: 0x400921FB54442D18)
        // Little-endian bytes: [0x18, 0x2D, 0x44, 0x54, 0xFB, 0x21, 0x09, 0x40]
        let data = [0x18, 0x2D, 0x44, 0x54, 0xFB, 0x21, 0x09, 0x40];

        let value_type = dbc.get_signal_value_type(256, "Pressure");
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();

        // Should be approximately 3.141592653589793
        assert!((decoded - 3.141592653589793).abs() < 0.000000000000001);
    }

    #[test]
    #[ignore = "Big-endian float extraction needs verification"]
    fn test_decode_float64_big_endian() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Pressure : 0|64@0+ (1,0) [0|1000] "kPa" *

SIG_VALTYPE_ 256 Pressure : 2 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Pressure").unwrap();

        // For big-endian, test with a simple value: 1.0 (IEEE 754 float64: 0x3FF0000000000000)
        // Big-endian byte order: [0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        let data = [0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let value_type = dbc.get_signal_value_type(256, "Pressure");
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();

        // Should be approximately 1.0
        assert!((decoded - 1.0).abs() < 0.000000000000001);
    }

    #[test]
    fn test_decode_float32_edge_cases() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|32@1+ (1,0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Temperature").unwrap();
        let value_type = dbc.get_signal_value_type(256, "Temperature");

        // Test zero
        let data_zero = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded_zero = signal.decode_with_value_type(&data_zero, value_type).unwrap();
        assert_eq!(decoded_zero, 0.0);

        // Test negative zero (should still be 0.0)
        let data_neg_zero = [0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00];
        let decoded_neg_zero = signal.decode_with_value_type(&data_neg_zero, value_type).unwrap();
        assert_eq!(decoded_neg_zero, 0.0);

        // Test infinity (0x7F800000)
        let data_inf = [0x00, 0x00, 0x80, 0x7F, 0x00, 0x00, 0x00, 0x00];
        let decoded_inf = signal.decode_with_value_type(&data_inf, value_type).unwrap();
        assert!(decoded_inf.is_infinite() && decoded_inf.is_sign_positive());

        // Test negative infinity (0xFF800000)
        let data_neg_inf = [0x00, 0x00, 0x80, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let decoded_neg_inf = signal.decode_with_value_type(&data_neg_inf, value_type).unwrap();
        assert!(decoded_neg_inf.is_infinite() && decoded_neg_inf.is_sign_negative());
    }

    #[test]
    fn test_decode_float32_invalid_length() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Temperature : 0|16@1+ (1,0) [0|100] "°C" *

SIG_VALTYPE_ 256 Temperature : 1 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Temperature").unwrap();
        let value_type = dbc.get_signal_value_type(256, "Temperature");

        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        // Should fail because float32 requires 32 bits, but signal is 16 bits
        let result = signal.decode_with_value_type(&data, value_type);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Float32"));
    }

    #[test]
    fn test_decode_float64_invalid_length() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Pressure : 0|32@1+ (1,0) [0|1000] "kPa" *

SIG_VALTYPE_ 256 Pressure : 2 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("Pressure").unwrap();
        let value_type = dbc.get_signal_value_type(256, "Pressure");

        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        // Should fail because float64 requires 64 bits, but signal is 32 bits
        let result = signal.decode_with_value_type(&data, value_type);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Float64"));
    }

    #[test]
    fn test_decode_integer_fallback() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("RPM").unwrap();

        // No SIG_VALTYPE_ entry, so should default to integer
        let value_type = dbc.get_signal_value_type(256, "RPM");
        assert_eq!(value_type, None);

        // Decode should work with integer decoding
        let data = [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 32 in little-endian
        let decoded = signal.decode(&data).unwrap();
        assert_eq!(decoded, 8.0); // 32 * 0.25 = 8.0
    }

    #[test]
    fn test_decode_with_explicit_integer_type() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

SIG_VALTYPE_ 256 RPM : 0 ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Failed to parse DBC");
        let message = dbc.messages().find_by_id(256).unwrap();
        let signal = message.signals().find("RPM").unwrap();

        let value_type = dbc.get_signal_value_type(256, "RPM");
        assert_eq!(value_type, Some(SignalExtendedValueType::Integer));

        // Decode should work with integer decoding
        let data = [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 32 in little-endian
        let decoded = signal.decode_with_value_type(&data, value_type).unwrap();
        assert_eq!(decoded, 8.0); // 32 * 0.25 = 8.0
    }
}

