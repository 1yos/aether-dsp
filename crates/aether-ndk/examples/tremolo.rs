//! Example: Tremolo node built with #[aether_node]
//!
//! Run with: cargo run --example tremolo -p aether-ndk

use aether_ndk::prelude::*;

/// Amplitude modulation (tremolo) using a sine LFO.
#[aether_node]
pub struct Tremolo {
    #[param(name = "Rate",  min = 0.1,  max = 20.0, default = 4.0)]
    rate: f32,
    #[param(name = "Depth", min = 0.0,  max = 1.0,  default = 0.5)]
    depth: f32,
    // Internal state — not a param
    phase: f32,
}

impl DspProcess for Tremolo {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let input = inputs.get(0);
        for (i, out) in output.iter_mut().enumerate() {
            let rate  = params.get(0).current;
            let depth = params.get(1).current;
            // LFO: 1.0 at phase=0, dips to (1-depth) at phase=0.5
            let lfo = 1.0 - depth * 0.5 * (1.0 - (self.phase * std::f32::consts::TAU).cos());
            *out = input[i] * lfo;
            self.phase = (self.phase + rate / sample_rate).fract();
            params.tick_all();
        }
    }
}

fn main() {
    // Demonstrate that the macro generated Default, param_defs, and type_name.
    let t = Tremolo::default();
    println!("Node type:  {}", Tremolo::type_name());
    println!("Param count: {}", Tremolo::PARAM_COUNT);
    for def in Tremolo::param_defs() {
        println!("  {} [{:.1}–{:.1}] default={:.2}", def.name, def.min, def.max, def.default);
    }

    // Wrap it for the engine
    let _boxed: Box<dyn aether_ndk::DspNode> = into_node(t);
    println!("\nNode wrapped and ready for the graph ✓");
}
