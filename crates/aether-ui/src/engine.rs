//! Audio engine bridge — connects the Iced UI to the AetherDSP real-time engine.
//!
//! Architecture:
//!
//! ```text
//! Iced UI (main thread)
//!     ↓  engine.play_note() / stop_note() / set_track_volume()
//! Engine (control thread)
//!     ↓  Scheduler lock  (Arc<Mutex<Scheduler>>)
//! CPAL audio thread
//!     ↓  process_block_simple() → DAC
//! ```
//!
//! Each Track in the UI maps to this internal node chain:
//!
//! ```text
//! Oscillator (sine, tuning-aware)
//!     ↓  slot 0
//! AdsrEnvelope
//!     ↓  slot 0
//! Gain  (track volume)
//!     ↓  slot 0
//! Master Mixer output node
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use aether_core::arena::NodeId;
use aether_core::param::Param;
use aether_core::scheduler::Scheduler;
use aether_midi::tuning::TuningTable;
use aether_nodes::{envelope::AdsrEnvelope, gain::Gain, oscillator::Oscillator};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::project::{TrackId, TuningSystem};

// ── Per-track node IDs ────────────────────────────────────────────────────────

/// The three nodes that represent one instrument track in the DSP graph.
#[derive(Debug, Clone, Copy)]
struct TrackNodes {
    /// Oscillator — generates audio at the tuned frequency.
    /// Params: 0=freq, 1=amplitude, 2=waveform, 3=midi_note
    oscillator: NodeId,
    /// ADSR envelope — shapes the note's amplitude over time.
    /// Params: 0=attack, 1=decay, 2=sustain, 3=release, 4=gate
    envelope: NodeId,
    /// Gain — controls track volume.
    /// Params: 0=gain (0..1)
    gain: NodeId,
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// The audio engine bridge.
///
/// Owns the CPAL stream and provides a simple API for the UI to control
/// the DSP graph without knowing anything about nodes or schedulers.
pub struct Engine {
    scheduler: Arc<Mutex<Scheduler>>,
    sample_rate: f32,
    /// Maps UI TrackId → DSP node IDs
    tracks: HashMap<TrackId, TrackNodes>,
    /// The _stream must stay alive for audio to play (CPAL requirement).
    _stream: cpal::Stream,
}

impl Engine {
    /// Start the audio engine. Returns an error if no audio device is available.
    pub fn start() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;

        let scheduler = Arc::new(Mutex::new(Scheduler::new(sample_rate)));

