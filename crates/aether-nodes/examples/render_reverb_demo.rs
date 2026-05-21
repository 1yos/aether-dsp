//! Render reverb effect demonstration.
//!
//! This example demonstrates the reverb effect by processing
//! a dry signal (short impulse) through the reverb node.
//!
//! Output files:
//! - reverb_dry.wav (dry impulse)
//! - reverb_wet.wav (with reverb)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use aether_nodes::reverb::Reverb;
use hound::{WavSpec, WavWriter};

const SAMPLE_RATE: u32 = 48000;
const DURATION_SECS: f32 = 3.0;
const NUM_SAMPLES: usize = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;

fn generate_impulse_train(num_samples: usize, sample_rate: f32) -> Vec<f32> {
    let mut signal = vec![0.0f32; num_samples];

    // Generate impulses every 0.5 seconds
    let impulse_interval = (sample_rate * 0.5) as usize;

    for i in (0..num_samples).step_by(impulse_interval) {
        if i < num_samples {
            signal[i] = 0.8; // Impulse
        }
    }

    signal
}

fn main() {
    println!("=== Reverb Effect Demonstration ===\n");

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Generate impulse train
    println!("Generating impulse train...");
    let signal = generate_impulse_train(NUM_SAMPLES, SAMPLE_RATE as f32);

    // Render dry (before)
    println!("Rendering dry signal to reverb_dry.wav...");
    let mut writer_dry = WavWriter::create("reverb_dry.wav", spec).unwrap();
    for &sample in &signal {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer_dry.write_sample(amplitude).unwrap();
    }
    writer_dry.finalize().unwrap();
    println!("✓ Rendered dry");

    // Render with reverb (wet)
    println!("Rendering with reverb to reverb_wet.wav...");
    let mut writer_wet = WavWriter::create("reverb_wet.wav", spec).unwrap();
    let mut reverb = Reverb::new(SAMPLE_RATE as f32);
    let mut params = ParamBlock::new();

    // Parameters: room_size, damping, wet_level, dry_level, width
    params.add(0.8); // Large room
    params.add(0.5); // Moderate damping
    params.add(0.7); // 70% wet
    params.add(0.3); // 30% dry
    params.add(1.0); // Full stereo width

    let mut sample_idx = 0;
    let mut input_buffer = [0.0f32; BUFFER_SIZE];

    while sample_idx < NUM_SAMPLES {
        // Fill input buffer
        for i in 0..BUFFER_SIZE {
            if sample_idx + i < NUM_SAMPLES {
                input_buffer[i] = signal[sample_idx + i];
            } else {
                input_buffer[i] = 0.0;
            }
        }

        let inputs = [Some(&input_buffer); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        reverb.process(&inputs, &mut output, &mut params, SAMPLE_RATE as f32);

        // Write output
        for &sample in &output {
            if sample_idx >= NUM_SAMPLES {
                break;
            }
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer_wet.write_sample(amplitude).unwrap();
            sample_idx += 1;
        }
    }

    writer_wet.finalize().unwrap();
    println!("✓ Rendered with reverb");

    println!("\n✓ Reverb demo rendered successfully!");
    println!("Compare: ffplay reverb_dry.wav");
    println!("         ffplay reverb_wet.wav");
    println!("\nNotice: The reverb version has a long decay tail after each impulse");
}
