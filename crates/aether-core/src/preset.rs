//! Preset system for saving and loading DSP graph configurations.
//!
//! Presets store complete graph state including nodes, connections, and parameter values.
//! They can be serialized to JSON for storage and sharing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete preset containing graph structure and parameter values.
///
/// # Example
///
/// ```
/// use aether_core::preset::Preset;
///
/// let mut preset = Preset::new("My Synth", "A simple synthesizer");
///
/// // Add nodes
/// preset.add_node(0, "Oscillator");
/// preset.add_node(1, "Filter");
/// preset.add_node(2, "Gain");
///
/// // Add connections
/// preset.add_connection(0, 1, 0); // Osc -> Filter
/// preset.add_connection(1, 2, 0); // Filter -> Gain
///
/// // Set parameters
/// preset.set_param(0, 0, 440.0); // Oscillator frequency
/// preset.set_param(1, 0, 1000.0); // Filter cutoff
/// preset.set_param(2, 0, 0.75); // Gain level
///
/// // Serialize to JSON
/// let json = preset.to_json().unwrap();
/// println!("{}", json);
///
/// // Deserialize from JSON
/// let loaded = Preset::from_json(&json).unwrap();
/// assert_eq!(loaded.name, "My Synth");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Preset name
    pub name: String,
    
    /// Preset description
    pub description: String,
    
    /// Author name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Nodes in the graph
    pub nodes: Vec<NodeConfig>,
    
    /// Connections between nodes
    pub connections: Vec<Connection>,
    
    /// Metadata (BPM, key, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Configuration for a single node in the preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node ID (unique within preset)
    pub id: usize,
    
    /// Node type name (e.g., "Oscillator", "Filter")
    pub node_type: String,
    
    /// Parameter values
    #[serde(default)]
    pub params: Vec<f32>,
    
    /// UI position (optional, for visual editors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<(f32, f32)>,
}

/// Connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Source node ID
    pub src_id: usize,
    
    /// Destination node ID
    pub dst_id: usize,
    
    /// Input slot on destination node
    pub slot: usize,
}

impl Preset {
    /// Creates a new empty preset.
    ///
    /// # Arguments
    ///
    /// * `name` - Preset name
    /// * `description` - Preset description
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let preset = Preset::new("Bass Synth", "Deep bass synthesizer");
    /// assert_eq!(preset.name, "Bass Synth");
    /// assert_eq!(preset.nodes.len(), 0);
    /// ```
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            author: None,
            tags: Vec::new(),
            nodes: Vec::new(),
            connections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a node to the preset.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID (unique within preset)
    /// * `node_type` - Node type name
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Test", "Test preset");
    /// preset.add_node(0, "Oscillator");
    /// preset.add_node(1, "Filter");
    /// assert_eq!(preset.nodes.len(), 2);
    /// ```
    pub fn add_node(&mut self, id: usize, node_type: impl Into<String>) {
        self.nodes.push(NodeConfig {
            id,
            node_type: node_type.into(),
            params: Vec::new(),
            position: None,
        });
    }

    /// Adds a connection between two nodes.
    ///
    /// # Arguments
    ///
    /// * `src_id` - Source node ID
    /// * `dst_id` - Destination node ID
    /// * `slot` - Input slot on destination node
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Test", "Test preset");
    /// preset.add_node(0, "Oscillator");
    /// preset.add_node(1, "Filter");
    /// preset.add_connection(0, 1, 0); // Osc -> Filter input 0
    /// assert_eq!(preset.connections.len(), 1);
    /// ```
    pub fn add_connection(&mut self, src_id: usize, dst_id: usize, slot: usize) {
        self.connections.push(Connection { src_id, dst_id, slot });
    }

