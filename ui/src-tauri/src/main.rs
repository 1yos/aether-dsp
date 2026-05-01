#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::Manager;

struct HostProcess(Mutex<Option<tauri::async_runtime::JoinHandle<()>>>);

#[tauri::command]
fn get_host_url() -> String {
    "ws://127.0.0.1:9001".to_string()
}

fn main() {
    tauri::Builder::default()
        .manage(HostProcess(Mutex::new(None)))
        .setup(|app| {
            let app_handle = app.handle();

            // Spawn aether-host as a sidecar process.
            // The sidecar binary is bundled at binaries/aether-host-{target}.
            // In dev mode, start it manually: cargo run -p aether-host --release
            match app.shell().sidecar("aether-host") {
                Ok(cmd) => {
                    let (mut rx, child) = cmd
                        .spawn()
                        .expect("Failed to spawn aether-host sidecar");

                    // Forward host stdout/stderr to the Tauri log
                    tauri::async_runtime::spawn(async move {
                        use tauri::api::process::CommandEvent;
                        while let Some(event) = rx.recv().await {
                            match event {
                                CommandEvent::Stdout(line) => {
                                    println!("[aether-host] {line}");
                                }
                                CommandEvent::Stderr(line) => {
                                    eprintln!("[aether-host] {line}");
                                }
                                CommandEvent::Terminated(status) => {
                                    eprintln!("[aether-host] terminated: {status:?}");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });

                    println!("aether-host sidecar started.");
                }
                Err(e) => {
                    // Dev mode — host started manually
                    eprintln!(
                        "Note: aether-host sidecar not found ({e}).\n\
                         In development, start it manually:\n\
                         cargo run -p aether-host --release"
                    );
                }
            }

            // Open devtools in debug builds
            #[cfg(debug_assertions)]
            {
                let window = app_handle.get_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_host_url])
        .run(tauri::generate_context!())
        .expect("error while running Aether Studio");
}
