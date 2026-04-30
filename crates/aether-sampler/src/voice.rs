//! Sampler voice — one playing note.

use crate::instrument::{ArticulationType, SampleZone};

/// The phase of a voice's lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum VoicePhase {
    /// Key is held — playing forward.
    Attack,
    /// Sustain loop active.
    Sustain,
    /// Key released — playing release tail.
    Release,
    /// Voice is done — can be recycled.
    Done,
}

/// ADSR envelope state.
#[derive(Debug, Clone)]
pub struct EnvelopeState {
    pub level: f32,
    pub phase: EnvPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvPhase { Attack, Decay, Sustain, Release, Done }

impl EnvelopeState {
    pub fn new() -> Self {
        Self { level: 0.0, phase: EnvPhase::Attack }
    }
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeState {
    pub fn tick(&mut self, attack_rate: f32, decay_rate: f32, sustain: f32, release_rate: f32) {
        match self.phase {
            EnvPhase::Attack => {
                self.level += attack_rate;
                if self.level >= 1.0 { self.level = 1.0; self.phase = EnvPhase::Decay; }
            }
            EnvPhase::Decay => {
                self.level -= decay_rate;
                if self.level <= sustain { self.level = sustain; self.phase = EnvPhase::Sustain; }
            }
            EnvPhase::Sustain => {}
            EnvPhase::Release => {
                self.level -= release_rate;
                if self.level <= 0.0 { self.level = 0.0; self.phase = EnvPhase::Done; }
            }
            EnvPhase::Done => {}
        }
    }

    pub fn release(&mut self) {
        if self.phase != EnvPhase::Done {
            self.phase = EnvPhase::Release;
        }
    }

    pub fn is_done(&self) -> bool { self.phase == EnvPhase::Done }
}

/// One active voice in the sampler.
pub struct SamplerVoice {
    /// MIDI note number.
    pub note: u8,
    /// MIDI channel.
    pub channel: u8,
    /// Velocity (0.0–1.0).
    pub velocity: f32,
    /// Current playback position in frames (sub-sample precision).
    pub position: f64,
    /// Playback speed ratio (accounts for pitch shifting).
    pub pitch_ratio: f64,
    /// Volume multiplier from zone + velocity.
    pub volume: f32,
    /// Current lifecycle phase.
    pub phase: VoicePhase,
    /// ADSR envelope.
    pub envelope: EnvelopeState,
    /// Zone id this voice is playing.
    pub zone_id: String,
    /// Articulation type (cached from zone).
    pub articulation: ArticulationType,
    /// Loop start frame (if sustain loop).
    pub loop_start: usize,
    /// Loop end frame (if sustain loop).
    pub loop_end: usize,
    /// Whether the key is still held.
    pub key_held: bool,
}

impl SamplerVoice {
    pub fn new(
        note: u8,
        channel: u8,
        velocity: f32,
        pitch_ratio: f64,
        volume: f32,
        zone: &SampleZone,
    ) -> Self {
        let (loop_start, loop_end) = match &zone.articulation {
            ArticulationType::SustainLoop { loop_start, loop_end } => (*loop_start, *loop_end),
            _ => (0, 0),
        };
        Self {
            note,
            channel,
            velocity,
            position: 0.0,
            pitch_ratio,
            volume,
            phase: VoicePhase::Attack,
            envelope: EnvelopeState::new(),
            zone_id: zone.id.clone(),
            articulation: zone.articulation.clone(),
            loop_start,
            loop_end,
            key_held: true,
        }
    }

    /// Signal key release.
    pub fn release(&mut self) {
        self.key_held = false;
        self.envelope.release();
        if self.phase == VoicePhase::Sustain || self.phase == VoicePhase::Attack {
            self.phase = VoicePhase::Release;
        }
    }

    /// Is this voice finished?
    pub fn is_done(&self) -> bool {
        self.phase == VoicePhase::Done || self.envelope.is_done()
    }

    /// Advance position by one sample, handling loop points.
    /// Returns the current frame position for sample lookup.
    pub fn advance(&mut self, buffer_frames: usize) -> f64 {
        let pos = self.position;
        self.position += self.pitch_ratio;

        match &self.articulation {
            ArticulationType::SustainLoop { loop_start, loop_end } => {
                if self.key_held && self.position >= *loop_end as f64 {
                    self.position = *loop_start as f64 + (self.position - *loop_end as f64);
                    self.phase = VoicePhase::Sustain;
                } else if !self.key_held && self.position >= buffer_frames as f64 {
                    self.phase = VoicePhase::Done;
                }
            }
            ArticulationType::OneShot => {
                if self.position >= buffer_frames as f64 {
                    self.phase = VoicePhase::Done;
                }
            }
            ArticulationType::SustainRelease => {
                if self.position >= buffer_frames as f64 {
                    self.phase = VoicePhase::Done;
                }
            }
        }

        pos
    }
}
