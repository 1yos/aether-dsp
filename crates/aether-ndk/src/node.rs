//! Node registration and factory system.
//!
//! The `NodeRegistry` maps type name strings → constructor functions,
//! enabling the manifest system and CLI to instantiate nodes by name.

use std::collections::HashMap;
use aether_core::node::DspNode;
use crate::ParamDef;

/// A constructor that produces a boxed DspNode.
pub type NodeFactory = fn() -> Box<dyn DspNode>;

/// Metadata about a registered node type.
#[derive(Clone)]
pub struct NodeEntry {
    pub type_name: &'static str,
    pub param_defs: &'static [ParamDef],
    pub factory: NodeFactory,
}

/// Global node registry — populated at startup via `register!`.
pub struct NodeRegistry {
    entries: HashMap<&'static str, NodeEntry>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Register a node type.
    pub fn register(&mut self, entry: NodeEntry) {
        self.entries.insert(entry.type_name, entry);
    }

    /// Instantiate a node by type name. Returns `None` if not registered.
    pub fn create(&self, type_name: &str) -> Option<Box<dyn DspNode>> {
        self.entries.get(type_name).map(|e| (e.factory)())
    }

    /// List all registered node type names.
    pub fn list(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.entries.keys().copied().collect();
        names.sort_unstable();
        names
    }

    /// Get param defs for a node type.
    pub fn param_defs(&self, type_name: &str) -> Option<&[ParamDef]> {
        self.entries.get(type_name).map(|e| e.param_defs)
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

impl Default for NodeRegistry {
    fn default() -> Self { Self::new() }
}

/// Register a node type into a `NodeRegistry`.
///
/// ```rust
/// use aether_ndk::{register_node, node::NodeRegistry};
/// // register_node!(registry, MyFilter);
/// ```
#[macro_export]
macro_rules! register_node {
    ($registry:expr, $ty:ty) => {{
        use $crate::{AetherNodeMeta, DspProcess, into_node};
        $registry.register($crate::node::NodeEntry {
            type_name: <$ty as AetherNodeMeta>::type_name(),
            param_defs: <$ty as AetherNodeMeta>::param_defs(),
            factory: || {
                let node = <$ty as Default>::default();
                into_node(node)
            },
        });
    }};
}
