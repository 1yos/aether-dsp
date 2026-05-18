//! JSON Schema validation for presets and configurations.
//!
//! Provides schema definitions and validation for:
//! - Preset files
//! - Node configurations
//! - Graph definitions
//! - Parameter mappings
//!
//! # Example
//!
//! ```rust
//! use aetherdsp_core::schema::{SchemaValidator, ValidationError};
//!
//! let validator = SchemaValidator::new();
//!
//! let preset_json = r#"{
//!     "name": "My Preset",
//!     "nodes": [],
//!     "connections": []
//! }"#;
//!
//! match validator.validate_preset(preset_json) {
//!     Ok(_) => println!("Valid preset"),
//!     Err(errors) => {
//!         for error in errors {
//!             println!("Validation error: {}", error);
//!         }
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// JSON path where error occurred (e.g., "/nodes/0/params/gain").
    pub path: String,
    
    /// Error message.
    pub message: String,
    
    /// Error kind.
    pub kind: ValidationErrorKind,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(path: impl Into<String>, message: impl Into<String>, kind: ValidationErrorKind) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            kind,
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.path, self.message, self.kind)
    }
}

impl std::error::Error for ValidationError {}

/// Validation error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// Missing required field.
    MissingField,
    /// Invalid type.
    InvalidType,
    /// Value out of range.
    OutOfRange,
    /// Invalid format.
    InvalidFormat,
    /// Duplicate value.
    Duplicate,
    /// Invalid reference.
    InvalidReference,
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField => write!(f, "missing field"),
            Self::InvalidType => write!(f, "invalid type"),
            Self::OutOfRange => write!(f, "out of range"),
            Self::InvalidFormat => write!(f, "invalid format"),
            Self::Duplicate => write!(f, "duplicate"),
            Self::InvalidReference => write!(f, "invalid reference"),
        }
    }
}

/// Schema validator.
pub struct SchemaValidator {
    /// Registered node types and their parameter schemas.
    node_schemas: HashMap<String, NodeSchema>,
}

impl SchemaValidator {
    /// Creates a new schema validator.
    pub fn new() -> Self {
        Self {
            node_schemas: HashMap::new(),
        }
    }

    /// Registers a node schema.
    pub fn register_node_schema(&mut self, node_type: impl Into<String>, schema: NodeSchema) {
        self.node_schemas.insert(node_type.into(), schema);
    }

