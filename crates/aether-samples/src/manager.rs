//! Sample manager — local installation state, download, extraction.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::SampleError;
use crate::manifest::{PackStatus, SampleManifest};

#[cfg(feature = "download")]
use crate::manifest::SamplePack;
#[cfg(feature = "download")]
use crate::progress::{DownloadPhase, DownloadProgress};
#[cfg(feature = "download")]
use crate::{MANIFEST_URL, RELEASES_BASE_URL};

// ── Local state ───────────────────────────────────────────────────────────────

/// Persisted to {samples_dir}/installed.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledState {
    /// Map from pack_id → installed version string
    pub packs: HashMap<String, InstalledPack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPack {
    pub version: String,
    /// Absolute path to the extracted pack directory
    pub path: PathBuf,
    /// Instrument IDs provided by this pack
    pub instrument_ids: Vec<String>,
}

// ── SampleManager ─────────────────────────────────────────────────────────────

pub struct SampleManager {
    /// Root directory for all sample data.
    /// Platform default: %APPDATA%\AetherDSP\samples (Windows)
    ///                   ~/.local/share/aetherdsp/samples (Linux)
    ///                   ~/Library/Application Support/AetherDSP/samples (macOS)
    pub samples_dir: PathBuf,
    /// Cached manifest (fetched from GitHub Releases).
    pub manifest: Option<SampleManifest>,
    /// Local installation state.
    pub installed: InstalledState,
}

impl SampleManager {
    /// Create a new manager using the platform default data directory.
    pub fn new() -> Result<Self, SampleError> {
        let base = dirs::data_dir()
            .ok_or_else(|| SampleError::StorageUnavailable("No data directory found".into()))?;
        let samples_dir = base.join("AetherDSP").join("samples");
        std::fs::create_dir_all(&samples_dir)?;
        let installed = Self::load_installed_state(&samples_dir)?;
        Ok(Self {
            samples_dir,
            manifest: None,
            installed,
        })
    }

    /// Create with an explicit directory (used in tests).
    pub fn with_dir(samples_dir: PathBuf) -> Result<Self, SampleError> {
        std::fs::create_dir_all(&samples_dir)?;
        let installed = Self::load_installed_state(&samples_dir)?;
        Ok(Self {
            samples_dir,
            manifest: None,
            installed,
        })
    }

    // ── Manifest ──────────────────────────────────────────────────────────────

    /// Fetch the manifest from GitHub Releases and cache it.
    #[cfg(feature = "download")]
    pub fn fetch_manifest(&mut self) -> Result<&SampleManifest, SampleError> {
        let response = reqwest::blocking::get(MANIFEST_URL)
            .map_err(|e| SampleError::Network(e.to_string()))?;
        let manifest: SampleManifest = response
            .json()
            .map_err(|e| SampleError::Network(e.to_string()))?;
        self.manifest = Some(manifest);
        Ok(self.manifest.as_ref().unwrap())
    }

    /// Load a manifest from a local file (used in tests and offline mode).
    pub fn load_manifest_from_file(&mut self, path: &Path) -> Result<&SampleManifest, SampleError> {
        let json = std::fs::read_to_string(path)?;
        let manifest: SampleManifest = serde_json::from_str(&json)?;
        self.manifest = Some(manifest);
        Ok(self.manifest.as_ref().unwrap())
    }

    /// Load a manifest from a JSON string.
    pub fn load_manifest_from_str(&mut self, json: &str) -> Result<&SampleManifest, SampleError> {
        let manifest: SampleManifest = serde_json::from_str(json)?;
        self.manifest = Some(manifest);
        Ok(self.manifest.as_ref().unwrap())
    }

    // ── Status ────────────────────────────────────────────────────────────────

    /// Get the installation status of a pack.
    pub fn pack_status(&self, pack_id: &str) -> PackStatus {
        match self.installed.packs.get(pack_id) {
            None => PackStatus::NotInstalled,
            Some(installed) => {
                if let Some(manifest) = &self.manifest {
                    if let Some(pack) = manifest.find(pack_id) {
                        if pack.version != installed.version {
                            return PackStatus::UpdateAvailable {
                                installed_version: installed.version.clone(),
                                latest_version: pack.version.clone(),
                            };
                        }
                    }
                }
                PackStatus::Installed {
                    version: installed.version.clone(),
                }
            }
        }
    }

