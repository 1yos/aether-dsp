//! Aether project manifest — the declarative project format.
//!
//! A manifest describes a complete audio application:
//! nodes, connections, parameters, and build targets.
//!
//! ```json
//! {
//!   "name": "my-synth",
//!   "version": "0.1.0",
//!   "engine": "aether-dsp",
//!   "sample_rate": 48000,
//!   "block_size": 64,
//!   "nodes": [
//!     { "id": "osc", "type": "Oscillator", "params": { "Frequency": 440.0 } },
//!     { "id": "filt", "type": "StateVariableFilter", "params": { "Cutoff": 2000.0 } },
//!     { "id": "out", "type": "Gain", "params": { "Gain": 0.8 } }
//!   ],
//!   "connections": [
//!     { "from": "osc", "to": "filt", "slot": 0 },
//!     { "from": "filt", "to": "out", "slot": 0 }
//!   ],
//!   "output_node": "out",
//!   "plugin_targets": ["clap"]
//! }
//! ```

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unknown node type: {0}")]
    UnknownNode(String),
    #[error("Unknown node id: {0}")]
    UnknownId(String),
    #[error("Duplicate node id: {0}")]
    DuplicateId(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Top-level project manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    pub nodes: Vec<NodeDef>,
    #[serde(default)]
    pub connections: Vec<ConnectionDef>,
    pub output_node: String,
    #[serde(default)]
    pub plugin_targets: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_engine() -> String { "aether-dsp".into() }
fn default_sample_rate() -> u32 { 48_000 }
fn default_block_size() -> usize { 64 }

/// A node instance in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    /// Unique identifier within this project (e.g. "osc1").
    pub id: String,
    /// Registered type name (e.g. "Oscillator").
    #[serde(rename = "type")]
    pub node_type: String,
    /// Initial parameter values by name.
    #[serde(default)]
    pub params: HashMap<String, f32>,
}

/// A connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDef {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub slot: usize,
}

impl Manifest {
    /// Parse a manifest from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Parse a manifest from a file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ManifestError> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Validate the manifest against a node registry.
    pub fn validate(
        &self,
        registry: &aether_ndk::node::NodeRegistry,
    ) -> Result<(), ManifestError> {
        let mut ids = std::collections::HashSet::new();
        for node in &self.nodes {
            if !ids.insert(node.id.as_str()) {
                return Err(ManifestError::DuplicateId(node.id.clone()));
            }
            if registry.create(&node.node_type).is_none() {
                return Err(ManifestError::UnknownNode(node.node_type.clone()));
            }
        }
        for conn in &self.connections {
            if !ids.contains(conn.from.as_str()) {
                return Err(ManifestError::UnknownId(conn.from.clone()));
            }
            if !ids.contains(conn.to.as_str()) {
                return Err(ManifestError::UnknownId(conn.to.clone()));
            }
        }
        if !ids.contains(self.output_node.as_str()) {
            return Err(ManifestError::UnknownId(self.output_node.clone()));
        }
        Ok(())
    }

    /// Instantiate a `DspGraph` from this manifest using the given registry.
    pub fn build_graph(
        &self,
        registry: &aether_ndk::node::NodeRegistry,
        _sample_rate: f32,
    ) -> Result<aether_core::graph::DspGraph, ManifestError> {
        use aether_core::graph::DspGraph;
        use aether_core::arena::NodeId;
        use std::collections::HashMap;

        self.validate(registry)?;

        let mut graph = DspGraph::new();
        let mut id_map: HashMap<&str, NodeId> = HashMap::new();

        for node_def in &self.nodes {
            let node = registry
                .create(&node_def.node_type)
                .ok_or_else(|| ManifestError::UnknownNode(node_def.node_type.clone()))?;

            let node_id = graph.add_node(node).expect("graph full");

            // Apply param overrides
            if let Some(defs) = registry.param_defs(&node_def.node_type) {
                let record = graph.arena.get_mut(node_id).unwrap();
                for def in defs {
                    let value = node_def.params.get(def.name).copied().unwrap_or(def.default);
                    record.params.add(value);
                }
            }

            id_map.insert(&node_def.id, node_id);
        }

        for conn in &self.connections {
            let src = *id_map.get(conn.from.as_str()).unwrap();
            let dst = *id_map.get(conn.to.as_str()).unwrap();
            graph.connect(src, dst, conn.slot);
        }

        let out_id = *id_map.get(self.output_node.as_str()).unwrap();
        graph.set_output_node(out_id);

        Ok(graph)
    }
}

/// Generate a starter manifest for a new project.
pub fn new_project_manifest(name: &str) -> Manifest {
    Manifest {
        name: name.to_string(),
        version: "0.1.0".into(),
        engine: "aether-dsp".into(),
        sample_rate: 48_000,
        block_size: 64,
        nodes: vec![
            NodeDef {
                id: "osc".into(),
                node_type: "Oscillator".into(),
                params: [("Frequency".into(), 440.0)].into(),
            },
            NodeDef {
                id: "out".into(),
                node_type: "Gain".into(),
                params: [("Gain".into(), 0.8)].into(),
            },
        ],
        connections: vec![ConnectionDef {
            from: "osc".into(),
            to: "out".into(),
            slot: 0,
        }],
        output_node: "out".into(),
        plugin_targets: vec!["clap".into()],
        metadata: HashMap::new(),
    }
}
