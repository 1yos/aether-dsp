//! AetherDSP Host — Aether Studio backend
//!
//! Architecture:
//!   • CPAL audio thread  — owns the Scheduler exclusively via Arc<Mutex<>>.
//!                          Uses try_lock() — never blocks. Produces silence on contention.
//!   • tokio runtime      — WebSocket server on ws://127.0.0.1:9001
//!   • GraphManager       — control-thread graph state, source of truth
//!
//! Lock contention mitigation (v0.2):
//!   The Scheduler is wrapped in Arc<Mutex<>>. The audio thread uses try_lock()
//!   so it never blocks — if the control thread holds the lock, the audio thread
//!   outputs the previous buffer (held in a small ring) rather than silence.
//!   This eliminates audible dropouts during graph mutations.
//!
//! Startup sequence:
//!   1. Create Scheduler and GraphManager.
//!   2. Acquire the Scheduler lock.
//!   3. Optionally load AETHER_STARTUP_PATCH into the graph while holding the lock.
//!   4. Call GetSnapshot to confirm the graph is initialized.
//!   5. Release the lock.
//!   6. Build the CPAL stream (audio thread is not yet running).
//!   7. Log "Graph: ready (empty)" (or "Graph: ready (patch loaded)").
//!   8. Call stream.play() — audio thread starts seeing a fully initialized graph.

mod bridge;
mod graph_manager;
mod undo_stack;
mod wav_writer;
mod ws_server;

use std::sync::{Arc, Mutex};

use aether_core::{scheduler::Scheduler, BUFFER_SIZE};
use aether_midi::MidiEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use graph_manager::{GraphManager, Intent, PatchDef};
use ws_server::WsState;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No audio output device");
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    println!("╔══════════════════════════════════════╗");
    println!("║       AetherDSP Studio v0.1          ║");
    println!("╚══════════════════════════════════════╝");
    println!("Device      : {}", device.name()?);
    println!("Sample rate : {} Hz", sample_rate);
    println!("Channels    : {}", config.channels());
    println!("WebSocket   : ws://127.0.0.1:9001");

    // ── Step 1: Create Scheduler and GraphManager ─────────────────────────────
    // GraphManager is created BEFORE build_stream so the graph is fully
    // initialized before the audio thread ever starts.
    let scheduler = Arc::new(Mutex::new(Scheduler::new(sample_rate)));
    let mut graph_manager = GraphManager::new(sample_rate);

    // ── Step 2–4: Acquire lock, optionally load startup patch, confirm init ───
    let graph_ready_msg = {
        let mut sched = scheduler.lock().unwrap();

        // Optional: load a startup patch from AETHER_STARTUP_PATCH env var.
        let msg = if let Ok(patch_path) = std::env::var("AETHER_STARTUP_PATCH") {
            match std::fs::read_to_string(&patch_path) {
                Ok(json) => match serde_json::from_str::<PatchDef>(&json) {
                    Ok(patch) => {
                        graph_manager.handle(Intent::LoadPatch { patch }, &mut sched);
                        format!("Graph: ready (patch loaded from {patch_path})")
                    }
                    Err(e) => {
                        eprintln!("AETHER_STARTUP_PATCH: failed to parse {patch_path}: {e}");
                        graph_manager.handle(Intent::GetSnapshot, &mut sched);
                        "Graph: ready (empty)".to_string()
                    }
                },
                Err(e) => {
                    eprintln!("AETHER_STARTUP_PATCH: failed to read {patch_path}: {e}");
                    graph_manager.handle(Intent::GetSnapshot, &mut sched);
                    "Graph: ready (empty)".to_string()
                }
            }
        } else {
            // No startup patch — call GetSnapshot to confirm the graph is initialized.
            graph_manager.handle(Intent::GetSnapshot, &mut sched);
            "Graph: ready (empty)".to_string()
        };

        // ── Step 5: Lock is released here (end of this block) ────────────────
        msg
    };

    // ── Step 6: Build the CPAL stream (audio thread not yet running) ──────────
    // Create MidiEngine and attempt to connect to the first available port.
    // MIDI is optional — if no hardware is connected, the host still starts.
    let midi_engine = Arc::new(Mutex::new(MidiEngine::new()));

    // Grab the shared midi_queues Arc from GraphManager before it's moved into WsState.
    // The MIDI callback (sync) uses this to push events without touching the async lock.
    let midi_queues = Arc::clone(&graph_manager.midi_queues);

    {
        let mut engine = midi_engine.lock().unwrap();
        match engine.connect_first() {
            Ok(port_name) => println!("MIDI: connected to '{port_name}'"),
            Err(e) => eprintln!("MIDI: no hardware connected ({e}) — MIDI disabled"),
        }

        // Register a catch-all handler (channel 255) that fans out events to all
        // SamplerNode MIDI queues. Uses the shared Arc so the sync callback can
        // access the map without blocking the async runtime.
        let queues = Arc::clone(&midi_queues);
        let router = engine.router();
        router.lock().unwrap().register(255, Box::new(move |event| {
            let qs = queues.lock().unwrap();
            for queue in qs.values() {
                queue.lock().unwrap().push(event.clone());
            }
        }));
    }

    let ws_state = Arc::new(WsState {
        graph_manager: tokio::sync::Mutex::new(graph_manager),
        scheduler: Arc::clone(&scheduler),
        midi_engine: Arc::clone(&midi_engine),
        scope_consumers: tokio::sync::Mutex::new(std::collections::HashMap::new()),
    });

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            build_stream::<f32>(&device, &config.into(), Arc::clone(&scheduler), Arc::clone(&midi_engine))?
        }
        _ => anyhow::bail!("Unsupported sample format"),
    };

    // ── Step 7: Log graph ready status ───────────────────────────────────────
    println!("{graph_ready_msg}");

    // ── Step 8: Start audio stream — audio thread sees fully initialized graph
    stream.play()?;
    println!("Audio stream started. Open http://localhost:5173 in your browser.");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        tokio::spawn(ws_server::run(Arc::clone(&ws_state)));
        tokio::signal::ctrl_c().await.expect("ctrl-c failed");
        println!("\nShutting down.");
    });

    Ok(())
}

