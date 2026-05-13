//! Lock-free command protocol between the control thread and the RT audio thread.
//!
//! Commands are sent via an SPSC ring buffer. The RT thread drains up to
//! MAX_COMMANDS_PER_TICK per callback, bounding mutation cost.

use crate::{arena::NodeId, param::Param};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// All mutations the control thread can request.
///
/// Commands are sent from the control thread (UI, MIDI handler, etc.) to the
/// real-time audio thread via an SPSC ring buffer. The audio thread drains
/// commands at the start of each processing block.
///
/// # Design
///
/// - **Lock-free:** Commands are sent via SPSC ring buffer (no locks)
/// - **Bounded:** Max `MAX_COMMANDS_PER_TICK` processed per audio block
/// - **Clone:** Commands are cloned when sent (cheap - no heap allocations)
/// - **Real-time safe:** All command processing is bounded and allocation-free
///
/// # Example
///
/// ```
/// use aether_core::command::Command;
/// use aether_core::arena::NodeId;
///
/// // Control thread sends commands
/// let cmd = Command::AddNode {
///     id: NodeId { index: 0, generation: 1 }
/// };
///
/// // Audio thread receives and processes
/// match cmd {
///     Command::AddNode { id } => {
///         // Add node to graph
///     }
///     _ => {}
/// }
/// ```
///
/// # Command Processing
///
/// Commands are processed in FIFO order. The audio thread processes up to
/// `MAX_COMMANDS_PER_TICK` commands per block to bound latency impact.
///
/// # See Also
///
/// * [`Scheduler::process_block`](crate::scheduler::Scheduler::process_block) - Drains commands
/// * [`DspGraph`](crate::graph::DspGraph) - Executes commands
#[derive(Debug, Clone)]
pub enum Command {
    /// Add a new node to the graph.
    ///
    /// The node must already exist in the arena. This command makes it
    /// visible to the scheduler for processing.
    ///
    /// # Fields
    ///
    /// * `id` - Node ID in the arena
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    ///
    /// let cmd = Command::AddNode {
    ///     id: NodeId { index: 0, generation: 1 }
    /// };
    /// ```
    AddNode { id: NodeId },

    /// Remove a node from the graph and release its buffer.
    ///
    /// The node is removed from the processing order and its output buffer
    /// is returned to the pool. All connections to/from this node are
    /// automatically disconnected.
    ///
    /// # Fields
    ///
    /// * `id` - Node ID to remove
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    ///
    /// let cmd = Command::RemoveNode {
    ///     id: NodeId { index: 0, generation: 1 }
    /// };
    /// ```
    RemoveNode { id: NodeId },

    /// Connect output of `src` to input slot `slot` of `dst`.
    ///
    /// Creates an audio connection between two nodes. The output of `src`
    /// will be routed to input slot `slot` of `dst`. If the slot is already
    /// connected, the old connection is replaced.
    ///
    /// # Fields
    ///
    /// * `src` - Source node ID (output)
    /// * `dst` - Destination node ID (input)
    /// * `slot` - Input slot index (0 to MAX_INPUTS-1)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    ///
    /// let osc = NodeId { index: 0, generation: 1 };
    /// let filter = NodeId { index: 1, generation: 1 };
    ///
    /// // Connect oscillator output to filter input slot 0
    /// let cmd = Command::Connect {
    ///     src: osc,
    ///     dst: filter,
    ///     slot: 0,
    /// };
    /// ```
    Connect { src: NodeId, dst: NodeId, slot: usize },

    /// Disconnect input slot `slot` of `dst`.
    ///
    /// Removes the connection to the specified input slot. The slot will
    /// receive silence (None) in subsequent processing.
    ///
    /// # Fields
    ///
    /// * `dst` - Destination node ID
    /// * `slot` - Input slot index to disconnect
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    ///
    /// let filter = NodeId { index: 1, generation: 1 };
    ///
    /// // Disconnect input slot 0
    /// let cmd = Command::Disconnect {
    ///     dst: filter,
    ///     slot: 0,
    /// };
    /// ```
    Disconnect { dst: NodeId, slot: usize },

    /// Update a parameter with a new smoothed value.
    ///
    /// Changes a node parameter with automatic smoothing to avoid clicks.
    /// The parameter will ramp from its current value to the new target
    /// over the configured smoothing time.
    ///
    /// # Fields
    ///
    /// * `node` - Node ID to update
    /// * `param_index` - Parameter index (0-based)
    /// * `new_param` - New parameter with target value and smoothing
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    /// use aether_core::param::Param;
    ///
    /// let filter = NodeId { index: 1, generation: 1 };
    ///
    /// // Update filter cutoff to 1000 Hz
    /// let cmd = Command::UpdateParam {
    ///     node: filter,
    ///     param_index: 0,
    ///     new_param: Param::new(1000.0),
    /// };
    /// ```
    UpdateParam { node: NodeId, param_index: usize, new_param: Param },

    /// Swap the output node.
    ///
    /// Designates a new node as the graph output. The output node's buffer
    /// is copied to the final output buffer each processing block.
    ///
    /// # Fields
    ///
    /// * `id` - Node ID to use as output
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    /// use aether_core::arena::NodeId;
    ///
    /// let master = NodeId { index: 5, generation: 1 };
    ///
    /// let cmd = Command::SetOutputNode { id: master };
    /// ```
    SetOutputNode { id: NodeId },

    /// Mute / unmute all audio output.
    ///
    /// When muted, the output buffer is filled with silence regardless of
    /// graph processing. Processing continues normally - only the final
    /// output is silenced.
    ///
    /// # Fields
    ///
    /// * `muted` - true to mute, false to unmute
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    ///
    /// // Mute output
    /// let cmd = Command::SetMute { muted: true };
    ///
    /// // Unmute output
    /// let cmd = Command::SetMute { muted: false };
    /// ```
    SetMute { muted: bool },

    /// Remove all nodes and edges — silence the graph.
    ///
    /// Clears the entire graph, removing all nodes and connections. All
    /// buffers are released back to the pool. The graph is left in an
    /// empty state.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::command::Command;
    ///
    /// let cmd = Command::ClearGraph;
    /// ```
    ClearGraph,
}

/// Serializable graph description for UI ↔ host communication.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub nodes: Vec<NodeSnapshot>,
    pub edges: Vec<EdgeSnapshot>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NodeSnapshot {
    pub id: u32,
    pub generation: u32,
    pub node_type: String,
    pub params: Vec<f32>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct EdgeSnapshot {
    pub src_id: u32,
    pub dst_id: u32,
    pub slot: usize,
}
