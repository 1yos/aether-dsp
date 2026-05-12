//! Graph Chain Example: Oscillator → Gain → Output
//!
//! This example demonstrates how to build a simple DSP graph with
//! multiple connected nodes.
//!
//! Run with: cargo run --example graph_chain -p aetherdsp-core

use aether_core::{
    node::DspNode, param::ParamBlock, scheduler::Scheduler, BUFFER_SIZE, MAX_INPUTS,
};

/// Sine wave oscillator
struct Oscillator {
    frequency: f32,
    phase: f32,
}

impl DspNode for Oscillator {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let phase_inc = self.frequency / sample_rate;
        for sample in output.iter_mut() {
            *sample = (self.phase * 2.0 * std::f32::consts::PI).sin();
            self.phase = (self.phase + phase_inc).fract();
        }
    }

    fn type_name(&self) -> &'static str {
        "Oscillator"
    }
}

/// Gain/volume control
struct Gain {
    amount: f32,
}

impl DspNode for Gain {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * self.amount;
        }
    }

    fn type_name(&self) -> &'static str {
        "Gain"
    }
}

fn main() {
    println!("AetherDSP Core - Graph Chain Example");
    println!("=====================================\n");

    // Create scheduler
    let mut sched = Scheduler::new(48_000.0);

    // Add oscillator (440Hz A4)
    let osc_id = sched
        .graph
        .add_node(Box::new(Oscillator {
            frequency: 440.0,
            phase: 0.0,
        }))
        .expect("Failed to add oscillator");

    println!("✓ Added Oscillator (440Hz)");

    // Add gain (50% volume)
    let gain_id = sched
        .graph
        .add_node(Box::new(Gain { amount: 0.5 }))
        .expect("Failed to add gain");

    println!("✓ Added Gain (50%)");

    // Connect: Oscillator output → Gain input (slot 0)
    sched.graph.connect(osc_id, gain_id, 0);
    println!("✓ Connected Oscillator → Gain");

    // Set gain as output node
    sched.graph.set_output_node(gain_id);
    println!("✓ Set Gain as output node\n");

    println!("Graph structure:");
    println!("  [Oscillator] → [Gain] → [Output]");
    println!("     440Hz         50%       DAC\n");

    // Process audio
    let mut output = vec![0.0f32; BUFFER_SIZE * 2];
    sched.process_block_simple(&mut output);

    println!("✓ Processed {} samples", output.len());

    // Analyze output
    let max_amplitude = output
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    println!("\nOutput analysis:");
    println!("  Max amplitude: {:.4}", max_amplitude);
    println!("  Expected: ~0.5 (sine wave * 50% gain)");

    if (max_amplitude - 0.5).abs() < 0.1 {
        println!("\n✓ Success! Graph processed correctly.");
    } else {
        println!("\n⚠ Warning: Unexpected amplitude");
    }
}