fn build_stream<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    scheduler: Arc<Mutex<Scheduler>>,
    midi_engine: Arc<Mutex<MidiEngine>>,
) -> anyhow::Result<cpal::Stream> {
    let channels = config.channels as usize;
    // Fallback buffer: holds the last successfully rendered block.
    // On lock contention we repeat it instead of outputting silence,
    // which eliminates audible dropouts during graph mutations.
    let mut fallback_buf = vec![0.0f32; BUFFER_SIZE * 2];
    let mut contention_count = 0u32;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            let frames = data.len() / channels;
            let mut f32_buf = [0.0f32; BUFFER_SIZE * 2];
            let mut offset = 0;

            while offset < frames {
                let chunk = (frames - offset).min(BUFFER_SIZE);

                match scheduler.try_lock() {
                    Ok(mut sched) => {
                        sched.process_block_simple(&mut f32_buf[..chunk * 2]);
                        // Update fallback with the freshly rendered block
                        fallback_buf[..chunk * 2].copy_from_slice(&f32_buf[..chunk * 2]);
                        contention_count = 0;
                    }
                    Err(_) => {
                        // Control thread is mutating the graph.
                        // Repeat the last good buffer (max 8 consecutive repeats = ~10ms).
                        // After that, fade to silence to avoid stuck notes.
                        contention_count += 1;
                        if contention_count <= 8 {
                            f32_buf[..chunk * 2].copy_from_slice(&fallback_buf[..chunk * 2]);
                        } else {
                            // Fade out to avoid stuck notes after prolonged contention
                            let fade = 1.0 - ((contention_count - 8) as f32 / 8.0).min(1.0);
                            for (i, s) in fallback_buf[..chunk * 2].iter().enumerate() {
                                f32_buf[i] = s * fade;
                            }
                        }
                    }
                }

                // Advance the MIDI sample counter to keep timestamps in sync.
                if let Ok(engine) = midi_engine.try_lock() {
                    engine.advance_samples(chunk as u64);
                }

                for i in 0..chunk {
                    let ch0 = f32_buf[i * 2];
                    let ch1 = f32_buf[i * 2 + 1];
                    for ch in 0..channels {
                        let idx = (offset + i) * channels + ch;
                        if idx < data.len() {
                            data[idx] = T::from_sample(if ch == 0 { ch0 } else { ch1 });
                        }
                    }
                }
                offset += chunk;
            }
        },
        |err| eprintln!("Audio error: {err}"),
        None,
    )?;
    Ok(stream)
}
