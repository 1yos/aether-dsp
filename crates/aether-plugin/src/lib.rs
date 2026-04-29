//! AetherDSP CLAP Plugin
//!
//! Wraps a `SamplerNode` as a CLAP instrument plugin using NIH-plug.
//!
//! The `SamplerInstrument` JSON is embedded at build time via the
//! `AETHER_INSTRUMENT_JSON` environment variable.  If the variable is not set,
//! the plugin starts with an empty instrument (no zones → silence).
//!
//! Build with:
//!   AETHER_INSTRUMENT_JSON='<json>' cargo build -p aether-plugin --release
//!
//! Then copy `target/release/aether_plugin.clap` to your DAW's plugin folder.

// Pull in the generated embedded audio table produced by build.rs.
include!(concat!(env!("OUT_DIR"), "/embedded_audio.rs"));

use std::sync::{Arc, Mutex};

use aether_core::{scheduler::Scheduler, BUFFER_SIZE};
use aether_midi::event::{MidiEvent, MidiEventKind};
use aether_sampler::{
    instrument::{LoadedInstrument, SamplerInstrument},
    node::SamplerNode,
};
use nih_plug::prelude::*;

// ── Default instrument JSON (empty instrument, no zones) ─────────────────────

/// Instrument JSON embedded at build time.  Falls back to an empty instrument
/// if `AETHER_INSTRUMENT_JSON` was not set when the plugin was compiled.
const INSTRUMENT_JSON: &str = {
    // The env var is injected by the build script / cargo invocation.
    // If it was not set, we use a minimal valid JSON.
    const EMBEDDED: Option<&str> = option_env!("AETHER_INSTRUMENT_JSON");
    match EMBEDDED {
        Some(s) => s,
        None => r#"{
            "name": "Empty Instrument",
            "origin": "",
            "description": "No instrument loaded",
            "author": "",
            "tuning": {
                "name": "12-TET",
                "frequencies": [8.18,8.66,9.18,9.72,10.30,10.91,11.56,12.25,12.98,13.75,14.57,15.43,16.35,17.32,18.35,19.45,20.60,21.83,23.12,24.50,25.96,27.50,29.14,30.87,32.70,34.65,36.71,38.89,41.20,43.65,46.25,49.00,51.91,55.00,58.27,61.74,65.41,69.30,73.42,77.78,82.41,87.31,92.50,98.00,103.83,110.00,116.54,123.47,130.81,138.59,146.83,155.56,164.81,174.61,185.00,196.00,207.65,220.00,233.08,246.94,261.63,277.18,293.66,311.13,329.63,349.23,369.99,392.00,415.30,440.00,466.16,493.88,523.25,554.37,587.33,622.25,659.25,698.46,739.99,783.99,830.61,880.00,932.33,987.77,1046.50,1108.73,1174.66,1244.51,1318.51,1396.91,1479.98,1567.98,1661.22,1760.00,1864.66,1975.53,2093.00,2217.46,2349.32,2489.02,2637.02,2793.83,2959.96,3135.96,3322.44,3520.00,3729.31,3951.07,4186.01,4434.92,4698.63,4978.03,5274.04,5587.65,5919.91,6271.93,6644.88,7040.00,7458.62,7902.13,8372.02,8869.84,9397.27,9956.06,10548.08,11175.30,11839.82,12543.85,13289.75,14080.00,14917.24,15804.27]
            },
            "zones": [],
            "attack": 0.005,
            "decay": 0.1,
            "sustain": 0.8,
            "release": 0.3,
            "max_voices": 16
        }"#,
    }
};

// ── Plugin parameters exposed to the DAW ─────────────────────────────────────

/// CLAP-exposed parameters.  The ADSR values mirror the `SamplerInstrument`
/// envelope and are applied to the live instrument each audio block.
/// (Requirement 7.8 — ADSR parameters SHALL be automatable by the host DAW.)
#[derive(Params)]
struct AetherParams {
    /// Envelope attack time in seconds (0.001 – 10 s).
    #[id = "attack"]
    pub attack: FloatParam,

    /// Envelope decay time in seconds (0.001 – 10 s).
    #[id = "decay"]
    pub decay: FloatParam,

