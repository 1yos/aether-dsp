//! Node abstraction for the DSP graph.
//!
//! Each node is a self-contained DSP unit. The `DspNode` trait is the only
//! interface the graph scheduler calls — keeping the hot path minimal.

use crate::{
    arena::NodeId, buffer_pool::BufferId, param::ParamBlock, state::StateBlob, BUFFER_SIZE,
    MAX_INPUTS,
};

/// The core DSP processing trait.
/// Implementations MUST be real-time safe:
///   - No allocation
///   - No locks
///   - No I/O
///   - Bounded execution time
pub trait DspNode: Send {
    /// Process one buffer of audio.
    ///
    /// `inputs`: resolved input buffer slices (None = silence).
    /// `output`: the node's output buffer to fill.
    /// `params`: the node's parameter block (pre-ticked by scheduler).
    /// `sample_rate`: current sample rate in Hz.
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    );

    /// Capture internal state for continuity transfer.
    fn capture_state(&self) -> StateBlob {
        StateBlob::EMPTY
    }

    /// Restore internal state after a graph mutation.
    fn restore_state(&mut self, _state: StateBlob) {}

    /// Human-readable node type name (for serialization/UI).
    fn type_name(&self) -> &'static str;
}

/// Graph-level node record. Stored in the arena.
pub struct NodeRecord {
    /// The DSP implementation (boxed, allocated at node creation time — not in RT).
    pub processor: Box<dyn DspNode>,
    /// Input connections: each slot holds the NodeId of the upstream node.
    pub inputs: [Option<NodeId>; MAX_INPUTS],
    /// The buffer this node writes its output into.
    pub output_buffer: BufferId,
    /// Parameter block for this node.
    pub params: ParamBlock,
}

impl NodeRecord {
    pub fn new(processor: Box<dyn DspNode>, output_buffer: BufferId) -> Self {
        Self {
            processor,
            inputs: [None; MAX_INPUTS],
            output_buffer,
            params: ParamBlock::new(),
        }
    }
}
