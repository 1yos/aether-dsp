//! Render compressor effect demonstration.
//!
//! This example demonstrates dynamic range compression by processing
//! a signal with varying amplitude through a compressor.
//!
//! Output files:
//! - compressor_before.wav (uncompressed)
//! - compressor_after.wav (compressed)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use aether_nodes::Compressor;
use hound::{WavSpec, WavWriter};

const SAMPLE_RATE: u32 = 48000;
const DURATION_SECS: f32 = 4.0;
const NUM_SAMPLES: usize = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;

fn generate_dynamic_signal(num_samples: usize, sample_rate: f32) -> Vec<f32> {
    let mut signal = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate;

        // 440 Hz sine wave
        let sine = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();

        // Amplitude envelope: quiet -> loud -> quiet -> loud
        let envelope_freq = 0.5; // 0.5 Hz (2 second period)
        let envelope = 0.2 + 0.6 * (t * envelope_freq * 2.0 * std::f32::consts::PI).sin().abs();

        signal.push(sine * envelope);
    }

    signal
}

fn main() {
    println!("=== Compressor Effect Demonstration ===\n");

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Generate dynamic signal
    println!("Generating dynamic signal (quiet -> loud -> quiet -> loud)...");
    let signal = generate_dynamic_signal(NUM_SAMPLES, SAMPLE_RATE as f32);

    // Render uncompressed (before)
    println!("Rendering uncompressed to compressor_before.wav...");
    let mut writer_before = WavWriter::create("compressor_before.wav", spec).unwrap();
    for &sample in &signal {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer_before.write_sample(amplitude).unwrap();
    }
    writer_before.finalize().unwrap();
    println!("✓ Rendered uncompressed");

    // Render compressed (after)
    println!("Rendering compressed to compressor_after.wav...");
    let mut writer_after = WavWriter::create("compressor_after.wav", spec).unwrap();
    let mut compressor = Compressor::new();
    let mut params = ParamBlock::new();

    // Parameters: threshold, ratio, attack, release, makeup, knee
    params.add(-20.0); // -20 dB threshold
    params.add(4.0); // 4:1 ratio
    params.add(5.0); // 5 ms attack
    params.add(100.0); // 100 ms release
    params.add(6.0); // 6 dB makeup gain
    params.add(3.0); // 3 dB soft knee

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
        compressor.process(&inputs, &mut output, &mut params, SAMPLE_RATE as f32);

        // Write output
        for &sample in &output {
            if sample_idx >= NUM_SAMPLES {
                break;
            }
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer_after.write_sample(amplitude).unwrap();
            sample_idx += 1;
        }
    }

    writer_after.finalize().unwrap();
    println!("✓ Rendered compressed");

    println!("\n✓ Compressor demo rendered successfully!");
    println!("Compare: ffplay compressor_before.wav");
    println!("         ffplay compressor_after.wav");
    println!("\nNotice: The compressed version has more consistent volume");
}
