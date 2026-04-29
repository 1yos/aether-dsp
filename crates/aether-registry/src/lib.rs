//! Aether Package Registry
//!
//! Tracks installed node packages and resolves them for the manifest system.
//! Packages are stored as JSON index files in `~/.aether/registry/`.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Package not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A package entry in the registry index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub nodes: Vec<String>,
    pub path: Option<PathBuf>,
    pub checksum: Option<String>,
}

/// The local package registry.
pub struct PackageRegistry {
    pub root: PathBuf,
    packages: HashMap<String, PackageEntry>,
}

impl PackageRegistry {
    /// Open or create the registry at `~/.aether/registry/`.
    pub fn open() -> Result<Self, RegistryError> {
        let root = dirs_home().join(".aether").join("registry");
        std::fs::create_dir_all(&root)?;
        let mut reg = Self { root: root.clone(), packages: HashMap::new() };
        reg.load_index()?;
        Ok(reg)
    }

    /// Open a registry at a custom path (for testing).
    pub fn open_at(root: PathBuf) -> Result<Self, RegistryError> {
        std::fs::create_dir_all(&root)?;
        let mut reg = Self { root, packages: HashMap::new() };
        reg.load_index()?;
        Ok(reg)
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    fn load_index(&mut self) -> Result<(), RegistryError> {
        let path = self.index_path();
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            let entries: Vec<PackageEntry> = serde_json::from_str(&json)?;
            for e in entries {
                self.packages.insert(e.name.clone(), e);
            }
        }
        Ok(())
    }

    fn save_index(&self) -> Result<(), RegistryError> {
        let entries: Vec<&PackageEntry> = self.packages.values().collect();
        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(self.index_path(), json)?;
        Ok(())
    }

    /// Install a package from a local path.
    pub fn install_local(&mut self, entry: PackageEntry) -> Result<(), RegistryError> {
        self.packages.insert(entry.name.clone(), entry);
        self.save_index()
    }

    /// Remove a package.
    pub fn remove(&mut self, name: &str) -> Result<(), RegistryError> {
        if self.packages.remove(name).is_none() {
            return Err(RegistryError::NotFound(name.to_string()));
        }
        self.save_index()
    }

    /// Look up a package by name.
    pub fn get(&self, name: &str) -> Option<&PackageEntry> {
        self.packages.get(name)
    }

    /// List all installed packages.
    pub fn list(&self) -> Vec<&PackageEntry> {
        let mut pkgs: Vec<_> = self.packages.values().collect();
        pkgs.sort_by_key(|p| p.name.as_str());
        pkgs
    }

    pub fn len(&self) -> usize { self.packages.len() }
    pub fn is_empty(&self) -> bool { self.packages.is_empty() }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
