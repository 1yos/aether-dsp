//! Render filter cutoff sweep to demonstrate filter response.
//!
//! This example renders a white noise signal through a lowpass filter
//! with a sweeping cutoff frequency, demonstrating the filter's effect.
//!
//! Output files:
//! - filter_sweep_before.wav (unfiltered white noise)
//! - filter_sweep_after.wav (filtered with sweep)

use aether_core::{
    node::DspNode,
    param::ParamBlock,
    BUFFER_SIZE,
    MAX_INPUTS,
};
use aether_nodes::filter::StateVariableFilter;
use hound::{WavSpec, WavWriter};
use rand::Rng;

const SAMPLE_RATE: u32 = 48000;
const DURATION_SECS: f32 = 4.0;
const NUM_SAMPLES: usize = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;

// Filter mode parameter values
const FILTER_MODE_LOWPASS: f32 = 0.0;

fn main() {
    println!("=== Filter Cutoff Sweep Rendering ===\n");

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Generate white noise
    println!("Generating white noise...");
    let mut rng = rand::thread_rng();
    let mut noise_samples = Vec::with_capacity(NUM_SAMPLES);
    for _ in 0..NUM_SAMPLES {
        noise_samples.push(rng.gen_range(-0.3..0.3));
    }

    // Render unfiltered (before)
    println!("Rendering unfiltered noise to filter_sweep_before.wav...");
    let mut writer_before = WavWriter::create("filter_sweep_before.wav", spec).unwrap();
    for &sample in &noise_samples {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer_before.write_sample(amplitude).unwrap();
    }
    writer_before.finalize().unwrap();
    println!("✓ Rendered unfiltered");

    // Render filtered with sweep (after)
    println!("Rendering filtered with cutoff sweep to filter_sweep_after.wav...");
    let mut writer_after = WavWriter::create("filter_sweep_after.wav", spec).unwrap();
    let mut filter = StateVariableFilter::new();
    let mut params = ParamBlock::new();

    // Parameters: cutoff, resonance, mode
    params.add(200.0); // Start at 200 Hz
    params.add(0.7);   // Moderate resonance
    params.add(FILTER_MODE_LOWPASS);

    let mut sample_idx = 0;
    let mut input_buffer = [0.0f32; BUFFER_SIZE];

    while sample_idx < NUM_SAMPLES {
        // Fill input buffer
        for i in 0..BUFFER_SIZE {
            if sample_idx + i < NUM_SAMPLES {
                input_buffer[i] = noise_samples[sample_idx + i];
            } else {
                input_buffer[i] = 0.0;
            }
        }

        // Sweep cutoff from 200 Hz to 8000 Hz over duration
        let progress = sample_idx as f32 / NUM_SAMPLES as f32;
        let cutoff = 200.0 + progress * 7800.0;
        params.get_mut(0).set_target(cutoff, 0);

        let inputs = [Some(&input_buffer); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        filter.process(&inputs, &mut output, &mut params, SAMPLE_RATE as f32);

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
    println!("✓ Rendered filtered with sweep");

    println!("\n✓ Filter sweep rendered successfully!");
    println!("Compare: ffplay filter_sweep_before.wav");
    println!("         ffplay filter_sweep_after.wav");
}
