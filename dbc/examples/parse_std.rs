//! Example: Reading DBC files from various sources (std only).
//!
//! This example demonstrates different ways to parse DBC files when using the `std` feature:
//! - From bytes using `parse_bytes`
//! - From owned strings using `parse`
//! - From file/reader (reading to buffer first)

use dbc_rs::Dbc;
use std::io::{Cursor, Read};

fn main() -> Result<(), dbc_rs::Error> {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    // Method 1: Parse from bytes
    println!("1. Parsing from &[u8]:");
    let bytes = dbc_content.as_bytes();
    let dbc1 = Dbc::parse_bytes(bytes)?;
    println!("   Parsed {} messages", dbc1.messages().len());

    // Method 2: Parse from String
    println!("2. Parsing from String:");
    let string = String::from(dbc_content);
    let dbc2 = Dbc::parse(&string)?;
    println!("   Parsed {} messages", dbc2.messages().len());

    // Method 3: Parse from std::io::Read (Cursor)
    println!("3. Parsing from std::io::Read (Cursor):");
    let mut cursor = Cursor::new(dbc_content.as_bytes());
    let mut buffer = Vec::new();
    cursor
        .read_to_end(&mut buffer)
        .map_err(|e| dbc_rs::Error::Dbc(format!("Read error: {}", e)))?;
    let dbc3 = Dbc::parse_bytes(&buffer)?;
    println!("   Parsed {} messages", dbc3.messages().len());

    // Method 4: Parse from File
    println!("4. Parsing from File:");
    if let Ok(mut file) = std::fs::File::open("tests/data/simple.dbc") {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| dbc_rs::Error::Dbc(format!("Read error: {}", e)))?;
        let dbc4 = Dbc::parse_bytes(&buffer)?;
        println!("   Parsed {} messages from file", dbc4.messages().len());
    } else {
        println!("   File not found (this is OK for the example)");
    }

    Ok(())
}