    /// Get all packs with their current status.
    pub fn all_pack_statuses(&self) -> Vec<(String, PackStatus)> {
        match &self.manifest {
            None => vec![],
            Some(manifest) => manifest
                .packs
                .iter()
                .map(|p| (p.id.clone(), self.pack_status(&p.id)))
                .collect(),
        }
    }

    /// Resolve the WAV file path for a given instrument ID and zone file path.
    /// Returns None if the instrument's pack is not installed.
    pub fn resolve_sample_path(&self, instrument_id: &str, relative_path: &str) -> Option<PathBuf> {
        // Find which pack provides this instrument
        let pack_id = self.find_pack_for_instrument(instrument_id)?;
        let installed = self.installed.packs.get(&pack_id)?;
        let full_path = installed.path.join(relative_path);
        if full_path.exists() {
            Some(full_path)
        } else {
            None
        }
    }

    /// Find the pack ID that provides a given instrument.
    pub fn find_pack_for_instrument(&self, instrument_id: &str) -> Option<String> {
        for (pack_id, installed) in &self.installed.packs {
            if installed.instrument_ids.contains(&instrument_id.to_string()) {
                return Some(pack_id.clone());
            }
        }
        None
    }

    /// Returns true if the instrument has samples installed.
    pub fn instrument_has_samples(&self, instrument_id: &str) -> bool {
        self.find_pack_for_instrument(instrument_id).is_some()
    }

    // ── Download ──────────────────────────────────────────────────────────────

    /// Download and install a sample pack.
    /// Reports progress via the `on_progress` callback.
    /// This is a blocking call — run it on a background thread.
    #[cfg(feature = "download")]
    pub fn download_pack(
        &mut self,
        pack_id: &str,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<(), SampleError> {
        let pack = self
            .manifest
            .as_ref()
            .ok_or_else(|| SampleError::PackNotFound("Manifest not loaded".into()))?
            .find(pack_id)
            .ok_or_else(|| SampleError::PackNotFound(pack_id.to_string()))?
            .clone();

        let url = format!("{}/{}", RELEASES_BASE_URL, pack.filename);
        let total = pack.compressed_bytes;

        on_progress(DownloadProgress {
            pack_id: pack_id.to_string(),
            bytes_downloaded: 0,
            bytes_total: total,
            phase: DownloadPhase::Downloading,
        });

        // Download to a temp file
        let tmp_path = self.samples_dir.join(format!("{}.tmp", pack.filename));
        {
            use std::io::Write;
            let mut response = reqwest::blocking::get(&url)
                .map_err(|e| SampleError::Network(e.to_string()))?;

            let mut file = std::fs::File::create(&tmp_path)?;
            let mut downloaded = 0u64;
            let mut buf = vec![0u8; 65536];

            loop {
                use std::io::Read;
                let n = response
                    .read(&mut buf)
                    .map_err(|e| SampleError::Network(e.to_string()))?;
                if n == 0 {
                    break;
                }
                file.write_all(&buf[..n])?;
                downloaded += n as u64;
                on_progress(DownloadProgress {
                    pack_id: pack_id.to_string(),
                    bytes_downloaded: downloaded,
                    bytes_total: total,
                    phase: DownloadPhase::Downloading,
                });
            }
        }

        // Verify checksum
        on_progress(DownloadProgress {
            pack_id: pack_id.to_string(),
            bytes_downloaded: total,
            bytes_total: total,
            phase: DownloadPhase::Verifying,
        });

        let actual_hash = sha256_file(&tmp_path)?;
        if actual_hash != pack.sha256 {
            std::fs::remove_file(&tmp_path).ok();
            return Err(SampleError::ChecksumMismatch {
                pack_id: pack_id.to_string(),
                expected: pack.sha256.clone(),
                actual: actual_hash,
            });
        }

        // Extract
        on_progress(DownloadProgress {
            pack_id: pack_id.to_string(),
            bytes_downloaded: total,
            bytes_total: total,
            phase: DownloadPhase::Extracting,
        });

        let extract_dir = self.samples_dir.clone();
        extract_tar_zst(&tmp_path, &extract_dir)
            .map_err(|e| SampleError::ExtractionFailed(e.to_string()))?;

        // Clean up temp file
        std::fs::remove_file(&tmp_path).ok();

        // Record installation
        let pack_dir = extract_dir.join(&pack.id);
        self.installed.packs.insert(
            pack_id.to_string(),
            InstalledPack {
                version: pack.version.clone(),
                path: pack_dir,
                instrument_ids: pack.instrument_ids.clone(),
            },
        );
        self.save_installed_state()?;

        on_progress(DownloadProgress {
            pack_id: pack_id.to_string(),
            bytes_downloaded: total,
            bytes_total: total,
            phase: DownloadPhase::Complete,
        });

        Ok(())
    }

    // ── Uninstall ─────────────────────────────────────────────────────────────

    /// Remove an installed pack from disk and from the installed state.
    pub fn uninstall_pack(&mut self, pack_id: &str) -> Result<(), SampleError> {
        let installed = self
            .installed
            .packs
            .remove(pack_id)
            .ok_or_else(|| SampleError::PackNotFound(pack_id.to_string()))?;

        if installed.path.exists() {
            std::fs::remove_dir_all(&installed.path)?;
        }

        self.save_installed_state()?;
        Ok(())
    }

    // ── Disk usage ────────────────────────────────────────────────────────────

    /// Total bytes used by all installed packs.
    pub fn total_disk_usage(&self) -> u64 {
        self.installed
            .packs
            .values()
            .map(|p| dir_size(&p.path).unwrap_or(0))
            .sum()
    }

    // ── Persistence ──────────────────────────────────────────────────────────

    fn load_installed_state(samples_dir: &Path) -> Result<InstalledState, SampleError> {
        let path = samples_dir.join("installed.json");
        if !path.exists() {
            return Ok(InstalledState::default());
        }
        let json = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&json)?)
    }

    fn save_installed_state(&self) -> Result<(), SampleError> {
        let path = self.samples_dir.join("installed.json");
        let json = serde_json::to_string_pretty(&self.installed)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for SampleManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SampleManager")
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute SHA-256 of a file, return as lowercase hex string.
#[cfg(feature = "download")]
fn sha256_file(path: &Path) -> Result<String, SampleError> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

/// Extract a .tar.zst archive to a directory.
#[cfg(feature = "download")]
fn extract_tar_zst(archive_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use ruzstd::streaming_decoder::StreamingDecoder;
    let file = std::fs::File::open(archive_path)?;
    let decoder = StreamingDecoder::new(file)?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest_dir)?;
    Ok(())
}

