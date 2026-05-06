//! Aether Studio — Native GPU UI
//!
//! Built with gpui (the UI framework from Zed editor).
//! Renders directly to Metal / Vulkan / DirectX — no browser engine.
//!
//! Architecture:
//!   - Audio engine runs on a dedicated CPAL thread (same process)
//!   - UI state lives in `AppState` (Arc<Mutex<>>)
//!   - Engine → UI: ring buffer of events, polled every frame
//!   - UI → Engine: direct function calls (no WebSocket)
//!
//! Views:
//!   - SongView    — timeline / arrange (F1)
//!   - PianoRoll   — MIDI note editor (F2)
//!   - MixerView   — channel strips (F3)
//!   - PatcherView — node graph (F4)
//!   - PerformView — live mixer (F5)

mod app_state;
mod engine;
mod theme;
mod views;
mod widgets;

use gpui::*;
use app_state::AppState;
use views::daw_shell::DawShell;

fn main() {
    tracing_subscriber::fmt::init();

    // Start the audio engine in the background
    let state = AppState::new();
    engine::start(state.clone());

    // Launch the GPU UI
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.0), px(0.0)),
                    size: size(px(1440.0), px(900.0)),
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Aether Studio".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(12.0), px(12.0))),
                }),
                kind: WindowKind::Normal,
                is_movable: true,
                display_id: None,
                window_min_size: Some(size(px(1024.0), px(600.0))),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |cx| cx.new_view(|cx| DawShell::new(state, cx)),
        )
        .unwrap();
    });
}
