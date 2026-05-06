//! aether-ui — Native GPU UI for Aether Studio.
//!
//! This crate is the binary entry point for the native desktop application.
//! It replaces the React/Tauri UI with a gpui-based native renderer.
//!
//! Architecture:
//!   - Audio engine: same-process CPAL thread (no WebSocket)
//!   - UI: gpui views rendered directly to Metal/Vulkan/DirectX
//!   - State: Arc<Mutex<AppState>> shared between engine and UI threads

pub mod app_state;
pub mod engine;
pub mod theme;
pub mod views;
pub mod widgets;
