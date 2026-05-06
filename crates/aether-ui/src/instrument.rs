//! Per-track instrument engine.
//!
//! Each track gets an `InstrumentVoice` — a chain of DSP nodes:
//!   Oscillator → AdsrEnvelope → StateVariableFilter → Gain → (master mixer)
//!
//! The `TrackEngine` manages a pool of polyphonic voices (up to 8) and
//! handles note-on / note-off events from the UI or the step sequencer.

use std::sync::{Arc, Mutex};
use aether_core::{
    arena::NodeId,
    param::Param,
    scheduler::Scheduler,
    BUFFER_SIZE, MAX_INPUTS,
};
use aether_nodes::{
    oscillator::Oscillator,
    envelope::AdsrEnvelope,
    filter::StateVariableFilter,
    gain::Gain,
    mixer::Mixer,
};

// ── MIDI event ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum MidiEvent {
    NoteOn  { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
    AllNotesOff,
}

// ── Instrument preset ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InstrumentPreset {
    pub waveform:  f32,   // 0=sine 1=saw 2=square 3=tri
    pub attack:    f32,
    pub decay:     f32,
    pub sustain:   f32,
    pub release:   f32,
    pub cutoff:    f32,   // Hz
    pub resonance: f32,
    pub gain:      f32,
}

impl InstrumentPreset {
    pub fn kick() -> Self {
        Self { waveform: 0.0, attack: 0.001, decay: 0.18, sustain: 0.0, release: 0.05,
               cutoff: 200.0, resonance: 0.7, gain: 0.9 }
    }
    pub fn bass() -> Self {
        Self { waveform: 1.0, attack: 0.005, decay: 0.3, sustain: 0.6, release: 0.15,
               cutoff: 800.0, resonance: 1.2, gain: 0.75 }
    }
    pub fn lead() -> Self {
        Self { waveform: 2.0, attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.2,
               cutoff: 3000.0, resonance: 1.5, gain: 0.65 }
    }
    pub fn pad() -> Self {
        Self { waveform: 3.0, attack: 0.3, decay: 0.5, sustain: 0.8, release: 0.8,
               cutoff: 2000.0, resonance: 0.8, gain: 0.55 }
    }
    pub fn default_instrument() -> Self {
        Self { waveform: 1.0, attack: 0.01, decay: 0.2, sustain: 0.5, release: 0.3,
               cutoff: 2000.0, resonance: 1.0, gain: 0.7 }
    }
}

// ── Single voice ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Voice {
    pub osc_id:  NodeId,
    pub env_id:  NodeId,
    pub filt_id: NodeId,
    pub gain_id: NodeId,
    pub pitch:   Option<u8>,   // None = free
    pub active:  bool,
}

// ── Track engine ──────────────────────────────────────────────────────────────

pub struct TrackEngine {
    pub voices:    Vec<Voice>,
    pub mixer_id:  NodeId,
    pub preset:    InstrumentPreset,
    pub volume:    f32,
    pub pan:       f32,
    pub muted:     bool,
    voice_cursor:  usize,
}

const MAX_VOICES: usize = 8;

impl TrackEngine {
    /// Build a full polyphonic instrument chain in the scheduler's graph.
    /// Returns None if the graph is full.
    pub fn build(
        sched: &mut Scheduler,
        preset: InstrumentPreset,
        master_mixer_id: NodeId,
        master_slot: usize,
    ) -> Option<Self> {
        // Per-track mixer that sums all voices
        let mixer_id = sched.graph.add_node(Box::new(Mixer))?;

        let mut voices = Vec::with_capacity(MAX_VOICES);

        for v in 0..MAX_VOICES {
            let osc_id  = sched.graph.add_node(Box::new(Oscillator::new()))?;
            let env_id  = sched.graph.add_node(Box::new(AdsrEnvelope::new()))?;
            let filt_id = sched.graph.add_node(Box::new(StateVariableFilter::new()))?;
            let gain_id = sched.graph.add_node(Box::new(Gain))?;

            // Wire: osc → env (input 0 = audio to modulate)
            sched.graph.connect(osc_id, env_id, 0);
            // Wire: env → filter
            sched.graph.connect(env_id, filt_id, 0);
            // Wire: filter → gain
            sched.graph.connect(filt_id, gain_id, 0);
            // Wire: gain → track mixer slot v
            sched.graph.connect(gain_id, mixer_id, v);

            // Set initial params
            Self::apply_preset_to_voice(sched, osc_id, env_id, filt_id, gain_id, &preset, 0.0);

            // Gate off
            set_param(sched, env_id, 4, 0.0);

            voices.push(Voice { osc_id, env_id, filt_id, gain_id, pitch: None, active: false });
        }

        // Connect track mixer → master mixer
        sched.graph.connect(mixer_id, master_mixer_id, master_slot);

        Some(Self {
            voices,
            mixer_id,
            preset,
            volume: 0.8,
            pan: 0.0,
            muted: false,
            voice_cursor: 0,
        })
    }

