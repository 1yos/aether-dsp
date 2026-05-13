//! Reverb demonstration with different room sizes
//!
//! This example demonstrates:
//! - Freeverb algorithmic reverb
//! - Room size parameter changes
//! - Wet/dry mix control
//! - Damping (high-frequency absorption)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example reverb_demo -p aetherdsp-nodes
//! ```
//!
//! Output: `reverb_demo.wav` (12 seconds, 48kHz, mono)
//!
//! # What You'll Hear
//!
//! Four drum hits with increasing reverb room sizes:
//! 1. Small room (size=0.3) - tight, short reverb
//! 2. Medium room (size=0.5) - balanced reverb
//! 3. Large room (size=0.7) - spacious reverb
//! 4. Hall (size=0.9) - long, cathedral-like reverb
//!
//! Each hit demonstrates how room size affects reverb tail length and density.

use aether_core::{
    command::Command,
    graph::DspGraph,
    param::Param,
    scheduler::Scheduler,
    BUFFER_SIZE,
};
use aether_nodes::{envelope::AdsrEnvelope, oscillator::Oscillator, reverb::Reverb};
use hound::{WavSpec, WavWriter};
use ringbuf::{traits::{Producer, Split}, HeapRb};

const SAMPLE_RATE: f32 = 48_000.0;
const DURATION_SECS: f32 = 12.0;
const TOTAL_SAMPLES: usize = (SAMPLE_RATE * DURATION_SECS) as usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Reverb Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Rendering: Oscillator → Envelope → Reverb → Output");
    println!("Duration: {} seconds", DURATION_SECS);
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!();

    // Create command ring buffer
    let (mut producer, mut consumer) = HeapRb::<Command>::new(1024).split();

    // Create scheduler and graph
    let mut scheduler = Scheduler::new(SAMPLE_RATE);
    let mut graph = DspGraph::new();

    // Build audio graph: oscillator → envelope → reverb → output
    println!("🔧 Building audio graph...");

    // Create oscillator (sine wave for drum-like sound)
    let osc = Box::new(Oscillator::new());
    let osc_id = graph.add_node(osc).expect("Failed to add oscillator");

    // Create ADSR envelope (short attack, quick decay for percussive sound)
    let envelope = Box::new(AdsrEnvelope::new());
    let env_id = graph.add_node(envelope).expect("Failed to add envelope");

    // Create reverb
    let reverb = Box::new(Reverb::new(SAMPLE_RATE));
    let reverb_id = graph.add_node(reverb).expect("Failed to add reverb");

    // Connect: oscillator → envelope → reverb → output
    graph.connect(osc_id, env_id, 0);
    graph.connect(env_id, reverb_id, 0);
    graph.set_output_node(reverb_id);

    // Send commands to scheduler
    producer.try_push(Command::AddNode { id: osc_id }).ok();
    producer.try_push(Command::AddNode { id: env_id }).ok();
    producer.try_push(Command::AddNode { id: reverb_id }).ok();
    producer.try_push(Command::Connect { src: osc_id, dst: env_id, slot: 0 }).ok();
    producer.try_push(Command::Connect { src: env_id, dst: reverb_id, slot: 0 }).ok();
    producer.try_push(Command::SetOutputNode { id: reverb_id }).ok();

    // Set oscillator parameters: frequency=200Hz (low drum), amplitude=1.0, sine wave
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 0,
        new_param: Param::new(200.0),  // frequency
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 1,
        new_param: Param::new(1.0),    // amplitude
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 2,
        new_param: Param::new(0.0),    // waveform (0=sine)
    }).ok();

    // Set envelope parameters: very short for percussive sound
    producer.try_push(Command::UpdateParam {
        node: env_id,
        param_index: 0,
        new_param: Param::new(0.001),  // attack (1ms)
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: env_id,
        param_index: 1,
        new_param: Param::new(0.05),   // decay (50ms)
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: env_id,
        param_index: 2,
        new_param: Param::new(0.0),    // sustain (0 - no sustain)
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: env_id,
        param_index: 3,
        new_param: Param::new(0.1),    // release (100ms)
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: env_id,
        param_index: 4,
        new_param: Param::new(0.0),    // gate (off)
    }).ok();

    // Set initial reverb parameters: room=0.3, damping=0.5, wet=0.8, width=1.0
    producer.try_push(Command::UpdateParam {
        node: reverb_id,
        param_index: 0,
        new_param: Param::new(0.3),    // room size
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: reverb_id,
        param_index: 1,
        new_param: Param::new(0.5),    // damping
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: reverb_id,
        param_index: 2,
        new_param: Param::new(0.8),    // wet
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: reverb_id,
        param_index: 3,
        new_param: Param::new(1.0),    // width
    }).ok();

    println!("  ✅ Oscillator → Envelope → Reverb → Output");
    println!("  🥁 Percussive sound: A=1ms, D=50ms, S=0, R=100ms");
    println!();

    // Create WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create("reverb_demo.wav", spec)?;

    println!("🎬 Rendering audio...");

    // Define drum hits with different room sizes
    let hits = vec![
        (1.0, 0.3, "Small room"),
        (4.0, 0.5, "Medium room"),
        (7.0, 0.7, "Large room"),
        (10.0, 0.9, "Hall"),
    ];

    let mut current_hit = 0;

    // Process audio in blocks
    let mut samples_rendered = 0;
    let mut output_buffer = vec![0.0f32; BUFFER_SIZE];

    while samples_rendered < TOTAL_SAMPLES {
        let current_time = samples_rendered as f32 / SAMPLE_RATE;

        // Check if we need to trigger a hit
        if current_hit < hits.len() {
            let (trigger_time, room_size, description) = hits[current_hit];

            if current_time >= trigger_time && current_time < trigger_time + 0.01 {
                // Update room size
                producer.try_push(Command::UpdateParam {
                    node: reverb_id,
                    param_index: 0,
                    new_param: Param::new(room_size),
                }).ok();

                // Trigger gate ON then OFF
                producer.try_push(Command::UpdateParam {
                    node: env_id,
                    param_index: 4,
                    new_param: Param::new(1.0),  // gate ON
                }).ok();
                producer.try_push(Command::UpdateParam {
                    node: env_id,
                    param_index: 4,
                    new_param: Param::new(0.0),  // gate OFF
                }).ok();

                println!("  🥁 Hit {} @ {:.1}s - {} (size={:.1})", 
                         current_hit + 1, current_time, description, room_size);

                current_hit += 1;
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
    println!("📁 Output: reverb_demo.wav");
    println!();
    println!("🎧 What to listen for:");
    println!("   - Hit 1: Tight, short reverb tail");
    println!("   - Hit 2: Balanced reverb, medium decay");
    println!("   - Hit 3: Spacious, longer reverb tail");
    println!("   - Hit 4: Cathedral-like, very long tail");
    println!();
    println!("💡 Try modifying:");
    println!("   - Room sizes (line 177)");
    println!("   - Damping (line 147) - controls high-frequency absorption");
    println!("   - Wet mix (line 152) - balance dry/wet signal");
    println!("   - Oscillator frequency (line 99) - drum pitch");

    Ok(())
}