/// Recursively compute directory size in bytes.
fn dir_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                total += dir_size(&entry.path())?;
            } else {
                total += meta.len();
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manager_creates_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("samples");
        let manager = SampleManager::with_dir(dir.clone()).unwrap();
        assert!(dir.exists());
        assert!(manager.installed.packs.is_empty());
    }

    #[test]
    fn test_pack_status_not_installed() {
        let tmp = TempDir::new().unwrap();
        let manager = SampleManager::with_dir(tmp.path().to_path_buf()).unwrap();
        assert_eq!(manager.pack_status("drums-studio-v1"), PackStatus::NotInstalled);
    }

    #[test]
    fn test_load_manifest_from_str() {
        let tmp = TempDir::new().unwrap();
        let mut manager = SampleManager::with_dir(tmp.path().to_path_buf()).unwrap();
        let json = r#"{
            "version": "1.0.0",
            "packs": [{
                "id": "test-pack",
                "name": "Test Pack",
                "description": "A test pack",
                "category": "drums",
                "version": "1.0.0",
                "filename": "test-pack.tar.zst",
                "sha256": "abc123",
                "compressed_bytes": 1000,
                "uncompressed_bytes": 5000,
                "instrument_ids": ["test-drum"],
                "quality": "lite",
                "full_pack_id": null,
                "license": "CC0",
                "attribution": ""
            }]
        }"#;
        manager.load_manifest_from_str(json).unwrap();
        assert_eq!(manager.manifest.as_ref().unwrap().packs.len(), 1);
        assert_eq!(manager.pack_status("test-pack"), PackStatus::NotInstalled);
    }

    #[test]
    fn test_installed_state_persistence() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_path_buf();
        {
            let mut manager = SampleManager::with_dir(dir.clone()).unwrap();
            manager.installed.packs.insert(
                "test-pack".to_string(),
                InstalledPack {
                    version: "1.0.0".to_string(),
                    path: dir.join("test-pack"),
                    instrument_ids: vec!["test-drum".to_string()],
                },
            );
            manager.save_installed_state().unwrap();
        }
        // Reload
        let manager2 = SampleManager::with_dir(dir).unwrap();
        assert!(manager2.installed.packs.contains_key("test-pack"));
        assert_eq!(
            manager2.installed.packs["test-pack"].version,
            "1.0.0"
        );
    }
}
