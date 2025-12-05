//! Example: Reading DBC files from string slices (pure no_std, no alloc).
//!
//! This example demonstrates parsing DBC files in pure `no_std` environments
//! without any dependencies on `std` or `alloc`. The `Dbc::parse()` method
//! works without the standard library and without allocation.
//!
//! Note: In pure no_std mode, messages are not parsed (they require alloc).
//! Only VERSION and BU_ sections are parsed.

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

    // Parse from string slice (works in pure no_std, no alloc required)
    if let Ok(dbc) = Dbc::parse(dbc_content) {
        // Access version (no alloc required)
        let _version_str = dbc.version().map(|v| v.as_str()).unwrap_or("");

        // Access nodes using iterator (no alloc required)
        // Note: iter_nodes() is only available in no_std builds
        // In std builds, use nodes() method instead
        #[cfg(not(feature = "std"))]
        let _node_count = dbc.nodes().iter_nodes().count();
        #[cfg(feature = "std")]
        let _node_count = dbc.nodes().nodes().map(|n| n.len()).unwrap_or(0);

        // In a real embedded system, you would use these values here
        // For this example, we just verify parsing succeeded
        // Messages are empty in no_std mode (require alloc)
        let _messages_count = dbc.message_count(); // Will be 0 in no_std

    // In embedded: set success flag, update status register, etc.
    } else {
        // In embedded: set error flag, trigger error handler, etc.
    }
}