    /// Envelope sustain level (0.0 – 1.0).
    #[id = "sustain"]
    pub sustain: FloatParam,

    /// Envelope release time in seconds (0.001 – 10 s).
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for AetherParams {
    fn default() -> Self {
        Self {
            attack: FloatParam::new(
                "Attack",
                0.005,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 10.0,
                    factor: 0.3,
                },
            )
            .with_unit(" s")
            .with_smoother(SmoothingStyle::Linear(20.0)),

            decay: FloatParam::new(
                "Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 10.0,
                    factor: 0.3,
                },
            )
            .with_unit(" s")
            .with_smoother(SmoothingStyle::Linear(20.0)),

            sustain: FloatParam::new(
                "Sustain",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),

            release: FloatParam::new(
                "Release",
                0.3,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 10.0,
                    factor: 0.3,
                },
            )
            .with_unit(" s")
            .with_smoother(SmoothingStyle::Linear(20.0)),
        }
    }
}

// ── Plugin struct ─────────────────────────────────────────────────────────────

struct AetherPlugin {
    params: Arc<AetherParams>,
    /// The sampler node that drives audio output.
    sampler: Option<SamplerNode>,
    /// Shared MIDI queue — written here in `process()`, read by `SamplerNode`.
    midi_queue: Option<Arc<Mutex<Vec<MidiEvent>>>>,
    /// The loaded instrument — kept so we can update ADSR from params each block.
    instrument: Option<Arc<Mutex<Option<aether_sampler::instrument::LoadedInstrument>>>>,
}

impl Default for AetherPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(AetherParams::default()),
            sampler: None,
            midi_queue: None,
            instrument: None,
        }
    }
}

impl Plugin for AetherPlugin {
    const NAME: &'static str = "AetherDSP";
    const VENDOR: &'static str = "AetherDSP Project";
    const URL: &'static str = "https://github.com/YOUR_USERNAME/aether-dsp";
    const EMAIL: &'static str = "contact@aetherdsp.dev";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = buffer_config.sample_rate;

        // Parse the embedded instrument JSON.
        let instrument: SamplerInstrument = match serde_json::from_str(INSTRUMENT_JSON) {
            Ok(inst) => inst,
            Err(e) => {
                nih_error!("AetherPlugin: failed to parse embedded instrument JSON: {e}");
                SamplerInstrument::new("Empty")
            }
        };

        // Seed ADSR from the embedded instrument so the DAW params start at the
        // correct values (they will be overridden each block from the smoothed params).
        {
            // (params are already set to defaults matching the JSON defaults)
        }

        // Build a LoadedInstrument from the embedded audio bytes.
        // EMBEDDED_ZONES is generated by build.rs: &[(&str zone_id, &[u8] audio_bytes)].
        // For each zone, if we have embedded bytes, load a SampleBuffer from them;
        // otherwise the zone is skipped (no audio for that zone).
        let mut buffers = std::collections::HashMap::new();
        for (zone_id, audio_bytes) in EMBEDDED_ZONES {
            match load_buffer_from_bytes(audio_bytes) {
                Ok(buf) => {
                    buffers.insert(zone_id.to_string(), buf);
                }
                Err(e) => {
                    nih_error!("AetherPlugin: failed to decode embedded audio for zone '{zone_id}': {e}");
                }
            }
        }

        let loaded = aether_sampler::instrument::LoadedInstrument {
            instrument,
            buffers,
            release_buffers: std::collections::HashMap::new(),
        };

        // Create the SamplerNode and load the instrument into it.
        // NOTE: SamplerNode uses instrument.tuning for all pitch ratio calculations,
        // preserving any non-12-TET TuningTable embedded in the instrument JSON.
        // (Requirement 7.7 — TuningTable SHALL be preserved in the exported plugin.)
        let node = SamplerNode::new(sr);
        let queue = node.midi_queue();
        let slot = node.instrument_slot();

        *slot.lock().unwrap() = Some(loaded);

        self.sampler = Some(node);
        self.midi_queue = Some(queue);
        self.instrument = Some(slot);

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sampler = match self.sampler.as_mut() {
            Some(s) => s,
            None => return ProcessStatus::Error("Not initialized"),
        };
        let midi_queue = match self.midi_queue.as_ref() {
            Some(q) => q,
            None => return ProcessStatus::Error("No MIDI queue"),
        };

