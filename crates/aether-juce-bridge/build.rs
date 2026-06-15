use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = PathBuf::from(&crate_dir).join("include");

    // Generate C header using cbindgen
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("AETHERDSP_JUCE_BRIDGE_H")
        .with_cpp_compat(true)  // Add extern "C" for C++ compatibility
        .with_documentation(true)
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(output_dir.join("aetherdsp_juce_bridge.h"));

    println!("cargo:rerun-if-changed=src/lib.rs");
}
