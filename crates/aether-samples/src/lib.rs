//! AetherDSP Sample Library Manager
//!
//! Manages on-demand download of sample packs from GitHub Releases.
//!
//! # Architecture
//!
//! - `SampleManifest` — the index of all available packs, fetched from GitHub Releases
//! - `SampleManager` — manages local installation state, downloads, and extraction
//! - `DownloadProgress` — real-time progress reporting via a callback
//!
//! # Storage layout
//!
//! ```text
//! {data_dir}/AetherDSP/samples/
//!   installed.json          — local state: which packs are installed
//!   drums-studio-v1/        — extracted pack directory
//!     manifest.json         — SamplerInstrument JSON for each instrument in the pack
//!     kick/
//!       kick-v1-p1.wav
//!       ...
//!   piano-grand-v1/
//!     ...
//! ```

pub mod manifest;
pub mod manager;
pub mod progress;
pub mod error;

pub use manifest::{SampleManifest, SamplePack, PackStatus};
pub use manager::SampleManager;
pub use progress::DownloadProgress;
pub use error::SampleError;

/// GitHub Releases URL for the sample manifest.
/// This points to the latest release's manifest asset.
pub const MANIFEST_URL: &str =
    "https://github.com/1yos/aether-dsp/releases/latest/download/samples-manifest.json";

/// Base URL for sample pack downloads.
pub const RELEASES_BASE_URL: &str =
    "https://github.com/1yos/aether-dsp/releases/latest/download";
