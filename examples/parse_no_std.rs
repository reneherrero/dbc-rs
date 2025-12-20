//! Example: Parsing DBC files in no_std environments.
//!
//! This example demonstrates parsing DBC files in `no_std` environments.
//! The `Dbc::parse()` method works without the standard library.
//!
//! Note: To parse messages, the `alloc` feature must be enabled.
//! In pure no_std without alloc, only VERSION and BU_ sections are parsed.

// Note: This example demonstrates no_std usage, but the example itself
// runs with std for output. The library code uses no_std when compiled
// with --no-default-features.

use dbc_rs::Dbc;

fn main() {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    // Parse from string slice (works in no_std with alloc feature)
    match Dbc::parse(dbc_content) {
        Ok(dbc) => {
            // Access version (available in both std and no_std)
            if let Some(version) = dbc.version() {
                println!("Version: {}", version.as_str());
            }

            // Access nodes (available in both std and no_std)
            println!("Nodes: {} found", dbc.nodes().len());
            for node in dbc.nodes().iter() {
                println!("  - {}", node);
            }

            // With alloc feature, messages are parsed
            println!("Messages: {} found", dbc.messages().len());
            for message in dbc.messages().iter() {
                println!("  - {} (ID: {})", message.name(), message.id());
                println!("    Signals: {}", message.signals().len());
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            // In embedded: set error flag, trigger error handler, etc.
        }
    }
}
