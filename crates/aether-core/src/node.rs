//! Node abstraction for the DSP graph.
//!
//! Each node is a self-contained DSP unit. The `DspNode` trait is the only
//! interface the graph scheduler calls — keeping the hot path minimal.

use crate::{
    arena::NodeId, buffer_pool::BufferId, param::ParamBlock, state::StateBlob, BUFFER_SIZE,
    MAX_INPUTS,
};

/// The core DSP processing trait.
///
/// Implement this trait to create custom audio processors (oscillators, filters,
/// effects, etc.). The scheduler calls `process()` once per audio block in
/// topological order.
///
/// # Real-Time Safety Requirements
///
/// Implementations **MUST** be real-time safe:
/// - ✅ No allocation (pre-allocate in `new()`)
/// - ✅ No locks (use lock-free data structures)
/// - ✅ No I/O (no file/network operations)
/// - ✅ Bounded execution time (no unbounded loops)
///
/// # Example
///
/// ```
/// use aether_core::node::DspNode;
/// use aether_core::param::ParamBlock;
/// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
///
/// /// Simple gain node
/// struct Gain {
///     gain: f32,
/// }
///
/// impl Gain {
///     fn new(gain: f32) -> Self {
///         Self { gain }
///     }
/// }
///
/// impl DspNode for Gain {
///     fn process(
///         &mut self,
///         inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
///         output: &mut [f32; BUFFER_SIZE],
///         _params: &mut ParamBlock,
///         _sample_rate: f32,
///     ) {
///         // Get first input (if connected)
///         if let Some(input) = inputs[0] {
///             // Apply gain to each sample
///             for (i, out) in output.iter_mut().enumerate() {
///                 *out = input[i] * self.gain;
///             }
///         } else {
///             // No input connected, output silence
///             output.fill(0.0);
///         }
///     }
///
///     fn type_name(&self) -> &'static str {
///         "Gain"
///     }
/// }
/// ```
///
/// # Performance Tips
///
/// - Pre-allocate buffers in `new()`, not in `process()`
/// - Use SIMD when possible (see `std::simd`)
/// - Avoid branching in inner loops
/// - Use `#[inline]` for hot functions
///
/// # See Also
///
/// * [`NodeRecord`] - Graph-level node storage
/// * [`Scheduler::process_block`](crate::scheduler::Scheduler::process_block) - Calls this trait
pub trait DspNode: Send {
    /// Process one buffer of audio.
    ///
    /// Called once per audio block by the scheduler. This is the hot path -
    /// optimize carefully and maintain real-time safety.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Array of optional input buffers. `None` means no connection (silence).
    ///   Index corresponds to input slot (0 to MAX_INPUTS-1).
    /// * `output` - Output buffer to fill with processed audio (64 samples).
    /// * `params` - Parameter block for this node (smoothed parameters).
    /// * `sample_rate` - Current sample rate in Hz (e.g., 48000.0).
    ///
    /// # Real-Time Safety
    ///
    /// This function is called from the audio thread. It **MUST**:
    /// - Complete within the buffer duration (1.33ms @ 48kHz)
    /// - Not allocate memory
    /// - Not acquire locks
    /// - Not perform I/O
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct Oscillator {
    ///     frequency: f32,
    ///     phase: f32,
    /// }
    ///
    /// impl DspNode for Oscillator {
    ///     fn process(
    ///         &mut self,
    ///         _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///         output: &mut [f32; BUFFER_SIZE],
    ///         _params: &mut ParamBlock,
    ///         sample_rate: f32,
    ///     ) {
    ///         let phase_inc = self.frequency / sample_rate;
    ///         
    ///         for sample in output.iter_mut() {
    ///             *sample = (self.phase * std::f32::consts::TAU).sin() * 0.3;
    ///             self.phase = (self.phase + phase_inc).fract();
    ///         }
    ///     }
    ///
    ///     fn type_name(&self) -> &'static str {
    ///         "Oscillator"
    ///     }
    /// }
    /// ```
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    );

