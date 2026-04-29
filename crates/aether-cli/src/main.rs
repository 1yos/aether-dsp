//! aether — The AetherSDK command-line tool
//!
//! Usage:
//!   aether new <name>              Create a new project
//!   aether node <name>             Scaffold a new DSP node
//!   aether build                   Build the current project
//!   aether run                     Run the audio host
//!   aether list                    List registered node types
//!   aether schema                  Print JSON schema for all nodes
//!   aether validate                Validate aether.json
//!   aether registry list           List installed packages
//!   aether registry install <path> Install a local package

mod commands;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let result = match args[1].as_str() {
        "new"      => commands::new_project(&args[2..]),
        "node"     => commands::new_node(&args[2..]),
        "build"    => commands::build(&args[2..]),
        "run"      => commands::run(&args[2..]),
        "list"     => commands::list_nodes(&args[2..]),
        "schema"   => commands::schema(&args[2..]),
        "validate" => commands::validate(&args[2..]),
        "registry" => commands::registry(&args[2..]),
        "version"  => { println!("aether-cli v0.1.0 (AetherSDK)"); Ok(()) }
        "help" | "--help" | "-h" => { print_help(); Ok(()) }
        cmd => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("Run `aether help` for usage.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
r#"aether — AetherSDK CLI v0.1.0

USAGE:
    aether <COMMAND> [OPTIONS]

COMMANDS:
    new <name>              Create a new AetherSDK project
    node <name>             Scaffold a new DSP node in src/nodes/
    build [--plugin clap]   Build the project (optionally as a plugin)
    run                     Run the audio host with aether.json
    list                    List all registered node types
    schema                  Print JSON schema for all nodes
    validate                Validate aether.json against the registry
    registry list           List installed packages
    registry install <path> Install a local package
    version                 Print version
    help                    Print this help

EXAMPLES:
    aether new my-synth
    aether node spectral-reverb
    aether run
    aether build --plugin clap
    aether schema > nodes.json
"#
    );
}
