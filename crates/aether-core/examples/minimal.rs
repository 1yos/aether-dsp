//! Minimal example: Oscillator → Output
//!
//! This example demonstrates the absolute minimum code needed to:
//! 1. Create a scheduler
//! 2. Add a simple DSP node (sine oscillator)
//! 3. Process one audio block
//!
//! Run with: cargo run --example minimal -p aetherdsp-core

use aether_core::{
    node::DspNode, param::ParamBlock, scheduler::Scheduler, BUFFER_SIZE, MAX_INPUTS,
};

/// Simple sine wave oscillator
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
        let phase_increment = self.frequency / sample_rate;

        for sample in output.iter_mut() {
            // Generate sine wave
            *sample = (self.phase * 2.0 * std::f32::consts::PI).sin() * 0.3;

            // Advance phase
            self.phase += phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn type_name(&self) -> &'static str {
        "Oscillator"
    }
}

fn main() {
    println!("AetherDSP Core - Minimal Example");
    println!("=================================\n");

    // Create scheduler at 48kHz
    let mut sched = Scheduler::new(48_000.0);

    // Create a 440Hz sine oscillator
    let osc = Box::new(Oscillator {
        frequency: 440.0,
        phase: 0.0,
    });

    // Add node to graph
    let osc_id = sched.graph.add_node(osc).expect("Failed to add node");

    // Set as output node (sends to DAC)
    sched.graph.set_output_node(osc_id);

    println!("✓ Created scheduler at 48kHz");
    println!("✓ Added 440Hz sine oscillator");
    println!("✓ Set as output node\n");

    // Process one block (64 samples, stereo = 128 values)
    let mut output = vec![0.0f32; BUFFER_SIZE * 2];
    sched.process_block_simple(&mut output);

    println!("✓ Processed {} samples", output.len());
    println!("\nFirst 10 samples:");
    for (i, sample) in output.iter().take(10).enumerate() {
        println!("  [{:2}] {:+.6}", i, sample);
    }

    println!("\n✓ Success! The oscillator generated audio.");
}