    /// Sets a parameter value for a node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID
    /// * `param_index` - Parameter index
    /// * `value` - Parameter value
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Test", "Test preset");
    /// preset.add_node(0, "Oscillator");
    /// preset.set_param(0, 0, 440.0); // Set frequency to 440 Hz
    /// preset.set_param(0, 1, 0.5);   // Set amplitude to 0.5
    /// ```
    pub fn set_param(&mut self, node_id: usize, param_index: usize, value: f32) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            // Extend params vec if needed
            if param_index >= node.params.len() {
                node.params.resize(param_index + 1, 0.0);
            }
            node.params[param_index] = value;
        }
    }

    /// Sets the UI position for a node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    pub fn set_position(&mut self, node_id: usize, x: f32, y: f32) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.position = Some((x, y));
        }
    }

    /// Adds a tag to the preset.
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag to add
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Bass", "Deep bass");
    /// preset.add_tag("bass");
    /// preset.add_tag("synth");
    /// preset.add_tag("electronic");
    /// assert_eq!(preset.tags.len(), 3);
    /// ```
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Sets metadata key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Song", "Song preset");
    /// preset.set_metadata("bpm", "120");
    /// preset.set_metadata("key", "C minor");
    /// ```
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Serializes the preset to JSON.
    ///
    /// # Returns
    ///
    /// JSON string representation of the preset.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Test", "Test preset");
    /// preset.add_node(0, "Oscillator");
    /// let json = preset.to_json().unwrap();
    /// assert!(json.contains("\"name\":\"Test\""));
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a preset from JSON.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string
    ///
    /// # Returns
    ///
    /// Deserialized preset.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let json = r#"{
    ///   "name": "Test",
    ///   "description": "Test preset",
    ///   "nodes": [],
    ///   "connections": []
    /// }"#;
    ///
    /// let preset = Preset::from_json(json).unwrap();
    /// assert_eq!(preset.name, "Test");
    /// ```
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validates the preset structure.
    ///
    /// Checks for:
    /// - Duplicate node IDs
    /// - Invalid connections (referencing non-existent nodes)
    /// - Empty preset name
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with error message if invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::preset::Preset;
    ///
    /// let mut preset = Preset::new("Test", "Test preset");
    /// preset.add_node(0, "Oscillator");
    /// preset.add_node(1, "Filter");
    /// preset.add_connection(0, 1, 0);
    /// assert!(preset.validate().is_ok());
    ///
    /// // Invalid connection
    /// preset.add_connection(0, 99, 0); // Node 99 doesn't exist
    /// assert!(preset.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Check for empty name
        if self.name.is_empty() {
            return Err("Preset name cannot be empty".to_string());
        }

        // Check for duplicate node IDs
        let mut seen_ids = std::collections::HashSet::new();
        for node in &self.nodes {
            if !seen_ids.insert(node.id) {
                return Err(format!("Duplicate node ID: {}", node.id));
            }
        }

        // Check connections reference valid nodes
        for conn in &self.connections {
            if !self.nodes.iter().any(|n| n.id == conn.src_id) {
                return Err(format!("Connection references non-existent source node: {}", conn.src_id));
            }
            if !self.nodes.iter().any(|n| n.id == conn.dst_id) {
                return Err(format!("Connection references non-existent destination node: {}", conn.dst_id));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_creation() {
        let preset = Preset::new("Test Synth", "A test synthesizer");
        assert_eq!(preset.name, "Test Synth");
        assert_eq!(preset.description, "A test synthesizer");
        assert_eq!(preset.nodes.len(), 0);
        assert_eq!(preset.connections.len(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.add_node(1, "Filter");
        preset.add_node(2, "Gain");
        assert_eq!(preset.nodes.len(), 3);
        assert_eq!(preset.nodes[0].node_type, "Oscillator");
        assert_eq!(preset.nodes[1].node_type, "Filter");
    }

    #[test]
    fn test_add_connections() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.add_node(1, "Filter");
        preset.add_connection(0, 1, 0);
        assert_eq!(preset.connections.len(), 1);
        assert_eq!(preset.connections[0].src_id, 0);
        assert_eq!(preset.connections[0].dst_id, 1);
    }

    #[test]
    fn test_set_params() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.set_param(0, 0, 440.0);
        preset.set_param(0, 1, 0.5);
        assert_eq!(preset.nodes[0].params.len(), 2);
        assert_eq!(preset.nodes[0].params[0], 440.0);
        assert_eq!(preset.nodes[0].params[1], 0.5);
    }

    #[test]
    fn test_json_serialization() {
        let mut preset = Preset::new("Test", "Test preset");
        preset.add_node(0, "Oscillator");
        preset.set_param(0, 0, 440.0);
        
        let json = preset.to_json().unwrap();
        assert!(json.contains("Test")); // Check name is present
        assert!(json.contains("Oscillator")); // Check node type is present
        
        let loaded = Preset::from_json(&json).unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].params[0], 440.0);
    }

    #[test]
    fn test_validation_valid_preset() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.add_node(1, "Filter");
        preset.add_connection(0, 1, 0);
        assert!(preset.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_name() {
        let preset = Preset::new("", "Test");
        assert!(preset.validate().is_err());
    }

    #[test]
    fn test_validation_duplicate_ids() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.add_node(0, "Filter"); // Duplicate ID
        assert!(preset.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_connection() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_node(0, "Oscillator");
        preset.add_connection(0, 99, 0); // Node 99 doesn't exist
        assert!(preset.validate().is_err());
    }

    #[test]
    fn test_tags_and_metadata() {
        let mut preset = Preset::new("Test", "Test");
        preset.add_tag("bass");
        preset.add_tag("synth");
        preset.set_metadata("bpm", "120");
        preset.set_metadata("key", "C minor");
        
        assert_eq!(preset.tags.len(), 2);
        assert_eq!(preset.metadata.get("bpm"), Some(&"120".to_string()));
    }
}
