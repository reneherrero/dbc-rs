use std::env;

fn main() {
    // Validate language feature selection - only one language feature should be enabled
    let lang_features = [
        ("lang-en", "English"),
        ("lang-de", "German"),
        ("lang-es", "Spanish"),
        ("lang-fr", "French"),
        ("lang-ja", "Japanese"),
    ];

    let enabled_langs: Vec<_> = lang_features
        .iter()
        .filter(|(feature, _)| {
            env::var(format!(
                "CARGO_FEATURE_{}",
                feature.to_uppercase().replace('-', "_")
            ))
            .is_ok()
        })
        .collect();

    if enabled_langs.len() > 1 {
        let enabled_names: Vec<_> = enabled_langs.iter().map(|(_, name)| *name).collect();
        panic!(
            "ERROR: Multiple language features enabled: {}. Only one language feature can be enabled at a time.\n\
            Please enable only one of: lang-en, lang-de, lang-es, lang-fr, lang-ja",
            enabled_names.join(", ")
        );
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
        .unwrap_or(10_000); // Default to 1000

    // Write the constants to a file in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("limits.rs");
    std::fs::write(
               &dest_path,
               format!(
                   "#[allow(dead_code)]\npub const MAX_SIGNALS_PER_MESSAGE: usize = {};\n#[allow(dead_code)]\npub const MAX_MESSAGES: usize = {};",
                   max_signals, max_messages
               ),
           )
           .unwrap();

    // Rebuild if the environment variables change
    println!("cargo:rerun-if-env-changed=DBC_MAX_SIGNALS_PER_MESSAGE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_MESSAGES");
}
