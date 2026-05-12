//! Command Ring Example: Control Thread → RT Thread Communication
//!
//! This example demonstrates how to safely send commands from a control
//! thread to the real-time audio thread using a lock-free SPSC ring buffer.
//!
//! Run with: cargo run --example command_ring -p aetherdsp-core

use aether_core::{
    command::Command,
    node::DspNode,
    param::{Param, ParamBlock},
    scheduler::Scheduler,
    BUFFER_SIZE, MAX_INPUTS,
};
use ringbuf::{traits::*, HeapRb};
use std::thread;
use std::time::Duration;

/// Simple gain node that can be controlled via parameters
struct Gain {
    gain: f32,
}

impl DspNode for Gain {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        // Get smoothed gain from parameter block
        for (i, out) in output.iter_mut().enumerate() {
            let gain_param = params.get(0);
            *out = input[i] * gain_param.current;
            params.tick_all(); // Smooth to next value
        }
    }

    fn type_name(&self) -> &'static str {
        "Gain"
    }
}

fn main() {
    println!("AetherDSP Core - Command Ring Example");
    println!("======================================\n");

    // Create SPSC ring buffer for commands
    let (mut producer, mut consumer) = HeapRb::<Command>::new(1024).split();

    // Create scheduler
    let mut sched = Scheduler::new(48_000.0);

    // Add gain node
    let gain_id = sched
        .graph
        .add_node(Box::new(Gain { gain: 1.0 }))
        .expect("Failed to add node");

    sched.graph.set_output_node(gain_id);

    println!("✓ Created scheduler with gain node");
    println!("✓ Created command ring buffer (capacity: 1024)\n");

    // Simulate control thread sending commands
    let control_thread = thread::spawn(move || {
        println!("[Control Thread] Sending gain automation...\n");

        // Ramp gain from 0.0 to 1.0 over 10 steps
        for i in 0..=10 {
            let gain_value = i as f32 / 10.0;

            // Create parameter update command
            let cmd = Command::UpdateParam {
                node: gain_id,
                param_index: 0,
                new_param: Param::new(gain_value),
            };

            // Send to RT thread
            if producer.try_push(cmd).is_ok() {
                println!("[Control] Set gain = {:.1}", gain_value);
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("\n[Control Thread] Done sending commands");
    });

    // Simulate RT audio thread processing
    println!("[RT Thread] Processing audio blocks...\n");

    let mut output = vec![0.0f32; BUFFER_SIZE * 2];

    for block in 0..15 {
        // Process one block (this drains commands from the ring)
        sched.process_block(&mut consumer, &mut output);

        // Simulate audio callback timing
        thread::sleep(Duration::from_millis(80));

        if block % 3 == 0 {
            println!("[RT] Processed block {}", block);
        }
    }

    // Wait for control thread to finish
    control_thread.join().unwrap();

    println!("\n✓ Success! Commands were safely sent from control → RT thread");
    println!("✓ No locks, no allocations, no blocking");
}
