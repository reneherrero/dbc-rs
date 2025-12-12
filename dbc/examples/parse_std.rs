//! Example: Reading DBC files from various sources (std only).
//!
//! This example demonstrates parsing DBC files when using the `std` feature:
//! - From string using `parse`
//! - From file (reading to buffer first, then using `parse_bytes`)

use dbc_rs::Dbc;
use std::io::Read;

fn main() -> Result<(), dbc_rs::Error> {
    // Parse from string
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    let dbc = Dbc::parse(dbc_content)?;
    println!("Parsed {} messages from string", dbc.messages().len());

    // Parse from file
    if let Ok(mut file) = std::fs::File::open("tests/data/simple.dbc") {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| dbc_rs::Error::Dbc(format!("Read error: {}", e)))?;
        let dbc_file = Dbc::parse_bytes(&buffer)?;
        println!("Parsed {} messages from file", dbc_file.messages().len());
    } else {
        println!("File not found (this is OK for the example)");
    }

    Ok(())
}
