//! Per-track instrument engine.
//!
//! Each track gets an `InstrumentVoice` — a chain of DSP nodes:
//!   Oscillator → AdsrEnvelope → StateVariableFilter → Gain → (master mixer)
//!
//! The `TrackEngine` manages a pool of polyphonic voices (up to 8) and
//! handles note-on / note-off events from the UI or the step sequencer.

use aether_core::{
    arena::NodeId,
    node::DspNode,
    scheduler::Scheduler,
};
use aether_nodes::{
    oscillator::Oscillator,
    envelope::AdsrEnvelope,
    filter::StateVariableFilter,
    gain::Gain,
    mixer::Mixer,
    compressor::Compressor,
    reverb::Reverb,
    delay::DelayLine,
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
    /// Kick drum - Uses sine wave with very low filter for sub-bass punch.
    /// NOTE: For realistic kick, this needs pitch envelope (150Hz → 50Hz over 80ms).
    /// Current implementation uses fixed pitch, so it sounds more like a bass tone.
    /// TODO: Add pitch envelope support to oscillator for proper kick synthesis.
    pub fn kick() -> Self {
        Self { 
            waveform: 0.0,      // Sine wave for clean sub-bass
            attack: 0.001,      // Instant attack (1ms)
            decay: 0.15,        // Short decay (150ms) 
            sustain: 0.0,       // No sustain (one-shot)
            release: 0.05,      // Quick release (50ms)
            cutoff: 120.0,      // Very low cutoff for sub-bass
            resonance: 1.5,     // Boost sub frequencies
            gain: 1.0           // Full gain for punch
        }
    }
    
    /// Bass - Sawtooth wave with moderate filter, classic analog bass sound.
    pub fn bass() -> Self {
        Self { 
            waveform: 1.0,      // Sawtooth for rich harmonics
            attack: 0.005,      // Quick attack (5ms)
            decay: 0.3,         // Medium decay (300ms)
            sustain: 0.6,       // 60% sustain level
            release: 0.15,      // Short release (150ms)
            cutoff: 800.0,      // Low-mid filter for warmth
            resonance: 1.2,     // Slight resonance for character
            gain: 0.75          // Moderate gain
        }
    }
    
    /// Lead - Square wave with bright filter, cutting through the mix.
    pub fn lead() -> Self {
        Self { 
            waveform: 2.0,      // Square wave for hollow, bright tone
            attack: 0.01,       // Quick attack (10ms)
            decay: 0.08,        // Short decay (80ms)
            sustain: 0.75,      // High sustain (75%)
            release: 0.2,       // Medium release (200ms)
            cutoff: 4000.0,     // Bright filter for presence
            resonance: 2.0,     // Strong resonance for character
            gain: 0.7           // Moderate gain
        }
    }
    
    /// Pad - Triangle wave with slow attack, smooth and atmospheric.
    /// NOTE: Sounds thin without unison/detuning. Consider adding chorus effect.
    pub fn pad() -> Self {
        Self { 
            waveform: 3.0,      // Triangle for smooth, mellow tone
            attack: 0.4,        // Slow attack for pad swell (400ms)
            decay: 0.6,         // Long decay (600ms)
            sustain: 0.85,      // High sustain (85%)
            release: 1.2,       // Long release for tail (1.2s)
            cutoff: 2500.0,     // Mid-bright filter
            resonance: 1.2,     // Moderate resonance for warmth
            gain: 0.6           // Lower gain (pads sit in background)
        }
    }
    
    /// Default instrument - Balanced sawtooth synth, good starting point.
    pub fn default_instrument() -> Self {
        Self { 
            waveform: 1.0,      // Sawtooth
            attack: 0.01,       // Quick attack
            decay: 0.2,         // Medium decay
            sustain: 0.5,       // 50% sustain
            release: 0.3,       // Medium release
            cutoff: 2000.0,     // Mid-range filter
            resonance: 1.0,     // Neutral resonance
            gain: 0.7           // Moderate gain
        }
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

// ── Effect node ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Compressor,
    Reverb,
    Delay,
    Filter,
    Eq,
}

#[derive(Debug, Clone)]
pub struct EffectNode {
    pub id: u64,
    pub node_id: NodeId,
    pub effect_type: EffectType,
    pub enabled: bool,
}

// ── Track engine ──────────────────────────────────────────────────────────────

pub struct TrackEngine {
    pub voices:    Vec<Voice>,
    pub mixer_id:  NodeId,
    pub effects:   Vec<EffectNode>,
    pub preset:    InstrumentPreset,
    pub volume:    f32,
    pub pan:       f32,
    pub muted:     bool,
    voice_cursor:  usize,
    master_mixer_id: NodeId,
    master_slot: usize,
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
            effects: Vec::new(),
            preset,
            volume: 0.8,
            pan: 0.0,
            muted: false,
            voice_cursor: 0,
            master_mixer_id,
            master_slot,
        })
    }

    /// Add an effect to the track's effects chain
    pub fn add_effect(&mut self, sched: &mut Scheduler, effect_type: EffectType, effect_id: u64) -> Option<()> {
        // Create the effect node
        let sample_rate = 48000.0; // TODO: Get from scheduler
        let node: Box<dyn DspNode> = match effect_type {
            EffectType::Compressor => Box::new(Compressor::new()),
            EffectType::Reverb => Box::new(Reverb::new(sample_rate)),
            EffectType::Delay => Box::new(DelayLine::new()),
            EffectType::Filter => Box::new(StateVariableFilter::new()),
            EffectType::Eq => Box::new(StateVariableFilter::new()), // Placeholder - use filter for now
        };
        
        let node_id = sched.graph.add_node(node)?;
        
        // Rewire the audio chain
        if self.effects.is_empty() {
            // First effect: disconnect mixer → master, insert effect
            sched.graph.disconnect(self.master_mixer_id, self.master_slot);
            sched.graph.connect(self.mixer_id, node_id, 0);
            sched.graph.connect(node_id, self.master_mixer_id, self.master_slot);
        } else {
            // Insert at end of chain
            let last_effect = self.effects.last().unwrap();
            sched.graph.disconnect(self.master_mixer_id, self.master_slot);
            sched.graph.connect(last_effect.node_id, node_id, 0);
            sched.graph.connect(node_id, self.master_mixer_id, self.master_slot);
        }
        
        // Set default parameters
        match effect_type {
            EffectType::Compressor => {
                set_param(sched, node_id, 0, -20.0); // threshold
                set_param(sched, node_id, 1, 4.0);   // ratio
                set_param(sched, node_id, 2, 0.01);  // attack
                set_param(sched, node_id, 3, 0.1);   // release
            }
            EffectType::Reverb => {
                set_param(sched, node_id, 0, 0.5);   // room size
                set_param(sched, node_id, 1, 0.5);   // damping
                set_param(sched, node_id, 2, 0.3);   // wet
            }
            EffectType::Delay => {
                set_param(sched, node_id, 0, 0.5);   // time
                set_param(sched, node_id, 1, 0.4);   // feedback
                set_param(sched, node_id, 2, 0.3);   // wet
            }
            EffectType::Filter => {
                set_param(sched, node_id, 0, 2000.0); // cutoff
                set_param(sched, node_id, 1, 1.0);    // resonance
                set_param(sched, node_id, 2, 0.0);    // mode (LP)
            }
            EffectType::Eq => {
                set_param(sched, node_id, 0, 1000.0); // cutoff
                set_param(sched, node_id, 1, 0.7);    // resonance
                set_param(sched, node_id, 2, 2.0);    // mode (BP)
            }
        }
        
        self.effects.push(EffectNode {
            id: effect_id,
            node_id,
            effect_type,
            enabled: true,
        });
        
        Some(())
    }
    
    /// Remove an effect from the chain
    pub fn remove_effect(&mut self, sched: &mut Scheduler, effect_id: u64) -> Option<()> {
        let idx = self.effects.iter().position(|e| e.id == effect_id)?;
        let effect = self.effects.remove(idx);
        
        // Rewire around the removed effect
        if self.effects.is_empty() {
            // Last effect removed: reconnect mixer directly to master
            sched.graph.disconnect(self.master_mixer_id, self.master_slot);
            sched.graph.connect(self.mixer_id, self.master_mixer_id, self.master_slot);
        } else if idx == 0 {
            // First effect removed
            sched.graph.disconnect(effect.node_id, 0);
            sched.graph.disconnect(self.effects[0].node_id, 0);
            sched.graph.connect(self.mixer_id, self.effects[0].node_id, 0);
        } else if idx == self.effects.len() {
            // Last effect removed (but not the only one)
            let prev = &self.effects[idx - 1];
            sched.graph.disconnect(effect.node_id, 0);
            sched.graph.disconnect(self.master_mixer_id, self.master_slot);
            sched.graph.connect(prev.node_id, self.master_mixer_id, self.master_slot);
        } else {
            // Middle effect removed
            let prev = &self.effects[idx - 1];
            let next = &self.effects[idx];
            sched.graph.disconnect(effect.node_id, 0);
            sched.graph.disconnect(next.node_id, 0);
            sched.graph.connect(prev.node_id, next.node_id, 0);
        }
        
        // Remove the node from the graph
        sched.graph.remove_node(effect.node_id);
        
        Some(())
    }
    
    /// Toggle effect bypass
    pub fn toggle_effect(&mut self, sched: &mut Scheduler, effect_id: u64) -> Option<()> {
        let effect = self.effects.iter_mut().find(|e| e.id == effect_id)?;
        effect.enabled = !effect.enabled;
        
        // Set bypass parameter (most effects use param index 10 for bypass)
        set_param(sched, effect.node_id, 10, if effect.enabled { 0.0 } else { 1.0 });
        
        Some(())
    }
    
    /// Set effect parameter
    pub fn set_effect_param(&mut self, sched: &mut Scheduler, effect_id: u64, param_idx: usize, value: f32) -> Option<()> {
        let effect = self.effects.iter().find(|e| e.id == effect_id)?;
        set_param(sched, effect.node_id, param_idx, value);
        Some(())
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
    pub metronome: Option<Metronome>,
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

        // Build metronome
        let metronome = Metronome::build(sched, master_id);

        Some(Self { tracks, master_id, metronome })
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

// ── Metronome ─────────────────────────────────────────────────────────────────

/// Metronome click generator
pub struct Metronome {
    pub click_osc_id: NodeId,
    pub click_env_id: NodeId,
    pub last_beat: f64,
}

impl Metronome {
    /// Build a metronome click generator
    pub fn build(sched: &mut Scheduler, master_id: NodeId) -> Option<Self> {
        // Create oscillator and envelope nodes
        let osc_id = sched.graph.add_node(Box::new(Oscillator::new()))?;
        let env_id = sched.graph.add_node(Box::new(AdsrEnvelope::new()))?;

        // Connect: osc → env → master (slot 15 - dedicated metronome slot)
        sched.graph.connect(osc_id, env_id, 0);
        sched.graph.connect(env_id, master_id, 15);

        // Configure click sound
        set_param(sched, osc_id, 0, 1000.0);  // freq (will be changed per click)
        set_param(sched, osc_id, 1, 0.3);     // amp
        set_param(sched, osc_id, 2, 0.0);     // waveform (sine)
        set_param(sched, osc_id, 3, -1.0);    // midi_note (-1 = use freq)

        // Configure envelope (short click)
        set_param(sched, env_id, 0, 0.001);   // attack (1ms)
        set_param(sched, env_id, 1, 0.030);   // decay (30ms)
        set_param(sched, env_id, 2, 0.0);     // sustain (0%)
        set_param(sched, env_id, 3, 0.010);   // release (10ms)
        set_param(sched, env_id, 4, 0.0);     // gate (off initially)

        Some(Self {
            click_osc_id: osc_id,
            click_env_id: env_id,
            last_beat: -1.0,
        })
    }

    /// Tick the metronome - call every frame when playing
    pub fn tick(&mut self, sched: &mut Scheduler, current_beat: f64, time_sig_num: u8) {
        let beat_floor = current_beat.floor();
        
        // Detect beat boundary crossing
        if beat_floor > self.last_beat.floor() {
            // Determine if this is a downbeat
            let beat_in_bar = (beat_floor as i32) % (time_sig_num as i32);
            let is_downbeat = beat_in_bar == 0;
            
            // Set frequency (1200Hz for downbeat, 1000Hz for regular beat)
            let freq = if is_downbeat { 1200.0 } else { 1000.0 };
            set_param(sched, self.click_osc_id, 0, freq);
            
            // Trigger gate
            set_param(sched, self.click_env_id, 4, 1.0);
            
            self.last_beat = beat_floor;
        }
    }
}
