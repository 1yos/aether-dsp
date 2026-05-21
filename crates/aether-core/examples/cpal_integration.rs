//! Full CPAL audio stream integration example
//!
//! This example demonstrates how to integrate aether-core with CPAL for real-time audio I/O.
//!
//! # What This Example Shows
//!
//! - Initializing CPAL audio device
//! - Creating scheduler in audio thread
//! - Handling buffer size mismatches
//! - Lock-free command sending from main thread
//! - Graceful error handling
//! - Building a simple audio graph (oscillator → output)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example cpal_integration -p aetherdsp-core --features="cpal"
//! ```
//!
//! Press Ctrl+C to stop.
//!
//! # Platform Notes
//!
//! - **Windows:** Uses WASAPI by default
//! - **macOS:** Uses CoreAudio
//! - **Linux:** Uses ALSA (install libasound2-dev)
//!
//! # Architecture
//!
//! ```text
//! Main Thread                    Audio Thread (RT)
//! ───────────                    ─────────────────
//! Build graph                    Scheduler::process_block()
//! Send commands ──────────────►  Drain commands
//!                                Process nodes
//!                                Write to output
//! ```

use aether_core::{
    command::Command, graph::DspGraph, node::DspNode, param::ParamBlock, scheduler::Scheduler,
    BUFFER_SIZE, MAX_INPUTS,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Producer, Split},
    HeapRb,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Simple sine wave oscillator for testing
struct SineOscillator {
    frequency: f32,
    phase: f32,
}

impl SineOscillator {
    fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
        }
    }
}

impl DspNode for SineOscillator {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let phase_inc = self.frequency / sample_rate;

        for sample in output.iter_mut() {
            *sample = (self.phase * std::f32::consts::TAU).sin() * 0.3;
            self.phase = (self.phase + phase_inc).fract();
        }
    }

    fn type_name(&self) -> &'static str {
        "SineOscillator"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 CPAL Integration Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Initialize CPAL
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;

    println!("🔊 Audio Device: {}", device.name()?);

    let config = device.default_output_config()?;
    println!("📊 Sample Rate: {} Hz", config.sample_rate().0);
    println!("🎚️  Channels: {}", config.channels());
    println!(
        "📦 Buffer Size: {} samples (aether uses {})",
        "varies", BUFFER_SIZE
    );
    println!();

    let sample_rate = config.sample_rate().0 as f32;

    // Create command ring buffer
    let (mut producer, consumer) = HeapRb::<Command>::new(1024).split();

    // Create scheduler
    let scheduler = Scheduler::new(sample_rate);
    let mut graph = DspGraph::new();

    // Build audio graph: oscillator → output
    println!("🔧 Building audio graph...");
    let osc = Box::new(SineOscillator::new(440.0)); // A4 note
    let osc_id = graph.add_node(osc).expect("Failed to add oscillator");
    graph.set_output_node(osc_id);

    // Send initial graph to scheduler
    producer.try_push(Command::AddNode { id: osc_id }).ok();
    producer
        .try_push(Command::SetOutputNode { id: osc_id })
        .ok();

    println!("  ✅ Oscillator (440 Hz) → Output");
    println!();

    // Shutdown flag
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        println!();
        println!("🛑 Shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Build audio stream
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(
            &device,
            &config.into(),
            scheduler,
            consumer,
            running.clone(),
        )?,
        cpal::SampleFormat::I16 => build_stream::<i16>(
            &device,
            &config.into(),
            scheduler,
            consumer,
            running.clone(),
        )?,
        cpal::SampleFormat::U16 => build_stream::<u16>(
            &device,
            &config.into(),
            scheduler,
            consumer,
            running.clone(),
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    // Start audio
    stream.play()?;
    println!("▶️  Audio started! Playing 440 Hz sine wave...");
    println!("   Press Ctrl+C to stop");
    println!();

    // Keep running until Ctrl+C
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Cleanup
    drop(stream);
    println!("✅ Shutdown complete");

    Ok(())
}

/// Build audio stream for a specific sample format
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut scheduler: Scheduler,
    mut consumer: ringbuf::HeapCons<Command>,
    running: Arc<AtomicBool>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            if !running.load(Ordering::SeqCst) {
                // Fill with silence when shutting down
                for sample in data.iter_mut() {
                    *sample = T::EQUILIBRIUM;
                }
                return;
            }

            // Process in 64-sample chunks
            let frames = data.len() / channels;
            let mut frame_offset = 0;

            while frame_offset < frames {
                let chunk_size = (frames - frame_offset).min(BUFFER_SIZE);

                // Allocate buffer for this chunk
                let mut output = vec![0.0f32; chunk_size * channels];

                // Process audio
                scheduler.process_block(&mut consumer, &mut output);

                // Copy to output buffer with sample format conversion
                for (i, &sample) in output.iter().enumerate() {
                    let idx = (frame_offset * channels) + i;
                    if idx < data.len() {
                        data[idx] = cpal::Sample::from_sample(sample);
                    }
                }

                frame_offset += chunk_size;
            }
        },
        move |err| {
            eprintln!("❌ Audio stream error: {}", err);
        },
        None,
    )?;

    Ok(stream)
}
