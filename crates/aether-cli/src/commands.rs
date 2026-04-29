//! CLI command implementations.

use std::path::{Path, PathBuf};
use aether_ndk::registry::builtin_registry;
use aether_ndk::schema::schema_to_json;
use aether_manifest::{Manifest, new_project_manifest};
use aether_registry::{PackageRegistry, PackageEntry};

pub type CmdResult = Result<(), Box<dyn std::error::Error>>;

// ── aether new <name> ────────────────────────────────────────────────────────

pub fn new_project(args: &[String]) -> CmdResult {
    let name = args.first().ok_or("Usage: aether new <name>")?;
    let dir = PathBuf::from(name);

    if dir.exists() {
        return Err(format!("Directory '{name}' already exists").into());
    }

    std::fs::create_dir_all(dir.join("src").join("nodes"))?;
    std::fs::create_dir_all(dir.join("presets"))?;

    // aether.json
    let manifest = new_project_manifest(name);
    std::fs::write(dir.join("aether.json"), manifest.to_json())?;

    // Cargo.toml for the project
    std::fs::write(dir.join("Cargo.toml"), format!(
r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
aether-ndk = {{ path = "../aether-dsp/crates/aether-ndk" }}
"#
    ))?;

    // src/lib.rs
    std::fs::write(dir.join("src").join("lib.rs"), format!(
r#"//! {name} — AetherSDK project
//!
//! Add your custom DSP nodes in src/nodes/.
//! Edit aether.json to wire them together.

pub mod nodes;
"#
    ))?;

    // src/nodes/mod.rs
    std::fs::write(dir.join("src").join("nodes").join("mod.rs"),
        "// Add your node modules here\n"
    )?;

    // .gitignore
    std::fs::write(dir.join(".gitignore"), "target/\n")?;

    println!("✓ Created project '{name}'");
    println!("  {name}/aether.json     — project manifest");
    println!("  {name}/src/nodes/      — add your DSP nodes here");
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  aether run");

    Ok(())
}

// ── aether node <name> ───────────────────────────────────────────────────────

pub fn new_node(args: &[String]) -> CmdResult {
    let name = args.first().ok_or("Usage: aether node <name>")?;
    let snake = to_snake_case(name);
    let pascal = to_pascal_case(name);

    let nodes_dir = PathBuf::from("src").join("nodes");
    std::fs::create_dir_all(&nodes_dir)?;

    let file = nodes_dir.join(format!("{snake}.rs"));
    if file.exists() {
        return Err(format!("File '{}' already exists", file.display()).into());
    }

    std::fs::write(&file, format!(
r#"//! {pascal} — custom AetherSDK DSP node

use aether_ndk::prelude::*;

#[aether_node]
pub struct {pascal} {{
    #[param(name = "Param1", min = 0.0, max = 1.0, default = 0.5)]
    param1: f32,
    // Add more #[param] fields or internal state fields here
}}

impl DspProcess for {pascal} {{
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {{
        let input = inputs.get(0);
        let p = params.get(0).current;
        for (i, out) in output.iter_mut().enumerate() {{
            *out = input[i] * p;
            params.tick_all();
        }}
    }}
}}
"#
    ))?;

    println!("✓ Created node '{pascal}' at {}", file.display());
    println!("\nRegister it in your project:");
    println!("  register_node!(registry, {pascal});");

    Ok(())
}

// ── aether build ─────────────────────────────────────────────────────────────

pub fn build(args: &[String]) -> CmdResult {
    let plugin = args.iter().any(|a| a == "--plugin");
    let target = args.iter().position(|a| a == "--plugin")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("clap");

    println!("Building project...");

    let status = if plugin {
        println!("  Target: {target} plugin");
        std::process::Command::new("cargo")
            .args(["build", "--release", "-p", "aether-plugin"])
            .status()?
    } else {
        std::process::Command::new("cargo")
            .args(["build", "--release"])
            .status()?
    };

    if status.success() {
        println!("✓ Build complete");
    } else {
        return Err("Build failed".into());
    }
    Ok(())
}

