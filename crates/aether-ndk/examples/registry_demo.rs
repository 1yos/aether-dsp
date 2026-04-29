//! Example: Using the built-in registry and manifest system
//!
//! Run with: cargo run --example registry_demo -p aether-ndk

use aether_ndk::registry::builtin_registry;
use aether_ndk::schema::schema_to_json;

fn main() {
    let registry = builtin_registry();

    println!("=== AetherSDK Node Registry ===\n");
    println!("Registered nodes: {}\n", registry.len());

    for name in registry.list() {
        let defs = registry.param_defs(name).unwrap_or(&[]);
        println!("  {name}");
        for d in defs {
            println!("    • {:<16} [{:>7.1} – {:>7.1}]  default: {:.3}",
                d.name, d.min, d.max, d.default);
        }
    }

    println!("\n=== JSON Schema (first 20 lines) ===\n");
    let schema = schema_to_json(&registry);
    for line in schema.lines().take(20) {
        println!("{line}");
    }
    println!("...");

    // Demonstrate instantiation
    println!("\n=== Instantiation Test ===\n");
    for name in registry.list() {
        let node = registry.create(name);
        println!("  {} → {}", name, if node.is_some() { "✓" } else { "✗" });
    }
}
