//! Filter cutoff frequency sweep demonstration
//!
//! This example demonstrates:
//! - Parameter automation (sweeping filter cutoff)
//! - Connecting nodes (oscillator → filter → output)
//! - Rendering audio to WAV file
//! - Using aether-nodes DSP nodes
//!
//! # Usage
//!
//! ```bash
//! cargo run --example filter_sweep -p aetherdsp-nodes
//! ```
//!
//! Output: `filter_sweep.wav` (5 seconds, 48kHz, mono)
//!
//! # What You'll Hear
//!
//! A sawtooth wave at 220 Hz (A3) with a low-pass filter sweeping from
//! 200 Hz to 8000 Hz over 5 seconds. The sound starts muffled and gradually
//! becomes brighter as more harmonics pass through the filter.

use aether_core::{
    command::Command,
    graph::DspGraph,
    param::Param,
    scheduler::Scheduler,
    BUFFER_SIZE,
};
use aether_nodes::{filter::StateVariableFilter, oscillator::Oscillator};
use hound::{WavSpec, WavWriter};
use ringbuf::{traits::{Producer, Split}, HeapRb};

const SAMPLE_RATE: f32 = 48_000.0;
const DURATION_SECS: f32 = 5.0;
const TOTAL_SAMPLES: usize = (SAMPLE_RATE * DURATION_SECS) as usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Filter Sweep Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Rendering: Sawtooth wave (220 Hz) → Low-pass filter sweep");
    println!("Duration: {} seconds", DURATION_SECS);
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!();

    // Create command ring buffer
    let (mut producer, mut consumer) = HeapRb::<Command>::new(1024).split();

    // Create scheduler and graph
    let mut scheduler = Scheduler::new(SAMPLE_RATE);
    let mut graph = DspGraph::new();

    // Build audio graph: oscillator → filter → output
    println!("🔧 Building audio graph...");

    // Create oscillator (sawtooth wave at 220 Hz)
    let osc = Box::new(Oscillator::new());
    let osc_id = graph.add_node(osc).expect("Failed to add oscillator");

    // Create filter (low-pass mode)
    let filter = Box::new(StateVariableFilter::new());
    let filter_id = graph.add_node(filter).expect("Failed to add filter");

    // Connect: oscillator → filter input 0
    graph.connect(osc_id, filter_id, 0);

    // Set filter as output
    graph.set_output_node(filter_id);

    // Send commands to scheduler
    producer.try_push(Command::AddNode { id: osc_id }).ok();
    producer.try_push(Command::AddNode { id: filter_id }).ok();
    producer.try_push(Command::Connect { src: osc_id, dst: filter_id, slot: 0 }).ok();
    producer.try_push(Command::SetOutputNode { id: filter_id }).ok();

    // Set oscillator parameters: frequency=220Hz, amplitude=0.5, waveform=1 (sawtooth)
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 0,
        new_param: Param::new(220.0),  // frequency
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 1,
        new_param: Param::new(0.5),    // amplitude
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: osc_id,
        param_index: 2,
        new_param: Param::new(1.0),    // waveform (1=sawtooth)
    }).ok();

    // Set filter parameters: cutoff=200Hz, Q=1.0, mode=0 (low-pass)
    producer.try_push(Command::UpdateParam {
        node: filter_id,
        param_index: 0,
        new_param: Param::new(200.0),  // cutoff
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: filter_id,
        param_index: 1,
        new_param: Param::new(1.0),    // Q (resonance)
    }).ok();
    producer.try_push(Command::UpdateParam {
        node: filter_id,
        param_index: 2,
        new_param: Param::new(0.0),    // mode (0=LP)
    }).ok();

    println!("  ✅ Oscillator (220 Hz sawtooth) → Filter (LP) → Output");
    println!();

    // Create WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create("filter_sweep.wav", spec)?;

    println!("🎬 Rendering audio...");

    // Calculate sweep parameters
    let start_cutoff: f32 = 200.0;
    let end_cutoff: f32 = 8000.0;
    let sweep_samples = TOTAL_SAMPLES;

    // Process audio in blocks
    let mut samples_rendered = 0;
    let mut output_buffer = vec![0.0f32; BUFFER_SIZE];

    while samples_rendered < TOTAL_SAMPLES {
        // Calculate current cutoff frequency (logarithmic sweep)
        let progress = samples_rendered as f32 / sweep_samples as f32;
        let log_start = start_cutoff.ln();
        let log_end = end_cutoff.ln();
        let current_cutoff = (log_start + progress * (log_end - log_start)).exp();

        // Update filter cutoff parameter
        let new_cutoff = Param::new(current_cutoff);
        producer.try_push(Command::UpdateParam {
            node: filter_id,
            param_index: 0,
            new_param: new_cutoff,
        }).ok();

        // Process one block
        scheduler.process_block(&mut consumer, &mut output_buffer);

        // Write to WAV file
        for &sample in output_buffer.iter() {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        samples_rendered += BUFFER_SIZE;

        // Progress indicator
        if samples_rendered % (SAMPLE_RATE as usize) == 0 {
            let seconds = samples_rendered / SAMPLE_RATE as usize;
            println!("  ⏱️  {} / {} seconds (cutoff: {:.0} Hz)", 
                     seconds, DURATION_SECS as usize, current_cutoff);
        }
    }

    writer.finalize()?;

    println!();
    println!("🎧 What to listen for:");
    println!("   - Sound starts muffled (low cutoff)");
    println!("   - Gradually becomes brighter");
    println!("   - More harmonics become audible");
    println!("   - Ends with full brightness (high cutoff)");
    println!();
    println!("💡 Try modifying:");
    println!("   - Oscillator frequency (line 73)");
    println!("   - Sweep range (lines 127-128)");
    println!("   - Filter Q/resonance (line 87)");
    println!("   - Filter mode: 0=LP, 1=HP, 2=BP (line 92)");

    Ok(())
}
