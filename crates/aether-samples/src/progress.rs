//! Download progress reporting.

use serde::{Deserialize, Serialize};

/// Real-time progress for a pack download.
/// Sent to the UI via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Pack ID being downloaded.
    pub pack_id: String,
    /// Bytes downloaded so far.
    pub bytes_downloaded: u64,
    /// Total bytes to download (compressed).
    pub bytes_total: u64,
    /// Current phase.
    pub phase: DownloadPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadPhase {
    /// Fetching the archive from GitHub Releases.
    Downloading,
    /// Verifying SHA-256 checksum.
    Verifying,
    /// Extracting .tar.zst to the samples directory.
    Extracting,
    /// Done.
    Complete,
    /// Failed with an error message.
    Failed { message: String },
}

impl DownloadProgress {
    pub fn percent(&self) -> u8 {
        if self.bytes_total == 0 {
            return 0;
        }
        ((self.bytes_downloaded as f64 / self.bytes_total as f64) * 100.0).min(100.0) as u8
    }
}
