//! Lock-free command protocol between the control thread and the RT audio thread.
//!
//! Commands are sent via an SPSC ring buffer. The RT thread drains up to
//! MAX_COMMANDS_PER_TICK per callback, bounding mutation cost.

use crate::{arena::NodeId, param::Param};
use serde::{Deserialize, Serialize};

/// All mutations the control thread can request.
#[derive(Debug, Clone)]
pub enum Command {
    /// Add a new node.
    AddNode { id: NodeId },

    /// Remove a node and release its buffer.
    RemoveNode { id: NodeId },

    /// Connect output of `src` to input slot `slot` of `dst`.
    Connect { src: NodeId, dst: NodeId, slot: usize },

    /// Disconnect input slot `slot` of `dst`.
    Disconnect { dst: NodeId, slot: usize },

    /// Update a parameter with a new smoothed value.
    UpdateParam { node: NodeId, param_index: usize, new_param: Param },

    /// Swap the output node.
    SetOutputNode { id: NodeId },

    /// Mute / unmute all audio output.
    SetMute { muted: bool },

    /// Remove all nodes and edges — silence the graph.
    ClearGraph,
}

/// Serializable graph description for UI ↔ host communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub nodes: Vec<NodeSnapshot>,
    pub edges: Vec<EdgeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub id: u32,
    pub generation: u32,
    pub node_type: String,
    pub params: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub src_id: u32,
    pub dst_id: u32,
    pub slot: usize,
}
