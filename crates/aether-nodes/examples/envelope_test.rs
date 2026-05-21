//! ADSR envelope demonstration
//!
//! This example demonstrates:
//! - ADSR envelope (Attack, Decay, Sustain, Release)
//! - Gate triggering (note on/off)
//! - Envelope modulation of audio signal
//! - Multiple note triggers
//!
//! # Usage
//!
//! ```bash
//! cargo run --example envelope_test -p aetherdsp-nodes
//! ```
//!
//! Output: `envelope_test.wav` (6 seconds, 48kHz, mono)
//!
//! # What You'll Hear
//!
//! Three notes played with an ADSR envelope:
//! 1. Short note (0.5s gate)
//! 2. Medium note (1.0s gate)
//! 3. Long note (1.5s gate)
//!
//! Each note demonstrates the full ADSR cycle:
//! - Attack: 0.1s (fade in)
//! - Decay: 0.2s (drop to sustain level)
//! - Sustain: 0.7 level (held while gate is on)
//! - Release: 0.3s (fade out after gate off)

use aether_core::{
    command::Command, graph::DspGraph, param::Param, scheduler::Scheduler, BUFFER_SIZE,
};
use aether_nodes::{envelope::AdsrEnvelope, oscillator::Oscillator};
use hound::{WavSpec, WavWriter};
use ringbuf::{
    traits::{Producer, Split},
    HeapRb,
};

const SAMPLE_RATE: f32 = 48_000.0;
const DURATION_SECS: f32 = 6.0;
const TOTAL_SAMPLES: usize = (SAMPLE_RATE * DURATION_SECS) as usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 ADSR Envelope Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Rendering: Oscillator → ADSR Envelope → Output");
    println!("Duration: {} seconds", DURATION_SECS);
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!();

    // Create command ring buffer
    let (mut producer, mut consumer) = HeapRb::<Command>::new(1024).split();

    // Create scheduler and graph
    let mut scheduler = Scheduler::new(SAMPLE_RATE);
    let mut graph = DspGraph::new();

    // Build audio graph: oscillator → envelope → output
    println!("🔧 Building audio graph...");

    // Create oscillator (sine wave at 440 Hz)
    let osc = Box::new(Oscillator::new());
    let osc_id = graph.add_node(osc).expect("Failed to add oscillator");

    // Create ADSR envelope
    let envelope = Box::new(AdsrEnvelope::new());
    let env_id = graph.add_node(envelope).expect("Failed to add envelope");

    // Connect: oscillator → envelope input 0
    graph.connect(osc_id, env_id, 0);

    // Set envelope as output
    graph.set_output_node(env_id);

    // Send commands to scheduler
    producer.try_push(Command::AddNode { id: osc_id }).ok();
    producer.try_push(Command::AddNode { id: env_id }).ok();
    producer
        .try_push(Command::Connect {
            src: osc_id,
            dst: env_id,
            slot: 0,
        })
        .ok();
    producer
        .try_push(Command::SetOutputNode { id: env_id })
        .ok();

    // Set oscillator parameters: frequency=440Hz, amplitude=1.0, waveform=0 (sine)
    producer
        .try_push(Command::UpdateParam {
            node: osc_id,
            param_index: 0,
            new_param: Param::new(440.0), // frequency
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: osc_id,
            param_index: 1,
            new_param: Param::new(1.0), // amplitude
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: osc_id,
            param_index: 2,
            new_param: Param::new(0.0), // waveform (0=sine)
        })
        .ok();

    // Set envelope parameters: A=0.1s, D=0.2s, S=0.7, R=0.3s, gate=0
    producer
        .try_push(Command::UpdateParam {
            node: env_id,
            param_index: 0,
            new_param: Param::new(0.1), // attack
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: env_id,
            param_index: 1,
            new_param: Param::new(0.2), // decay
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: env_id,
            param_index: 2,
            new_param: Param::new(0.7), // sustain
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: env_id,
            param_index: 3,
            new_param: Param::new(0.3), // release
        })
        .ok();
    producer
        .try_push(Command::UpdateParam {
            node: env_id,
            param_index: 4,
            new_param: Param::new(0.0), // gate (off)
        })
        .ok();

    println!("  ✅ Oscillator (440 Hz) → ADSR Envelope → Output");
    println!("  📊 Envelope: A=0.1s, D=0.2s, S=0.7, R=0.3s");
    println!();

    // Create WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create("envelope_test.wav", spec)?;

    println!("🎬 Rendering audio...");

    // Define note triggers (time in seconds, gate duration in seconds)
    let notes = vec![
        (0.5, 0.5), // Note 1: starts at 0.5s, held for 0.5s
        (2.0, 1.0), // Note 2: starts at 2.0s, held for 1.0s
        (4.0, 1.5), // Note 3: starts at 4.0s, held for 1.5s
    ];

    let mut current_note = 0;
    let mut gate_on_time: Option<f32> = None;

    // Process audio in blocks
    let mut samples_rendered = 0;
    let mut output_buffer = vec![0.0f32; BUFFER_SIZE];

    while samples_rendered < TOTAL_SAMPLES {
        let current_time = samples_rendered as f32 / SAMPLE_RATE;

        // Check if we need to trigger a note
        if current_note < notes.len() {
            let (trigger_time, duration) = notes[current_note];

            // Gate ON
            if gate_on_time.is_none() && current_time >= trigger_time {
                producer
                    .try_push(Command::UpdateParam {
                        node: env_id,
                        param_index: 4,
                        new_param: Param::new(1.0), // gate ON
                    })
                    .ok();
                gate_on_time = Some(current_time);
                println!("  🎹 Note {} ON  @ {:.2}s", current_note + 1, current_time);
            }

            // Gate OFF
            if let Some(on_time) = gate_on_time {
                if current_time >= on_time + duration {
                    producer
                        .try_push(Command::UpdateParam {
                            node: env_id,
                            param_index: 4,
                            new_param: Param::new(0.0), // gate OFF
                        })
                        .ok();
                    println!("  🎹 Note {} OFF @ {:.2}s", current_note + 1, current_time);
                    gate_on_time = None;
                    current_note += 1;
                }
            }
        }

        // Process one block
        scheduler.process_block(&mut consumer, &mut output_buffer);

        // Write to WAV file
        for &sample in output_buffer.iter() {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        samples_rendered += BUFFER_SIZE;
    }

    writer.finalize()?;

    println!();
    println!("✅ Rendering complete!");
    println!("📁 Output: envelope_test.wav");
    println!();
    println!("🎧 What to listen for:");
    println!("   - Note 1: Quick attack, short sustain, smooth release");
    println!("   - Note 2: Same envelope, longer sustain phase");
    println!("   - Note 3: Same envelope, longest sustain phase");
    println!(
        "   - Each note fades in (attack), drops (decay), holds (sustain), fades out (release)"
    );
    println!();
    println!("💡 Try modifying:");
    println!("   - Attack time (line 107)");
    println!("   - Decay time (line 112)");
    println!("   - Sustain level (line 117)");
    println!("   - Release time (line 122)");
    println!("   - Note timings (line 153)");

    Ok(())
}
