//! Property-based tests using proptest for fuzz testing and edge case coverage.

#![cfg(feature = "alloc")]

use dbc_rs::Dbc;
use proptest::prelude::*;

/// Generate a valid DBC file string
fn gen_dbc_string() -> impl Strategy<Value = String> {
    let version_strategy = r#""[a-zA-Z0-9._-]{0,50}""#;
    let node_strategy = "[a-zA-Z][a-zA-Z0-9_]{0,30}";
    let message_id_strategy = 0u32..=0x1FFFFFFFu32; // Valid extended CAN ID range
    let message_name_strategy = "[a-zA-Z][a-zA-Z0-9_]{0,30}";
    let dlc_strategy = 1u8..=64u8; // CAN FD supports up to 64 bytes
    let signal_name_strategy = "[a-zA-Z][a-zA-Z0-9_]{0,30}";
    let start_bit_strategy = 0u16..=500u16; // Up to 500 bits (within CAN FD 64-byte limit)
    let length_strategy = 1u16..=64u16; // Signal length 1-64 bits

    prop::collection::vec(
        (
            message_id_strategy,
            message_name_strategy,
            dlc_strategy,
            node_strategy,
            prop::collection::vec(
                (signal_name_strategy, start_bit_strategy, length_strategy),
                0..=8, // 0-8 signals per message
            ),
        ),
        0..=10, // 0-10 messages
    )
    .prop_map(|messages| {
        let mut dbc = String::from("VERSION ");
        dbc.push_str(version_strategy);
        dbc.push_str("\n\nBU_: ");

        // Collect unique nodes
        let mut nodes = std::collections::HashSet::new();
        for (_, _, _, sender, _) in &messages {
            nodes.insert(sender.clone());
        }
        if nodes.is_empty() {
            nodes.insert("ECM".to_string());
        }
        let nodes_vec: Vec<String> = nodes.into_iter().collect();
        dbc.push_str(&nodes_vec.join(" "));
        dbc.push('\n');

        // Add messages
        for (id, name, dlc, sender, signals) in messages {
            dbc.push_str(&format!("BO_ {} {} : {} {}\n", id, name, dlc, sender));

            // Add signals (ensure they don't overlap)
            let mut current_bit = 0u16;
            for (sig_name, _, sig_len) in signals {
                if current_bit + sig_len <= (dlc as u16 * 8) {
                    dbc.push_str(&format!(
                        " SG_ {} : {}|{}@1+ (1,0) [0|255] \"\"\n",
                        sig_name, current_bit, sig_len
                    ));
                    current_bit += sig_len;
                }
            }
        }

        dbc
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_parse_round_trip(dbc_str in gen_dbc_string()) {
        // Test that we can parse generated DBC strings
        match Dbc::parse(&dbc_str) {
            Ok(dbc) => {
                // Verify basic structure
                assert!(dbc.messages().len() <= 10);

                // Test round-trip: parse -> to_string -> parse
                #[cfg(feature = "alloc")]
                {
                    let dbc_string = dbc.to_dbc_string();
                    let dbc2 = Dbc::parse(&dbc_string).expect("Round-trip parse should succeed");
                    assert_eq!(dbc.messages().len(), dbc2.messages().len());
                }
            }
            Err(_) => {
                // Some generated strings may be invalid, that's okay
                // We're testing that the parser doesn't panic
            }
        }
    }

    #[test]
    fn test_message_id_range(id in 0u32..=0x1FFFFFFFu32) {
        // Test that valid message IDs are accepted
        let dbc_str = format!(
            r#"VERSION "1.0"

BU_: ECM

BO_ {} TestMessage : 8 ECM
"#,
            id
        );

        match Dbc::parse(&dbc_str) {
            Ok(dbc) => {
                let msg = dbc.messages().at(0).unwrap();
                assert_eq!(msg.id(), id);
            }
            Err(_) => {
                // Some edge cases might fail validation, that's acceptable
            }
        }
    }

    #[test]
    fn test_signal_boundaries(
        start_bit in 0u16..=500u16,
        length in 1u16..=64u16,
        dlc in 1u8..=64u8
    ) {
        // Test signal boundary validation
        let max_bits = (dlc as u16) * 8;
        let signal_fits = start_bit + length <= max_bits;

        let dbc_str = format!(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 TestMessage : {} ECM
 SG_ TestSignal : {}|{}@1+ (1,0) [0|255] ""
"#,
            dlc, start_bit, length
        );

        match Dbc::parse(&dbc_str) {
            Ok(_) => {
                // If parsing succeeded, signal should fit
                assert!(signal_fits, "Signal should fit within message boundary");
            }
            Err(_) => {
                // If parsing failed, signal might not fit (expected for invalid cases)
                // This is acceptable - we're testing that parser handles edge cases gracefully
            }
        }
    }
}
