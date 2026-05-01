#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

#[tauri::command]
fn get_host_url() -> String {
    "ws://127.0.0.1:9001".to_string()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Spawn aether-host as a sidecar process (Tauri v1 API).
            // The sidecar binary must be at:
            //   src-tauri/binaries/aether-host-{target-triple}[.exe]
            //
            // In development, start aether-host manually:
            //   cargo run -p aether-host --release
            let result = tauri::api::process::Command::new_sidecar("aether-host")
                .map(|cmd| cmd.spawn());

            match result {
                Ok(Ok((mut rx, _child))) => {
                    // Forward host stdout/stderr to the console
                    tauri::async_runtime::spawn(async move {
                        use tauri::api::process::CommandEvent;
                        while let Some(event) = rx.recv().await {
                            match event {
                                CommandEvent::Stdout(line) => println!("[host] {line}"),
                                CommandEvent::Stderr(line) => eprintln!("[host] {line}"),
                                CommandEvent::Terminated(s) => {
                                    eprintln!("[host] terminated: {s:?}");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });
                    println!("aether-host sidecar started.");
                }
                Ok(Err(e)) => {
                    eprintln!("Note: could not start aether-host sidecar: {e}");
                    eprintln!("Start it manually: cargo run -p aether-host --release");
                }
                Err(e) => {
                    eprintln!("Note: aether-host sidecar not found: {e}");
                    eprintln!("Start it manually: cargo run -p aether-host --release");
                }
            }

            // Open devtools in debug builds
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_host_url])
        .run(tauri::generate_context!())
        .expect("error while running Aether Studio");
}
