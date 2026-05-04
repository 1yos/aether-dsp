//! Sample manifest — the index of all available packs hosted on GitHub Releases.

use serde::{Deserialize, Serialize};

/// The top-level manifest fetched from GitHub Releases.
/// URL: https://github.com/1yos/aether-dsp/releases/latest/download/samples-manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleManifest {
    /// Manifest schema version.
    pub version: String,
    /// All available sample packs.
    pub packs: Vec<SamplePack>,
}

/// A single sample pack available for download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePack {
    /// Unique identifier, e.g. "drums-studio-v1"
    pub id: String,
    /// Human-readable name, e.g. "Studio Drum Kit"
    pub name: String,
    /// Short description shown in the UI
    pub description: String,
    /// Category for grouping in the UI
    pub category: PackCategory,
    /// Version string, e.g. "1.0.0"
    pub version: String,
    /// Filename of the .tar.zst archive on GitHub Releases
    pub filename: String,
    /// SHA-256 checksum of the .tar.zst archive (hex string)
    pub sha256: String,
    /// Compressed size in bytes
    pub compressed_bytes: u64,
    /// Uncompressed size in bytes
    pub uncompressed_bytes: u64,
    /// Instrument IDs covered by this pack (matches CatalogInstrument.id)
    pub instrument_ids: Vec<String>,
    /// Whether this is a lite (single velocity) or full (multi-velocity) pack
    pub quality: PackQuality,
    /// Optional: ID of the full-quality version of this pack (if this is lite)
    pub full_pack_id: Option<String>,
    /// License of the samples in this pack
    pub license: String,
    /// Attribution required by the license
    pub attribution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PackCategory {
    Drums,
    Piano,
    Strings,
    Brass,
    Woodwinds,
    Guitar,
    Bass,
    Choir,
    WorldInstruments,
    ImpulseResponses,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PackQuality {
    /// Single velocity layer, no round-robin. Bundled with the app.
    Lite,
    /// 3 velocity layers, 3 round-robin variations. Downloaded on demand.
    Full,
}

/// Installation status of a pack — computed at runtime by comparing
/// the manifest against the local installed.json.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PackStatus {
    /// Not installed.
    NotInstalled,
    /// Currently downloading.
    Downloading { progress_pct: u8 },
    /// Installed and up to date.
    Installed { version: String },
    /// Installed but a newer version is available.
    UpdateAvailable { installed_version: String, latest_version: String },
}

impl SampleManifest {
    /// Find a pack by ID.
    pub fn find(&self, id: &str) -> Option<&SamplePack> {
        self.packs.iter().find(|p| p.id == id)
    }

    /// All packs in a given category.
    pub fn by_category(&self, category: &PackCategory) -> Vec<&SamplePack> {
        self.packs.iter().filter(|p| &p.category == category).collect()
    }
}
