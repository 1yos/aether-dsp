#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Try to spawn aether-host as a sidecar.
            // In dev mode the host is usually started manually, so we log
            // a warning instead of panicking if the sidecar binary is missing.
            match app.shell().sidecar("aether-host") {
                Ok(cmd) => {
                    match cmd.spawn() {
                        Ok(_) => {
                            println!("aether-host sidecar started.");
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: could not start aether-host sidecar: {e}\n\
                                 Start aether-host manually before using the studio."
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: sidecar 'aether-host' not found: {e}\n\
                         Start aether-host manually before using the studio."
                    );
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
