//! Example: Reading DBC files from string slices (no_std compatible).
//!
//! This example demonstrates parsing DBC files in `no_std` environments.
//! The `Dbc::parse()` method works without the standard library and only
//! requires `alloc` for the `Vec<Message>` storage.

use dbc_rs::Dbc;
#[cfg(not(feature = "std"))]
extern crate alloc;

fn main() -> Result<(), dbc_rs::Error> {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    // Parse from string slice (works in no_std)
    println!("Parsing from &str (no_std compatible):");
    let dbc = Dbc::parse(dbc_content)?;
    println!("   Parsed {} messages", dbc.messages().len());
    if let Some(version) = dbc.version() {
        println!("   Version: {}", version.as_str());
    } else {
        println!("   Version: (none)");
    }
    if let Some(nodes) = dbc.nodes().nodes() {
        let nodes_str: String =
            nodes.iter().enumerate().fold(String::new(), |mut acc, (i, node)| {
                if i > 0 {
                    acc.push(' ');
                }
                acc.push_str(node);
                acc
            });
        println!("   Nodes: {}", nodes_str);
    } else {
        println!("   Nodes: (none)");
    }

    Ok(())
}
