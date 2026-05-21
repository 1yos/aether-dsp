//! Parameter utilities for node authors.

use crate::ParamDef;
use aether_core::param::ParamBlock;

/// Initialize a `ParamBlock` from a slice of `ParamDef`s.
/// Called automatically by the engine when a node is added to the graph.
pub fn init_params(block: &mut ParamBlock, defs: &[ParamDef]) {
    for def in defs {
        block.add(def.default);
    }
}

/// Clamp a param value to its defined range.
pub fn clamp_param(value: f32, def: &ParamDef) -> f32 {
    value.clamp(def.min, def.max)
}
