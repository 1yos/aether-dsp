//! JSON schema generation for registered nodes.
//! Used by the CLI and manifest system.

use serde::{Deserialize, Serialize};
use crate::{node::NodeRegistry, ParamDef};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeSchema {
    pub type_name: String,
    pub params: Vec<ParamSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamSchema {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl From<&ParamDef> for ParamSchema {
    fn from(d: &ParamDef) -> Self {
        Self {
            name: d.name.to_string(),
            min: d.min,
            max: d.max,
            default: d.default,
        }
    }
}

/// Generate JSON schema for all registered nodes.
pub fn generate_schema(registry: &NodeRegistry) -> Vec<NodeSchema> {
    registry
        .list()
        .into_iter()
        .map(|name| NodeSchema {
            type_name: name.to_string(),
            params: registry
                .param_defs(name)
                .unwrap_or(&[])
                .iter()
                .map(ParamSchema::from)
                .collect(),
        })
        .collect()
}

/// Serialize schema to pretty JSON.
pub fn schema_to_json(registry: &NodeRegistry) -> String {
    let schema = generate_schema(registry);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}