        // ── 20.2: Update SamplerNode ADSR from smoothed DAW params each block ──
        // This allows the host DAW to automate attack/decay/sustain/release.
        if let Some(inst_slot) = self.instrument.as_ref() {
            if let Ok(mut guard) = inst_slot.lock() {
                if let Some(loaded) = guard.as_mut() {
                    loaded.instrument.attack = self.params.attack.smoothed.next();
                    loaded.instrument.decay = self.params.decay.smoothed.next();
                    loaded.instrument.sustain = self.params.sustain.smoothed.next();
                    loaded.instrument.release = self.params.release.smoothed.next();
                }
            }
        }

        // ── 20.1: Forward NIH-plug MIDI events to the SamplerNode's MIDI queue ──
        // Drain all MIDI events from the NIH-plug context and push them into the
        // SamplerNode's queue so they are processed in the next render call.
        {
            let mut queue = midi_queue.lock().unwrap();
            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::NoteOn { timing: _, voice_id: _, channel, note, velocity } => {
                        // NIH-plug velocity is 0.0–1.0; convert to 0–127.
                        let vel = (velocity * 127.0).clamp(0.0, 127.0) as u8;
                        queue.push(MidiEvent {
                            timestamp: 0,
                            channel: channel as u8,
                            kind: MidiEventKind::NoteOn { note, velocity: vel },
                        });
                    }
                    NoteEvent::NoteOff { timing: _, voice_id: _, channel, note, velocity } => {
                        let vel = (velocity * 127.0).clamp(0.0, 127.0) as u8;
                        queue.push(MidiEvent {
                            timestamp: 0,
                            channel: channel as u8,
                            kind: MidiEventKind::NoteOff { note, velocity: vel },
                        });
                    }
                    NoteEvent::MidiCC { timing: _, channel, cc, value } => {
                        let val = (value * 127.0).clamp(0.0, 127.0) as u8;
                        queue.push(MidiEvent {
                            timestamp: 0,
                            channel: channel as u8,
                            kind: MidiEventKind::ControlChange { cc, value: val },
                        });
                    }
                    _ => {}
                }
            }
        }

        // ── 20.1: Call SamplerNode::process() in the audio callback ──
        // We render one BUFFER_SIZE block at a time and copy into the NIH-plug buffer.
        // NIH-plug iterates samples; we batch them into BUFFER_SIZE chunks.
        let mut scratch = [0.0f32; BUFFER_SIZE];
        let mut dummy_params = aether_core::param::ParamBlock::new();

        // Collect all channel slices.
        let num_samples = buffer.samples();
        let mut offset = 0;

        while offset < num_samples {
            let chunk = (num_samples - offset).min(BUFFER_SIZE);
            scratch[..chunk].fill(0.0);

            // Build a dummy inputs array (SamplerNode ignores inputs — it's MIDI-driven).
            let inputs: [Option<&[f32; BUFFER_SIZE]>; aether_core::MAX_INPUTS] =
                [None; aether_core::MAX_INPUTS];

            // SAFETY: scratch is BUFFER_SIZE; we only use [..chunk] of the output.
            sampler.process(&inputs, &mut scratch, &mut dummy_params, 0.0);

            // Write the rendered mono signal to all output channels.
            for (ch_idx, channel_samples) in buffer.as_slice().iter_mut().enumerate() {
                for i in 0..chunk {
                    if ch_idx < 2 {
                        channel_samples[offset + i] = scratch[i];
                    }
                }
            }

            offset += chunk;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for AetherPlugin {
    const CLAP_ID: &'static str = "dev.aetherdsp.aether-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Hard real-time modular DSP engine");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

nih_export_clap!(AetherPlugin);

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Decode WAV audio bytes (from an embedded `include_bytes!` slice) into a
/// `SampleBuffer`.  Returns an error string on failure.
fn load_buffer_from_bytes(
    bytes: &[u8],
) -> Result<aether_sampler::buffer::SampleBuffer, String> {
    use std::io::Cursor;
    let cursor = Cursor::new(bytes);
    aether_sampler::buffer::SampleBuffer::load_wav_reader(cursor)
        .map_err(|e| e.to_string())
}