        let stream = {
            let scheduler = Arc::clone(&scheduler);
            let channels = config.channels() as usize;
            let mut fallback = vec![0.0f32; aether_core::BUFFER_SIZE * 2];

            device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let frames = data.len() / channels;
                    let mut buf = [0.0f32; aether_core::BUFFER_SIZE * 2];
                    let mut offset = 0;

                    while offset < frames {
                        let chunk = (frames - offset).min(aether_core::BUFFER_SIZE);
                        match scheduler.try_lock() {
                            Ok(mut sched) => {
                                sched.process_block_simple(&mut buf[..chunk * 2]);
                                fallback[..chunk * 2].copy_from_slice(&buf[..chunk * 2]);
                            }
                            Err(_) => {
                                buf[..chunk * 2].copy_from_slice(&fallback[..chunk * 2]);
                            }
                        }
                        for i in 0..chunk {
                            let ch0 = buf[i * 2];
                            let ch1 = buf[i * 2 + 1];
                            for ch in 0..channels {
                                let idx = (offset + i) * channels + ch;
                                if idx < data.len() {
                                    data[idx] = if ch == 0 { ch0 } else { ch1 };
                                }
                            }
                        }
                        offset += chunk;
                    }
                },
                |err| eprintln!("[engine] audio error: {err}"),
                None,
            )?
        };

        stream.play()?;

        println!("[engine] started at {sample_rate} Hz");

        Ok(Self {
            scheduler,
            sample_rate,
            tracks: HashMap::new(),
            _stream: stream,
        })
    }

    // ── Track management ──────────────────────────────────────────────────────

    /// Add a new instrument track to the DSP graph.
    ///
    /// Wires up: Oscillator → AdsrEnvelope → Gain → Output.
    /// The oscillator is configured with the track's tuning system.
    pub fn add_track(&mut self, id: TrackId, tuning: TuningSystem, volume: f32) {
        let mut sched = self.scheduler.lock().unwrap();

        // Build the tuning table for this track's qenet
        let table = build_tuning_table(tuning);

        // Create oscillator with tuning
        let mut osc = Oscillator::new();
        osc.set_tuning(table.frequencies.try_into().unwrap_or([440.0f32; 128]));
        let osc_id = match sched.graph.add_node(Box::new(osc)) {
            Some(id) => id,
            None => {
                eprintln!("[engine] failed to add oscillator node");
                return;
            }
        };
        // Oscillator params: freq=440, amplitude=0.3, waveform=0 (sine), midi_note=-1
        {
            let rec = sched.graph.arena.get_mut(osc_id).unwrap();
            rec.params.add(440.0);   // 0: frequency
            rec.params.add(0.3);     // 1: amplitude
            rec.params.add(0.0);     // 2: waveform (sine)
            rec.params.add(-1.0);    // 3: midi_note (-1 = use freq param)
        }

        // Create ADSR envelope
        let env_id = match sched.graph.add_node(Box::new(AdsrEnvelope::new())) {
            Some(id) => id,
            None => {
                eprintln!("[engine] failed to add envelope node");
                return;
            }
        };
        // Envelope params: attack, decay, sustain, release, gate
        {
            let rec = sched.graph.arena.get_mut(env_id).unwrap();
            rec.params.add(0.005);   // 0: attack  (5ms)
            rec.params.add(0.1);     // 1: decay   (100ms)
            rec.params.add(0.7);     // 2: sustain (70%)
            rec.params.add(0.15);    // 3: release (150ms)
            rec.params.add(0.0);     // 4: gate    (off)
        }

        // Create gain node
        let gain_id = match sched.graph.add_node(Box::new(Gain)) {
            Some(id) => id,
            None => {
                eprintln!("[engine] failed to add gain node");
                return;
            }
        };
        {
            let rec = sched.graph.arena.get_mut(gain_id).unwrap();
            rec.params.add(volume);  // 0: gain
        }

        // Wire: oscillator → envelope → gain
        sched.graph.connect(osc_id, env_id, 0);
        sched.graph.connect(env_id, gain_id, 0);

        // Set this track's gain as the output node
        // (simple approach for now — last track wins; mixer comes in Step 10)
        sched.graph.set_output_node(gain_id);

        self.tracks.insert(id, TrackNodes {
            oscillator: osc_id,
            envelope: env_id,
            gain: gain_id,
        });

        println!("[engine] track {:?} added ({:?})", id.0, tuning);
    }

    /// Remove a track and all its nodes from the DSP graph.
    pub fn remove_track(&mut self, id: TrackId) {
        if let Some(nodes) = self.tracks.remove(&id) {
            let mut sched = self.scheduler.lock().unwrap();
            sched.graph.remove_node(nodes.gain);
            sched.graph.remove_node(nodes.envelope);
            sched.graph.remove_node(nodes.oscillator);
            println!("[engine] track {:?} removed", id.0);
        }
    }

    // ── Note control ──────────────────────────────────────────────────────────

    /// Trigger a note on a track.
    ///
    /// Sets the oscillator frequency from the tuning table and opens the ADSR gate.
    pub fn note_on(&mut self, track_id: TrackId, midi_note: u8) {
        let nodes = match self.tracks.get(&track_id) {
            Some(n) => *n,
            None => return,
        };

        let mut sched = self.scheduler.lock().unwrap();

        // Set midi_note param on oscillator (param 3) — the oscillator's tuning
        // table will convert this to the correct frequency automatically.
        if let Some(rec) = sched.graph.arena.get_mut(nodes.oscillator) {
            rec.params.params[3] = Param::new(midi_note as f32);
        }

        // Open the ADSR gate (param 4 → 1.0)
        if let Some(rec) = sched.graph.arena.get_mut(nodes.envelope) {
            rec.params.params[4] = Param::new(1.0);
        }
    }

    /// Release a note on a track (close the ADSR gate).
    pub fn note_off(&mut self, track_id: TrackId) {
        let nodes = match self.tracks.get(&track_id) {
            Some(n) => *n,
            None => return,
        };

        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.envelope) {
            rec.params.params[4] = Param::new(0.0);
        }
    }

    // ── Track parameters ──────────────────────────────────────────────────────

    /// Set track volume (0.0 - 1.0). Applied with a short ramp to avoid clicks.
    pub fn set_track_volume(&mut self, track_id: TrackId, volume: f32) {
        let nodes = match self.tracks.get(&track_id) {
            Some(n) => *n,
            None => return,
        };
        let ramp = (self.sample_rate * 0.010) as u32; // 10ms ramp
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.gain) {
            rec.params.params[0].set_target(volume.clamp(0.0, 1.0), ramp);
        }
    }

    /// Mute or unmute a track.
    pub fn set_track_mute(&mut self, track_id: TrackId, muted: bool) {
        let volume = if muted { 0.0 } else { 0.75 };
        self.set_track_volume(track_id, volume);
    }

    /// Update a track's tuning system (replaces the oscillator's tuning table).
    pub fn set_track_tuning(&mut self, track_id: TrackId, tuning: TuningSystem) {
        let nodes = match self.tracks.get(&track_id) {
            Some(n) => *n,
            None => return,
        };
        let table = build_tuning_table(tuning);
        let freqs: [f32; 128] = table.frequencies.try_into().unwrap_or([440.0f32; 128]);

        // We need to access the oscillator processor to call set_tuning().
        // We do this via a downcast — the oscillator is inside a Box<dyn DspNode>.
        // Rather than unsafe downcasting, we rebuild the oscillator node.
        // This causes a brief click — acceptable since tuning changes are manual.
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.oscillator) {
            // Safe: we cast to Oscillator which is the concrete type behind the trait object.
            // The type_name check confirms the type before we do anything.
            if rec.processor.type_name() == "Oscillator" {
                // SAFETY: We verified the concrete type via type_name().
                // The oscillator pointer is exclusively owned by this NodeRecord.
                let osc_ptr = &mut *rec.processor as *mut dyn aether_core::node::DspNode;
                let osc = unsafe { &mut *(osc_ptr as *mut Oscillator) };
                osc.set_tuning(freqs);
            }
        }
    }

    // ── Global transport ──────────────────────────────────────────────────────

    /// Mute all audio output (transport stop).
    pub fn set_mute(&self, muted: bool) {
        let mut sched = self.scheduler.lock().unwrap();
        sched.muted = muted;
    }

    /// Returns true if the engine is currently running (always true after start).
    pub fn is_running(&self) -> bool {
        true
    }

    /// Returns the detected sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

