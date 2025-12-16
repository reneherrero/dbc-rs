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
        .unwrap_or(64); // Default to 64

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

    // Validate that all values are powers of 2 when heapless feature is enabled
    // heapless::Vec, heapless::String, and heapless::FnvIndexMap require power-of-2 capacities
    if has_heapless {
        let heapless_constants = [
            ("DBC_MAX_MESSAGES", max_messages, "MAX_MESSAGES"),
            (
                "DBC_MAX_SIGNALS_PER_MESSAGE",
                max_signals,
                "MAX_SIGNALS_PER_MESSAGE",
            ),
            ("DBC_MAX_NODES", max_nodes, "MAX_NODES"),
            ("DBC_MAX_NAME_SIZE", max_name_size, "MAX_NAME_SIZE"),
        ];

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
    std::fs::write(
               &dest_path,
               format!(
                   "#[allow(dead_code)]\npub const MAX_SIGNALS_PER_MESSAGE: usize = {};\n#[allow(dead_code)]\npub const MAX_MESSAGES: usize = {};\n#[allow(dead_code)]\npub const MAX_NODES: usize = {};\n#[allow(dead_code)]\npub const MAX_VALUE_DESCRIPTIONS: usize = {};\n#[allow(dead_code)]\npub const MAX_NAME_SIZE: usize = {};",
                   max_signals, max_messages, max_nodes, max_value_descriptions, max_name_size
               ),
           )
           .unwrap();

    // Rebuild if the environment variables change
    println!("cargo:rerun-if-env-changed=DBC_MAX_SIGNALS_PER_MESSAGE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_MESSAGES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NODES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_VALUE_DESCRIPTIONS");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NAME_SIZE");
}