    fn apply_preset_to_voice(
        sched: &mut Scheduler,
        osc_id: NodeId, env_id: NodeId, filt_id: NodeId, gain_id: NodeId,
        preset: &InstrumentPreset,
        freq: f32,
    ) {
        // Oscillator: freq, amp, waveform, midi_note(-1=use freq)
        set_param(sched, osc_id, 0, freq);
        set_param(sched, osc_id, 1, 1.0);
        set_param(sched, osc_id, 2, preset.waveform);
        set_param(sched, osc_id, 3, -1.0);

        // Envelope: A D S R gate
        set_param(sched, env_id, 0, preset.attack);
        set_param(sched, env_id, 1, preset.decay);
        set_param(sched, env_id, 2, preset.sustain);
        set_param(sched, env_id, 3, preset.release);

        // Filter: cutoff, resonance, mode(LP=0)
        set_param(sched, filt_id, 0, preset.cutoff);
        set_param(sched, filt_id, 1, preset.resonance);
        set_param(sched, filt_id, 2, 0.0);

        // Gain
        set_param(sched, gain_id, 0, preset.gain);
    }

    pub fn note_on(&mut self, sched: &mut Scheduler, pitch: u8, velocity: u8) {
        if self.muted { return; }

        // Steal oldest voice if all busy
        let v = self.voice_cursor % MAX_VOICES;
        self.voice_cursor += 1;

        let freq = midi_to_hz(pitch);
        let vel  = velocity as f32 / 127.0;
        let voice = &self.voices[v];

        Self::apply_preset_to_voice(
            sched, voice.osc_id, voice.env_id, voice.filt_id, voice.gain_id,
            &self.preset, freq,
        );
        set_param(sched, voice.gain_id, 0, self.preset.gain * vel * self.volume);
        // Trigger gate
        set_param(sched, voice.env_id, 4, 1.0);

        let voice = &mut self.voices[v];
        voice.pitch  = Some(pitch);
        voice.active = true;
    }

    pub fn note_off(&mut self, sched: &mut Scheduler, pitch: u8) {
        for voice in &mut self.voices {
            if voice.pitch == Some(pitch) && voice.active {
                set_param(sched, voice.env_id, 4, 0.0);
                voice.active = false;
                voice.pitch  = None;
            }
        }
    }

    pub fn all_notes_off(&mut self, sched: &mut Scheduler) {
        for voice in &mut self.voices {
            set_param(sched, voice.env_id, 4, 0.0);
            voice.active = false;
            voice.pitch  = None;
        }
    }

    pub fn set_volume(&mut self, sched: &mut Scheduler, vol: f32) {
        self.volume = vol;
        // Update gain on all voices
        for voice in &self.voices {
            set_param(sched, voice.gain_id, 0, self.preset.gain * vol);
        }
    }

    /// Update the instrument preset and apply to all idle voices
    pub fn update_preset(&mut self, sched: &mut Scheduler, preset: InstrumentPreset) {
        self.preset = preset;
        // Apply to all voices (they'll pick up new params on next note-on)
        for voice in &self.voices {
            if !voice.active {
                Self::apply_preset_to_voice(
                    sched, voice.osc_id, voice.env_id, voice.filt_id, voice.gain_id,
                    &self.preset, 440.0,
                );
            }
        }
    }
}

// ── Master engine ─────────────────────────────────────────────────────────────

/// Owns all track engines and the master mixer node.
pub struct MasterEngine {
    pub tracks:    Vec<Option<TrackEngine>>,
    pub master_id: NodeId,
}

impl MasterEngine {
    pub fn build(sched: &mut Scheduler, track_count: usize) -> Option<Self> {
        let master_id = sched.graph.add_node(Box::new(Mixer))?;
        sched.graph.set_output_node(master_id);

        let mut tracks = Vec::with_capacity(track_count);
        let presets = [
            InstrumentPreset::kick(),
            InstrumentPreset::bass(),
            InstrumentPreset::lead(),
            InstrumentPreset::pad(),
        ];

        for i in 0..track_count {
            let preset = presets.get(i).cloned().unwrap_or_else(InstrumentPreset::default_instrument);
            let engine = TrackEngine::build(sched, preset, master_id, i);
            tracks.push(engine);
        }

        Some(Self { tracks, master_id })
    }

    pub fn send_event(&mut self, sched: &mut Scheduler, track_idx: usize, event: MidiEvent) {
        if let Some(Some(engine)) = self.tracks.get_mut(track_idx) {
            match event {
                MidiEvent::NoteOn { pitch, velocity } => engine.note_on(sched, pitch, velocity),
                MidiEvent::NoteOff { pitch }          => engine.note_off(sched, pitch),
                MidiEvent::AllNotesOff                => engine.all_notes_off(sched),
            }
        }
    }

    pub fn ensure_track(&mut self, sched: &mut Scheduler, track_idx: usize) {
        while self.tracks.len() <= track_idx {
            self.tracks.push(None);
        }
        if self.tracks[track_idx].is_none() {
            let slot = track_idx;
            let engine = TrackEngine::build(
                sched,
                InstrumentPreset::default_instrument(),
                self.master_id,
                slot,
            );
            self.tracks[track_idx] = engine;
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn set_param(sched: &mut Scheduler, node_id: NodeId, param_idx: usize, value: f32) {
    if let Some(record) = sched.graph.arena.get_mut(node_id) {
        // Ensure param slot exists
        while record.params.count <= param_idx {
            record.params.add(0.0);
        }
        record.params.params[param_idx].current = value;
        record.params.params[param_idx].target  = value;
        record.params.params[param_idx].step    = 0.0;
    }
}

pub fn midi_to_hz(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}