// ── Tuning table builder ──────────────────────────────────────────────────────

/// Convert a TuningSystem enum value into a TuningTable with all 128 frequencies.
pub fn build_tuning_table(tuning: TuningSystem) -> TuningTable {
    use aether_midi::tuning::TuningTable as MidiTuning;

    match tuning {
        TuningSystem::EthiopianTizitaMajor  => MidiTuning::ethiopian_tizita(440.0),
        TuningSystem::EthiopianTizitaMinor  => MidiTuning::ethiopian_tizita_minor(440.0),
        TuningSystem::EthiopianBatiMinor    => MidiTuning::ethiopian_bati(440.0),
        TuningSystem::EthiopianBatiMajor    => MidiTuning::ethiopian_bati_major(440.0),
        TuningSystem::EthiopianAmbassel     => MidiTuning::ethiopian_ambassel(440.0),
        TuningSystem::EthiopianAnchihoye    => MidiTuning::ethiopian_anchihoye(440.0),
        TuningSystem::ArabicRast            => MidiTuning::arabic_maqam_rast(440.0),
        TuningSystem::ArabicBayati          => MidiTuning::arabic_maqam_bayati(440.0),
        TuningSystem::ArabicHijaz           => MidiTuning::arabic_maqam_hijaz(440.0),
        TuningSystem::IndianYaman           => MidiTuning::indian_raga_yaman(440.0),
        TuningSystem::GamelanSlendro        => MidiTuning::gamelan_slendro(440.0),
        TuningSystem::GamelanPelog          => MidiTuning::gamelan_pelog(440.0),
        TuningSystem::EqualTemperament      => MidiTuning::equal_temperament(440.0),
        TuningSystem::JustIntonation        => MidiTuning::just_intonation(440.0),
    }
}
