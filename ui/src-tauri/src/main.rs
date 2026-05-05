#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

// ── Sample library state ──────────────────────────────────────────────────────

struct SampleState {
    manager: Arc<Mutex<aether_samples::SampleManager>>,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn get_host_url() -> String {
    "ws://127.0.0.1:9001".to_string()
}

/// Fetch the sample manifest from GitHub Releases and return it as JSON.
#[tauri::command]
fn fetch_sample_manifest(state: State<SampleState>) -> Result<String, String> {
    #[cfg(feature = "download")]
    {
        let mut manager = state.manager.lock().unwrap();
        manager.fetch_manifest().map_err(|e| e.to_string())?;
    }
    let manager = state.manager.lock().unwrap();
    let manifest = manager.manifest.as_ref()
        .ok_or_else(|| "Manifest not loaded".to_string())?;
    serde_json::to_string(manifest).map_err(|e| e.to_string())
}

/// Get the installation status of all known packs.
#[tauri::command]
fn get_pack_statuses(state: State<SampleState>) -> Result<String, String> {
    let manager = state.manager.lock().unwrap();
    let statuses = manager.all_pack_statuses();
    serde_json::to_string(&statuses).map_err(|e| e.to_string())
}

/// Download and install a sample pack. Emits progress events to the window.
#[tauri::command]
async fn download_sample_pack(
    pack_id: String,
    window: tauri::Window,
    state: State<'_, SampleState>,
) -> Result<(), String> {
    // Clone the Arc so the closure owns its own reference — no lifetime issues.
    let manager_arc = Arc::clone(&state.manager);

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut manager = manager_arc.lock().unwrap();
        #[cfg(feature = "download")]
        {
            manager.download_pack(&pack_id, |progress| {
                let _ = window.emit("sample-download-progress", &progress);
            })
        }
        #[cfg(not(feature = "download"))]
        {
            let _ = pack_id;
            Err(aether_samples::SampleError::Network(
                "Download feature not enabled".into(),
            ))
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    result.map_err(|e| e.to_string())
}

/// Uninstall a sample pack.
#[tauri::command]
fn uninstall_sample_pack(pack_id: String, state: State<SampleState>) -> Result<(), String> {
    let mut manager = state.manager.lock().unwrap();
    manager.uninstall_pack(&pack_id).map_err(|e| e.to_string())
}

/// Get total disk usage of installed samples in bytes.
#[tauri::command]
fn get_sample_disk_usage(state: State<SampleState>) -> u64 {
    let manager = state.manager.lock().unwrap();
    manager.total_disk_usage()
}

/// Resolve the absolute path for a sample file.
/// Used by the host to load samples from the correct location.
#[tauri::command]
fn resolve_sample_path(
    instrument_id: String,
    relative_path: String,
    state: State<SampleState>,
) -> Option<String> {
    let manager = state.manager.lock().unwrap();
    manager
        .resolve_sample_path(&instrument_id, &relative_path)
        .map(|p| p.to_string_lossy().to_string())
}

/// Check if an instrument has samples installed.
#[tauri::command]
fn instrument_has_samples(instrument_id: String, state: State<SampleState>) -> bool {
    let manager = state.manager.lock().unwrap();
    manager.instrument_has_samples(&instrument_id)
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let sample_manager = aether_samples::SampleManager::new()
        .unwrap_or_else(|e| {
            eprintln!("Warning: could not initialize sample manager: {e}");
            aether_samples::SampleManager::with_dir(
                std::path::PathBuf::from("samples")
            ).expect("Failed to create fallback sample directory")
        });

    tauri::Builder::default()
        .manage(SampleState {
            manager: Arc::new(Mutex::new(sample_manager)),
        })
        .setup(|app| {
            // Spawn aether-host as a sidecar process (Tauri v1 API).
            let result = tauri::api::process::Command::new_sidecar("aether-host")
                .map(|cmd| cmd.spawn());

            match result {
                Ok(Ok((mut rx, _child))) => {
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

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_host_url,
            fetch_sample_manifest,
            get_pack_statuses,
            download_sample_pack,
            uninstall_sample_pack,
            get_sample_disk_usage,
            resolve_sample_path,
            instrument_has_samples,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Aether Studio");
}