    /// Validates a preset JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - Preset JSON string
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(errors)` if invalid.
    pub fn validate_preset(&self, json: &str) -> Result<(), Vec<ValidationError>> {
        let value: Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(e) => {
                return Err(vec![ValidationError::new(
                    "/",
                    format!("Invalid JSON: {}", e),
                    ValidationErrorKind::InvalidFormat,
                )]);
            }
        };

        let mut errors = Vec::new();

        // Validate root object
        if !value.is_object() {
            errors.push(ValidationError::new(
                "/",
                "Root must be an object",
                ValidationErrorKind::InvalidType,
            ));
            return Err(errors);
        }

        let obj = value.as_object().unwrap();

        // Validate required fields
        if !obj.contains_key("name") {
            errors.push(ValidationError::new(
                "/name",
                "Missing required field 'name'",
                ValidationErrorKind::MissingField,
            ));
        }

        if !obj.contains_key("nodes") {
            errors.push(ValidationError::new(
                "/nodes",
                "Missing required field 'nodes'",
                ValidationErrorKind::MissingField,
            ));
        }

        if !obj.contains_key("connections") {
            errors.push(ValidationError::new(
                "/connections",
                "Missing required field 'connections'",
                ValidationErrorKind::MissingField,
            ));
        }

        // Validate nodes array
        if let Some(nodes) = obj.get("nodes") {
            if !nodes.is_array() {
                errors.push(ValidationError::new(
                    "/nodes",
                    "Field 'nodes' must be an array",
                    ValidationErrorKind::InvalidType,
                ));
            } else {
                let nodes_array = nodes.as_array().unwrap();
                let mut node_ids = std::collections::HashSet::new();

                for (i, node) in nodes_array.iter().enumerate() {
                    let path = format!("/nodes/{}", i);
                    
                    if !node.is_object() {
                        errors.push(ValidationError::new(
                            &path,
                            "Node must be an object",
                            ValidationErrorKind::InvalidType,
                        ));
                        continue;
                    }

                    let node_obj = node.as_object().unwrap();

                    // Validate node ID
                    if let Some(id) = node_obj.get("id") {
                        if let Some(id_num) = id.as_u64() {
                            if !node_ids.insert(id_num) {
                                errors.push(ValidationError::new(
                                    format!("{}/id", path),
                                    format!("Duplicate node ID: {}", id_num),
                                    ValidationErrorKind::Duplicate,
                                ));
                            }
                        } else {
                            errors.push(ValidationError::new(
                                format!("{}/id", path),
                                "Node ID must be a number",
                                ValidationErrorKind::InvalidType,
                            ));
                        }
                    } else {
                        errors.push(ValidationError::new(
                            format!("{}/id", path),
                            "Missing required field 'id'",
                            ValidationErrorKind::MissingField,
                        ));
                    }

                    // Validate node type
                    if !node_obj.contains_key("node_type") {
                        errors.push(ValidationError::new(
                            format!("{}/node_type", path),
                            "Missing required field 'node_type'",
                            ValidationErrorKind::MissingField,
                        ));
                    }
                }
            }
        }

        // Validate connections array
        if let Some(connections) = obj.get("connections") {
            if !connections.is_array() {
                errors.push(ValidationError::new(
                    "/connections",
                    "Field 'connections' must be an array",
                    ValidationErrorKind::InvalidType,
                ));
            } else {
                let connections_array = connections.as_array().unwrap();

                for (i, conn) in connections_array.iter().enumerate() {
                    let path = format!("/connections/{}", i);
                    
                    if !conn.is_object() {
                        errors.push(ValidationError::new(
                            &path,
                            "Connection must be an object",
                            ValidationErrorKind::InvalidType,
                        ));
                        continue;
                    }

                    let conn_obj = conn.as_object().unwrap();

                    // Validate required fields
                    for field in &["from_node", "from_output", "to_node", "to_input"] {
                        if !conn_obj.contains_key(*field) {
                            errors.push(ValidationError::new(
                                format!("{}/{}", path, field),
                                format!("Missing required field '{}'", field),
                                ValidationErrorKind::MissingField,
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validates a node configuration.
    pub fn validate_node_config(&self, node_type: &str, config: &Value) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(schema) = self.node_schemas.get(node_type) {
            // Validate parameters
            if let Some(params) = config.get("params") {
                if let Some(params_obj) = params.as_object() {
                    for (param_name, param_value) in params_obj {
                        if let Some(param_schema) = schema.params.get(param_name) {
                            // Validate parameter value
                            if let Some(value) = param_value.as_f64() {
                                let value_f32 = value as f32;
                                if value_f32 < param_schema.min || value_f32 > param_schema.max {
                                    errors.push(ValidationError::new(
                                        format!("/params/{}", param_name),
                                        format!("Value {} out of range [{}, {}]", value_f32, param_schema.min, param_schema.max),
                                        ValidationErrorKind::OutOfRange,
                                    ));
                                }
                            } else {
                                errors.push(ValidationError::new(
                                    format!("/params/{}", param_name),
                                    "Parameter value must be a number",
                                    ValidationErrorKind::InvalidType,
                                ));
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Node schema definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSchema {
    /// Node type name.
    pub node_type: String,
    
    /// Parameter schemas.
    pub params: HashMap<String, ParamSchema>,
    
    /// Number of inputs.
    pub num_inputs: usize,
    
    /// Number of outputs.
    pub num_outputs: usize,
}

impl NodeSchema {
    /// Creates a new node schema.
    pub fn new(node_type: impl Into<String>, num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            node_type: node_type.into(),
            params: HashMap::new(),
            num_inputs,
            num_outputs,
        }
    }

    /// Adds a parameter schema.
    pub fn add_param(&mut self, name: impl Into<String>, schema: ParamSchema) {
        self.params.insert(name.into(), schema);
    }
}

/// Parameter schema definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSchema {
    /// Parameter name.
    pub name: String,
    
    /// Minimum value.
    pub min: f32,
    
    /// Maximum value.
    pub max: f32,
    
    /// Default value.
    pub default: f32,
    
    /// Unit (e.g., "Hz", "dB").
    pub unit: Option<String>,
}

impl ParamSchema {
    /// Creates a new parameter schema.
    pub fn new(name: impl Into<String>, min: f32, max: f32, default: f32) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            default,
            unit: None,
        }
    }

    /// Sets the unit.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_preset_valid() {
        let validator = SchemaValidator::new();
        
        let json = r#"{
            "name": "Test Preset",
            "nodes": [
                {
                    "id": 0,
                    "node_type": "Oscillator",
                    "params": {}
                }
            ],
            "connections": []
        }"#;

        let result = validator.validate_preset(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_preset_missing_name() {
        let validator = SchemaValidator::new();
        
        let json = r#"{
            "nodes": [],
            "connections": []
        }"#;

        let result = validator.validate_preset(json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ValidationErrorKind::MissingField);
        assert!(errors[0].path.contains("name"));
    }

    #[test]
    fn test_validate_preset_duplicate_node_id() {
        let validator = SchemaValidator::new();
        
        let json = r#"{
            "name": "Test",
            "nodes": [
                {"id": 0, "node_type": "Oscillator"},
                {"id": 0, "node_type": "Filter"}
            ],
            "connections": []
        }"#;

        let result = validator.validate_preset(json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.kind == ValidationErrorKind::Duplicate));
    }

    #[test]
    fn test_validate_preset_invalid_json() {
        let validator = SchemaValidator::new();
        
        let json = "{ invalid json }";

        let result = validator.validate_preset(json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ValidationErrorKind::InvalidFormat);
    }

    #[test]
    fn test_validate_node_config_param_out_of_range() {
        let mut validator = SchemaValidator::new();
        
        let mut schema = NodeSchema::new("Oscillator", 0, 1);
        schema.add_param("frequency", ParamSchema::new("frequency", 20.0, 20000.0, 440.0));
        validator.register_node_schema("Oscillator", schema);

        let config = serde_json::json!({
            "params": {
                "frequency": 30000.0
            }
        });

        let result = validator.validate_node_config("Oscillator", &config);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ValidationErrorKind::OutOfRange);
    }

    #[test]
    fn test_node_schema_builder() {
        let mut schema = NodeSchema::new("Filter", 1, 1);
        schema.add_param("cutoff", ParamSchema::new("cutoff", 20.0, 20000.0, 1000.0).with_unit("Hz"));
        schema.add_param("resonance", ParamSchema::new("resonance", 0.0, 1.0, 0.5));

        assert_eq!(schema.node_type, "Filter");
        assert_eq!(schema.num_inputs, 1);
        assert_eq!(schema.num_outputs, 1);
        assert_eq!(schema.params.len(), 2);
        assert!(schema.params.contains_key("cutoff"));
        assert!(schema.params.contains_key("resonance"));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::new(
            "/nodes/0/id",
            "Duplicate node ID: 5",
            ValidationErrorKind::Duplicate,
        );

        let display = format!("{}", error);
        assert!(display.contains("/nodes/0/id"));
        assert!(display.contains("Duplicate node ID: 5"));
        assert!(display.contains("duplicate"));
    }
}
