use std::env;

/// Check if a number is a power of 2
fn is_power_of_2(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Calculate the next power of 2 greater than or equal to n
fn next_power_of_2(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    if is_power_of_2(n) {
        return n;
    }
    let mut v = n;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    #[cfg(target_pointer_width = "64")]
    {
        v |= v >> 32;
    }
    v + 1
}

fn main() {
    // Enforce that either alloc or heapless feature must be enabled
    // Check Cargo feature flags via CARGO_FEATURE_* environment variables
    // (Note: hyphens become underscores, all uppercase)
    let has_alloc = env::var("CARGO_FEATURE_ALLOC").is_ok();
    let has_heapless = env::var("CARGO_FEATURE_HEAPLESS").is_ok();
    let has_std = env::var("CARGO_FEATURE_STD").is_ok();
    let has_attributes = env::var("CARGO_FEATURE_ATTRIBUTES").is_ok();

    // std includes alloc, so we only need to check if neither alloc nor heapless is enabled
    // Note: This check provides a better error message, but mayheap will also enforce this
    if !has_alloc && !has_heapless && !has_std {
        eprintln!("error: Either the `alloc` or `heapless` feature must be enabled");
        eprintln!("\ndbc-rs requires one of the following features:");
        eprintln!("  - `alloc`: Heap-allocated collections via alloc crate");
        eprintln!("  - `heapless`: Stack-allocated, bounded collections");
        eprintln!("  - `std`: Includes alloc + standard library features (default)");
        eprintln!("\nAdd to Cargo.toml:");
        eprintln!("  [dependencies]");
        eprintln!(
            "  dbc-rs = {{ version = \"...\", default-features = false, features = [\"alloc\"] }}"
        );
        eprintln!("  # OR");
        eprintln!(
            "  dbc-rs = {{ version = \"...\", default-features = false, features = [\"heapless\"] }}"
        );
        std::process::exit(1);
    }

    // Allow override of MAX_SIGNALS_PER_MESSAGE via environment variable
    let max_signals = env::var("DBC_MAX_SIGNALS_PER_MESSAGE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256); // Default to 256

    // Allow override of MAX_MESSAGES via environment variable
    let max_messages = env::var("DBC_MAX_MESSAGES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8192); // Default to 8192 (2^13, power of 2)

    // Allow override of MAX_NODES via environment variable
    let max_nodes = env::var("DBC_MAX_NODES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256); // Default to 256

    // Allow override of MAX_VALUE_DESCRIPTIONS via environment variable
    let max_value_descriptions = env::var("DBC_MAX_VALUE_DESCRIPTIONS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64); // Default to 64

    // Allow override of MAX_NAME_SIZE via environment variable
    let max_name_size = env::var("DBC_MAX_NAME_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(32); // Default to 32 (per DBC specification)

    // Allow override of MAX_EXTENDED_MULTIPLEXING via environment variable
    let max_extended_multiplexing = env::var("DBC_MAX_EXTENDED_MULTIPLEXING")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(512); // Default to 512 (power of 2, per-file limit for extended multiplexing entries)

    // Attribute limits (only when attributes feature is enabled)
    let (max_attribute_definitions, max_attribute_values, max_attribute_enum_values) =
        if has_attributes {
            let max_attribute_definitions = env::var("DBC_MAX_ATTRIBUTE_DEFINITIONS")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(256);

            let max_attribute_values = env::var("DBC_MAX_ATTRIBUTE_VALUES")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(4096);

            let max_attribute_enum_values = env::var("DBC_MAX_ATTRIBUTE_ENUM_VALUES")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(64);

            (
                Some(max_attribute_definitions),
                Some(max_attribute_values),
                Some(max_attribute_enum_values),
            )
        } else {
            (None, None, None)
        };

    // Validate that all values are powers of 2 when heapless feature is enabled
    // heapless::Vec, heapless::String, and heapless::FnvIndexMap require power-of-2 capacities
    if has_heapless {
        let mut heapless_constants: Vec<(&str, usize, &str)> = vec![
            ("DBC_MAX_MESSAGES", max_messages, "MAX_MESSAGES"),
            (
                "DBC_MAX_SIGNALS_PER_MESSAGE",
                max_signals,
                "MAX_SIGNALS_PER_MESSAGE",
            ),
            ("DBC_MAX_NODES", max_nodes, "MAX_NODES"),
            ("DBC_MAX_NAME_SIZE", max_name_size, "MAX_NAME_SIZE"),
            (
                "DBC_MAX_EXTENDED_MULTIPLEXING",
                max_extended_multiplexing,
                "MAX_EXTENDED_MULTIPLEXING",
            ),
        ];

        // Add attribute constants only when feature is enabled
        if let Some(val) = max_attribute_definitions {
            heapless_constants.push((
                "DBC_MAX_ATTRIBUTE_DEFINITIONS",
                val,
                "MAX_ATTRIBUTE_DEFINITIONS",
            ));
        }
        if let Some(val) = max_attribute_values {
            heapless_constants.push(("DBC_MAX_ATTRIBUTE_VALUES", val, "MAX_ATTRIBUTE_VALUES"));
        }
        if let Some(val) = max_attribute_enum_values {
            heapless_constants.push((
                "DBC_MAX_ATTRIBUTE_ENUM_VALUES",
                val,
                "MAX_ATTRIBUTE_ENUM_VALUES",
            ));
        }

        for (env_var, value, const_name) in heapless_constants.iter() {
            if !is_power_of_2(*value) {
                eprintln!(
                    "error: {} must be a power of 2 when using `heapless` feature",
                    const_name
                );
                eprintln!("  Current value: {} (set via {}={})", value, env_var, value);
                eprintln!(
                    "  {} is used with heapless collections which require power-of-2 capacities.",
                    const_name
                );
                eprintln!(
                    "\nValid power-of-2 values: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, ..."
                );
                eprintln!("\nExample: Set {} to a power of 2:", env_var);
                eprintln!("  {}={} cargo build ...", env_var, next_power_of_2(*value));
                std::process::exit(1);
            }
        }
    }

    // Write the constants to a file in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("limits.rs");

    let mut limits_content = format!(
        r#"#[allow(dead_code)]
pub const MAX_SIGNALS_PER_MESSAGE: usize = {};
#[allow(dead_code)]
pub const MAX_MESSAGES: usize = {};
#[allow(dead_code)]
pub const MAX_NODES: usize = {};
#[allow(dead_code)]
pub const MAX_VALUE_DESCRIPTIONS: usize = {};
#[allow(dead_code)]
pub const MAX_NAME_SIZE: usize = {};
#[allow(dead_code)]
pub const MAX_EXTENDED_MULTIPLEXING: usize = {};
"#,
        max_signals,
        max_messages,
        max_nodes,
        max_value_descriptions,
        max_name_size,
        max_extended_multiplexing
    );

    // Add attribute constants only when feature is enabled
    if let (Some(attr_defs), Some(attr_vals), Some(attr_enums)) = (
        max_attribute_definitions,
        max_attribute_values,
        max_attribute_enum_values,
    ) {
        limits_content.push_str(&format!(
            r#"#[allow(dead_code)]
pub const MAX_ATTRIBUTE_DEFINITIONS: usize = {};
#[allow(dead_code)]
pub const MAX_ATTRIBUTE_VALUES: usize = {};
#[allow(dead_code)]
pub const MAX_ATTRIBUTE_ENUM_VALUES: usize = {};
"#,
            attr_defs, attr_vals, attr_enums
        ));
    }

    std::fs::write(&dest_path, limits_content).unwrap();

    // Rebuild if the environment variables change
    println!("cargo:rerun-if-env-changed=DBC_MAX_SIGNALS_PER_MESSAGE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_MESSAGES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NODES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_VALUE_DESCRIPTIONS");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NAME_SIZE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_EXTENDED_MULTIPLEXING");
    println!("cargo:rerun-if-env-changed=DBC_MAX_ATTRIBUTE_DEFINITIONS");
    println!("cargo:rerun-if-env-changed=DBC_MAX_ATTRIBUTE_VALUES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_ATTRIBUTE_ENUM_VALUES");
}