    /// Capture internal state for continuity transfer.
    ///
    /// Called when the graph structure changes (add/remove nodes, reconnect).
    /// Return any internal state that should be preserved across the mutation.
    ///
    /// # Returns
    ///
    /// A `StateBlob` containing serialized state, or `StateBlob::EMPTY` if
    /// the node has no state to preserve.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::node::DspNode;
    /// use aether_core::state::StateBlob;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct Oscillator {
    ///     phase: f32,
    /// }
    ///
    /// impl DspNode for Oscillator {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         output.fill(0.0);
    ///     }
    ///
    ///     fn capture_state(&self) -> StateBlob {
    ///         // Preserve phase to avoid clicks
    ///         StateBlob::from_value(&self.phase)
    ///     }
    ///
    ///     fn restore_state(&mut self, state: StateBlob) {
    ///         if state.len == std::mem::size_of::<f32>() {
    ///             self.phase = state.to_value::<f32>();
    ///         }
    ///     }
    ///
    ///     fn type_name(&self) -> &'static str {
    ///         "Oscillator"
    ///     }
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// * [`restore_state`](Self::restore_state) - Restore captured state
    fn capture_state(&self) -> StateBlob {
        StateBlob::EMPTY
    }

    /// Restore internal state after a graph mutation.
    ///
    /// Called after the graph structure changes. Restore any state that was
    /// captured by `capture_state()` to maintain audio continuity.
    ///
    /// # Arguments
    ///
    /// * `state` - State blob from `capture_state()`
    ///
    /// # Example
    ///
    /// See [`capture_state`](Self::capture_state) for a complete example.
    fn restore_state(&mut self, _state: StateBlob) {}

    /// Human-readable node type name (for serialization/UI).
    ///
    /// Returns a static string identifying the node type. Used for:
    /// - Debugging and logging
    /// - Serialization/deserialization
    /// - UI display
    /// - Node registry lookups
    ///
    /// # Returns
    ///
    /// A static string slice with the node type name.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct Reverb;
    ///
    /// impl DspNode for Reverb {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         output.fill(0.0);
    ///     }
    ///
    ///     fn type_name(&self) -> &'static str {
    ///         "Reverb" // Used for identification
    ///     }
    /// }
    ///
    /// let reverb = Reverb;
    /// assert_eq!(reverb.type_name(), "Reverb");
    /// ```
    fn type_name(&self) -> &'static str;
}

/// Graph-level node record. Stored in the arena.
///
/// Each node in the DSP graph is represented by a `NodeRecord` stored in the
/// arena. The record contains the DSP processor, input connections, output
/// buffer, and parameters.
///
/// # Structure
///
/// - **processor:** The DSP implementation (boxed trait object)
/// - **inputs:** Array of input connections (NodeId or None)
/// - **output_buffer:** Buffer ID where this node writes output
/// - **params:** Parameter block for smoothed parameter automation
///
/// # Memory Layout
///
/// The processor is heap-allocated (Box) at node creation time, not during
/// audio processing. All other fields are inline in the arena slot.
///
/// # Example
///
/// ```
/// use aether_core::node::{NodeRecord, DspNode};
/// use aether_core::buffer_pool::{BufferPool, BufferId};
/// use aether_core::param::ParamBlock;
/// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
///
/// // Custom node implementation
/// struct Gain { gain: f32 }
///
/// impl DspNode for Gain {
///     fn process(
///         &mut self,
///         inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
///         output: &mut [f32; BUFFER_SIZE],
///         _params: &mut ParamBlock,
///         _sample_rate: f32,
///     ) {
///         if let Some(input) = inputs[0] {
///             for (i, out) in output.iter_mut().enumerate() {
///                 *out = input[i] * self.gain;
///             }
///         }
///     }
///
///     fn type_name(&self) -> &'static str {
///         "Gain"
///     }
/// }
///
/// // Create node record
/// let mut pool = BufferPool::new(10);
/// let buffer = pool.acquire().unwrap();
/// let processor = Box::new(Gain { gain: 0.5 });
/// let record = NodeRecord::new(processor, buffer);
/// ```
///
/// # Lifecycle
///
/// 1. **Creation:** Allocated in arena when `AddNode` command is processed
/// 2. **Processing:** `processor.process()` called each audio block
/// 3. **Removal:** Dropped when `RemoveNode` command is processed
///
/// # See Also
///
/// * [`DspNode`] - The processing trait
/// * [`Arena`](crate::arena::Arena) - Storage for node records
/// * [`Scheduler`](crate::scheduler::Scheduler) - Processes nodes in order
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
