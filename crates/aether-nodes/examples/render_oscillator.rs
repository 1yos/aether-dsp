//! Render oscillator waveforms to WAV files.
//!
//! This example demonstrates the oscillator node by rendering
//! different waveforms (sine, saw, square, triangle) to WAV files.
//!
//! Output files:
//! - oscillator_sine_440hz.wav
//! - oscillator_saw_440hz.wav
//! - oscillator_square_440hz.wav
//! - oscillator_triangle_440hz.wav

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use aether_nodes::oscillator::Oscillator;
use hound::{WavSpec, WavWriter};

const SAMPLE_RATE: u32 = 48000;
const DURATION_SECS: f32 = 2.0;
const NUM_SAMPLES: usize = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;

// Waveform parameter values
const WAVEFORM_SINE: f32 = 0.0;
const WAVEFORM_SAW: f32 = 1.0;
const WAVEFORM_SQUARE: f32 = 2.0;
const WAVEFORM_TRIANGLE: f32 = 3.0;

fn render_waveform(waveform: f32, waveform_name: &str, filename: &str) {
    println!("Rendering {} to {}...", waveform_name, filename);

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(filename, spec).unwrap();
    let mut osc = Oscillator::new();
    let mut params = ParamBlock::new();

    // Parameters: frequency, amplitude, waveform, midi_note
    params.add(440.0); // A4
    params.add(0.5); // 50% amplitude
    params.add(waveform); // Waveform type
    params.add(-1.0); // Use frequency param (not MIDI)

    let inputs = [None; MAX_INPUTS];
    let mut samples_rendered = 0;

    while samples_rendered < NUM_SAMPLES {
        let mut output = [0.0f32; BUFFER_SIZE];
        osc.process(&inputs, &mut output, &mut params, SAMPLE_RATE as f32);

        for &sample in &output {
            if samples_rendered >= NUM_SAMPLES {
                break;
            }
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude).unwrap();
            samples_rendered += 1;
        }
    }

    writer.finalize().unwrap();
    println!("✓ Rendered {} samples to {}", samples_rendered, filename);
}

fn main() {
    println!("=== Oscillator Waveform Rendering ===\n");

    render_waveform(WAVEFORM_SINE, "Sine", "oscillator_sine_440hz.wav");
    render_waveform(WAVEFORM_SAW, "Saw", "oscillator_saw_440hz.wav");
    render_waveform(WAVEFORM_SQUARE, "Square", "oscillator_square_440hz.wav");
    render_waveform(
        WAVEFORM_TRIANGLE,
        "Triangle",
        "oscillator_triangle_440hz.wav",
    );

    println!("\n✓ All waveforms rendered successfully!");
    println!("Play the files with: ffplay oscillator_sine_440hz.wav");
}
