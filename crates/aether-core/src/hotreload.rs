//! Hot reload support for DSP nodes.
//!
//! Allows reloading node implementations without restarting the audio engine.
//! Useful for rapid development and live coding.
//!
//! # Architecture
//!
//! Hot reload works by:
//! 1. Watching for file changes in node source files
//! 2. Recompiling the changed node
//! 3. Loading the new implementation via dynamic library
//! 4. Swapping the old node with the new one while preserving state
//!
//! # Limitations
//!
//! - Requires nodes to be compiled as dynamic libraries (.dll/.so/.dylib)
//! - State preservation is best-effort (depends on state structure compatibility)
//! - Not suitable for production use (development only)
//! - Requires Rust toolchain to be installed

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Hot reload configuration.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Directory to watch for changes.
    pub watch_dir: PathBuf,

    /// File extensions to watch (e.g., ".rs").
    pub watch_extensions: Vec<String>,

    /// Debounce time in milliseconds (wait this long after last change before reloading).
    pub debounce_ms: u64,

    /// Whether to preserve node state across reloads.
    pub preserve_state: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watch_dir: PathBuf::from("crates/aether-nodes/src"),
            watch_extensions: vec![".rs".to_string()],
            debounce_ms: 500,
            preserve_state: true,
        }
    }
}

/// Hot reload manager.
///
/// Watches for file changes and triggers recompilation/reload.
pub struct HotReloadManager {
    config: HotReloadConfig,
    last_modified: std::collections::HashMap<PathBuf, SystemTime>,
    pending_reload: Option<SystemTime>,
}

impl HotReloadManager {
    /// Creates a new hot reload manager.
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            last_modified: std::collections::HashMap::new(),
            pending_reload: None,
        }
    }

    /// Checks for file changes and returns true if a reload is needed.
    ///
    /// Call this periodically (e.g., every 100ms) from a background thread.
    ///
    /// # Returns
    ///
    /// `true` if files have changed and debounce period has elapsed.
    pub fn check_for_changes(&mut self) -> bool {
        let mut changed = false;

        // Scan watch directory for changes
        if let Ok(entries) = std::fs::read_dir(&self.config.watch_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();

                        // Check if file extension matches
                        if let Some(ext) = path.extension() {
                            let ext_str = format!(".{}", ext.to_string_lossy());
                            if !self.config.watch_extensions.contains(&ext_str) {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // Check if file was modified
                        if let Ok(modified) = metadata.modified() {
                            if let Some(&last_mod) = self.last_modified.get(&path) {
                                if modified > last_mod {
                                    changed = true;
                                    self.last_modified.insert(path.clone(), modified);
                                }
                            } else {
                                // First time seeing this file
                                self.last_modified.insert(path, modified);
                            }
                        }
                    }
                }
            }
        }

        // If changes detected, start debounce timer
        if changed {
            self.pending_reload = Some(SystemTime::now());
            return false;
        }

        // Check if debounce period has elapsed
        if let Some(pending_time) = self.pending_reload {
            if let Ok(elapsed) = pending_time.elapsed() {
                if elapsed.as_millis() >= self.config.debounce_ms as u128 {
                    self.pending_reload = None;
                    return true;
                }
            }
        }

        false
    }

    /// Triggers a reload of the specified node.
    ///
    /// This is a placeholder - actual implementation would:
    /// 1. Run `cargo build --release -p aetherdsp-nodes`
    /// 2. Load the new .dll/.so/.dylib
    /// 3. Extract the node factory function
    /// 4. Create new node instance
    /// 5. Transfer state from old node to new node
    /// 6. Swap nodes in the graph
    ///
    /// # Arguments
    ///
    /// * `node_name` - Name of the node to reload (e.g., "Oscillator")
    ///
    /// # Returns
    ///
    /// `Ok(())` if reload succeeded, `Err` otherwise.
    pub fn reload_node(&self, node_name: &str) -> Result<(), String> {
        // Placeholder implementation
        println!("Hot reload: Recompiling {}...", node_name);

        // In a real implementation, this would:
        // 1. Run cargo build
        // 2. Load dynamic library
        // 3. Swap node implementation

        Err("Hot reload not fully implemented (placeholder)".to_string())
    }

    /// Gets the watch directory.
    pub fn watch_dir(&self) -> &Path {
        &self.config.watch_dir
    }

    /// Gets the number of files being watched.
    pub fn watched_file_count(&self) -> usize {
        self.last_modified.len()
    }
}

/// Node state snapshot for hot reload.
///
/// Captures the internal state of a node so it can be restored after reload.
#[derive(Debug, Clone)]
pub struct NodeStateSnapshot {
    /// Node type name.
    pub node_type: String,

    /// Serialized state (JSON or binary).
    pub state_data: Vec<u8>,

    /// Parameter values.
    pub param_values: Vec<f32>,
}

impl NodeStateSnapshot {
    /// Creates a new state snapshot.
    pub fn new(node_type: impl Into<String>) -> Self {
        Self {
            node_type: node_type.into(),
            state_data: Vec::new(),
            param_values: Vec::new(),
        }
    }

    /// Adds a parameter value to the snapshot.
    pub fn add_param(&mut self, value: f32) {
        self.param_values.push(value);
    }

    /// Sets the state data.
    pub fn set_state_data(&mut self, data: Vec<u8>) {
        self.state_data = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotreload_config_default() {
        let config = HotReloadConfig::default();
        assert_eq!(config.debounce_ms, 500);
        assert!(config.preserve_state);
        assert_eq!(config.watch_extensions.len(), 1);
    }

    #[test]
    fn test_hotreload_manager_creation() {
        let config = HotReloadConfig::default();
        let manager = HotReloadManager::new(config);
        assert_eq!(manager.watched_file_count(), 0);
    }

    #[test]
    fn test_node_state_snapshot() {
        let mut snapshot = NodeStateSnapshot::new("Oscillator");
        snapshot.add_param(440.0);
        snapshot.add_param(0.5);
        snapshot.set_state_data(vec![1, 2, 3, 4]);

        assert_eq!(snapshot.node_type, "Oscillator");
        assert_eq!(snapshot.param_values.len(), 2);
        assert_eq!(snapshot.state_data.len(), 4);
    }

    #[test]
    fn test_hotreload_reload_node_placeholder() {
        let config = HotReloadConfig::default();
        let manager = HotReloadManager::new(config);

        // Should return error (placeholder implementation)
        let result = manager.reload_node("Oscillator");
        assert!(result.is_err());
    }
}