// ── aether run ───────────────────────────────────────────────────────────────

pub fn run(_args: &[String]) -> CmdResult {
    let manifest_path = PathBuf::from("aether.json");
    if !manifest_path.exists() {
        return Err("No aether.json found. Run `aether new <name>` first.".into());
    }

    let manifest = Manifest::from_file(&manifest_path)?;
    let registry = builtin_registry();
    manifest.validate(&registry)?;

    println!("✓ Manifest valid: {} nodes, {} connections",
        manifest.nodes.len(), manifest.connections.len());
    println!("  Sample rate: {} Hz", manifest.sample_rate);
    println!("  Block size:  {} samples", manifest.block_size);
    println!("\nStarting audio host...");

    let status = std::process::Command::new("cargo")
        .args(["run", "--release", "-p", "aether-host"])
        .status()?;

    if !status.success() {
        return Err("Host exited with error".into());
    }
    Ok(())
}

// ── aether list ──────────────────────────────────────────────────────────────

pub fn list_nodes(_args: &[String]) -> CmdResult {
    let registry = builtin_registry();
    println!("Registered node types ({}):\n", registry.len());
    for name in registry.list() {
        let defs = registry.param_defs(name).unwrap_or(&[]);
        println!("  {name}  ({} params)", defs.len());
        for d in defs {
            println!("    • {}  [{:.1} – {:.1}]  default: {:.2}",
                d.name, d.min, d.max, d.default);
        }
        println!();
    }
    Ok(())
}

// ── aether schema ────────────────────────────────────────────────────────────

pub fn schema(_args: &[String]) -> CmdResult {
    let registry = builtin_registry();
    println!("{}", schema_to_json(&registry));
    Ok(())
}

// ── aether validate ──────────────────────────────────────────────────────────

pub fn validate(_args: &[String]) -> CmdResult {
    let path = PathBuf::from("aether.json");
    if !path.exists() {
        return Err("No aether.json found.".into());
    }
    let manifest = Manifest::from_file(&path)?;
    let registry = builtin_registry();
    manifest.validate(&registry)?;
    println!("✓ aether.json is valid");
    println!("  Project: {} v{}", manifest.name, manifest.version);
    println!("  Nodes:   {}", manifest.nodes.len());
    println!("  Edges:   {}", manifest.connections.len());
    Ok(())
}

// ── aether registry ──────────────────────────────────────────────────────────

pub fn registry(args: &[String]) -> CmdResult {
    match args.first().map(|s| s.as_str()) {
        Some("list") => registry_list(),
        Some("install") => {
            let path = args.get(1).ok_or("Usage: aether registry install <path>")?;
            registry_install(Path::new(path))
        }
        _ => {
            eprintln!("Usage: aether registry <list|install>");
            Ok(())
        }
    }
}

fn registry_list() -> CmdResult {
    let reg = PackageRegistry::open()?;
    if reg.is_empty() {
        println!("No packages installed.");
        println!("Install one with: aether registry install <path>");
        return Ok(());
    }
    println!("Installed packages ({}):\n", reg.len());
    for pkg in reg.list() {
        println!("  {} v{}  — {}", pkg.name, pkg.version, pkg.description);
        for node in &pkg.nodes {
            println!("    • {node}");
        }
    }
    Ok(())
}

fn registry_install(path: &Path) -> CmdResult {
    // Read package manifest from path/package.json
    let pkg_json = path.join("package.json");
    if !pkg_json.exists() {
        return Err(format!("No package.json found at {}", path.display()).into());
    }
    let json = std::fs::read_to_string(&pkg_json)?;
    let mut entry: PackageEntry = serde_json::from_str(&json)?;
    entry.path = Some(path.to_path_buf());

    let mut reg = PackageRegistry::open()?;
    let name = entry.name.clone();
    reg.install_local(entry)?;
    println!("✓ Installed package '{name}'");
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 { out.push('_'); }
        out.push(c.to_lowercase().next().unwrap());
    }
    out.replace('-', "_")
}

fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
