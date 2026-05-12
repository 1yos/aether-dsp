//! Simplest Possible Node: Gain Control
//!
//! This example shows the absolute minimum code needed to create
//! a custom DSP node using the #[aether_node] macro.
//!
//! Run with: cargo run --example simple_gain -p aetherdsp-ndk

use aether_ndk::prelude::*;

/// Simple gain/volume control
///
/// This is the simplest possible DSP node - it just multiplies
/// the input signal by a gain factor.
#[aether_node]
pub struct SimpleGain {
    /// Gain amount (0.0 = silence, 1.0 = unity, 2.0 = double)
    #[param(name = "Gain", min = 0.0, max = 2.0, default = 1.0)]
    gain: f32,
}

impl DspProcess for SimpleGain {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        // Get input buffer (or silence if not connected)
        let input = inputs.get(0);

        // Get current gain value (smoothed automatically)
        let gain = params.get(0).current;

        // Process each sample
        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * gain;
            params.tick_all(); // Advance parameter smoothing
        }
    }
}

fn main() {
    println!("AetherDSP NDK - Simple Gain Example");
    println!("====================================\n");

    // The #[aether_node] macro generated:
    // - Default impl (using parameter defaults)
    // - AetherNodeMeta trait (type_name, param_defs)
    // - PARAM_COUNT constant

    let gain = SimpleGain::default();

    println!("✓ Created {} node", SimpleGain::type_name());
    println!("✓ Parameter count: {}", SimpleGain::PARAM_COUNT);
    println!("\nParameters:");

    for def in SimpleGain::param_defs() {
        println!("  • {} [{:.1}–{:.1}] default={:.2}",
            def.name, def.min, def.max, def.default);
    }

    // Wrap for use in the engine
    let _boxed: Box<dyn aether_ndk::DspNode> = into_node(gain);

    println!("\n✓ Node wrapped and ready for the graph!");
    println!("\n💡 This node can now be:");
    println!("  • Added to an AudioGraph");
    println!("  • Connected to other nodes");
    println!("  • Controlled via parameters");
    println!("  • Processed in real-time");
}
