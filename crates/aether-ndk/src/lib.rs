//! Aether Node Development Kit
//!
//! Everything a third-party developer needs to build a DSP node:
//!
//! ```rust
//! use aether_ndk::prelude::*;
//!
//! #[aether_node]
//! pub struct Tremolo {
//!     #[param(name = "Rate", min = 0.1, max = 20.0, default = 4.0)]
//!     rate: f32,
//!     #[param(name = "Depth", min = 0.0, max = 1.0, default = 0.5)]
//!     depth: f32,
//!     phase: f32,
//! }
//!
//! impl DspProcess for Tremolo {
//!     fn process(
//!         &mut self,
//!         inputs: &NodeInputs,
//!         output: &mut NodeOutput,
//!         params: &mut ParamBlock,
//!         sample_rate: f32,
//!     ) {
//!         let input = inputs.get(0);
//!         for (i, out) in output.iter_mut().enumerate() {
//!             let rate = params.get(0).current;
//!             let depth = params.get(1).current;
//!             let lfo = 1.0 - depth * 0.5 * (1.0 - (self.phase * std::f32::consts::TAU).cos());
//!             *out = input[i] * lfo;
//!             self.phase = (self.phase + rate / sample_rate).fract();
//!             params.tick_all();
//!         }
//!     }
//! }
//! ```

pub mod node;
pub mod param;
pub mod registry;
pub mod schema;

pub use aether_ndk_macro::aether_node;

// Re-export core types so node authors don't need to depend on aether-core directly
pub use aether_core::{
    node::DspNode,
    param::ParamBlock,
    state::StateBlob,
    BUFFER_SIZE, MAX_INPUTS,
};

/// A parameter definition — name, range, and default value.
/// Generated statically by `#[aether_node]`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParamDef {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

/// Metadata trait auto-implemented by `#[aether_node]`.
pub trait AetherNodeMeta {
    fn type_name() -> &'static str;
    fn param_defs() -> &'static [ParamDef];
}

/// The processing trait node authors implement.
/// Simpler than `DspNode` — no raw pointer handling required.
pub trait DspProcess: Send {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        sample_rate: f32,
    );

    fn capture_state(&self) -> StateBlob { StateBlob::EMPTY }
    fn restore_state(&mut self, _state: StateBlob) {}
}

/// Ergonomic wrapper around the raw input buffer array.
pub struct NodeInputs<'a> {
    pub raw: &'a [Option<&'a [f32; BUFFER_SIZE]>; MAX_INPUTS],
}

impl<'a> NodeInputs<'a> {
    pub fn get(&self, slot: usize) -> &[f32; BUFFER_SIZE] {
        static SILENCE: [f32; BUFFER_SIZE] = [0.0f32; BUFFER_SIZE];
        self.raw.get(slot).and_then(|s| *s).unwrap_or(&SILENCE)
    }
}

/// Ergonomic wrapper around the output buffer.
pub type NodeOutput = [f32; BUFFER_SIZE];

/// Adapter that bridges `DspProcess` → `DspNode` so NDK nodes
/// work transparently inside the AetherDSP engine.
pub struct NodeAdapter<T: DspProcess + AetherNodeMeta> {
    pub inner: T,
}

impl<T: DspProcess + AetherNodeMeta + 'static> DspNode for NodeAdapter<T> {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let wrapped = NodeInputs { raw: inputs };
        self.inner.process(&wrapped, output, params, sample_rate);
    }

    fn capture_state(&self) -> StateBlob { self.inner.capture_state() }
    fn restore_state(&mut self, state: StateBlob) { self.inner.restore_state(state); }
    fn type_name(&self) -> &'static str { T::type_name() }
}

/// Convenience function: wrap any `DspProcess + AetherNodeMeta` into a
/// boxed `DspNode` ready for the graph.
pub fn into_node<T>(node: T) -> Box<dyn DspNode>
where
    T: DspProcess + AetherNodeMeta + 'static,
{
    Box::new(NodeAdapter { inner: node })
}

/// Prelude — import everything needed to write a node.
pub mod prelude {
    pub use super::{
        aether_node, into_node, AetherNodeMeta, DspProcess, NodeInputs,
        NodeOutput, ParamDef,
    };
    pub use aether_core::param::ParamBlock;
    pub use aether_core::state::StateBlob;
    pub use aether_core::{BUFFER_SIZE, MAX_INPUTS};
}
