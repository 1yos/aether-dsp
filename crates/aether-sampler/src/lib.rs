//! Aether Sampler Engine
//!
//! A polyphonic sampler that maps audio recordings to MIDI notes.
//! Supports velocity layers, sustain loops, release samples,
//! and custom tuning tables for non-Western instruments.
//!
//! # Architecture
//!
//! ```
//! SamplerInstrument
//!   └── Vec<SampleZone>        — note range + velocity range → audio file
//!         └── SampleBuffer     — decoded PCM audio in memory
//!
//! SamplerVoice                 — one playing note (polyphony = many voices)
//!   ├── zone: &SampleZone      — which sample to play
//!   ├── position: f64          — current playback position (sub-sample)
//!   ├── phase: VoicePhase      — Attack / Sustain / Release / Done
//!   └── envelope: AdsrState   — amplitude envelope
//! ```

pub mod buffer;
pub mod instrument;
pub mod node;
pub mod voice;

pub use instrument::{
    ArticulationType, RoundRobinMode, RoundRobinState, SampleZone, SamplerInstrument, ZoneGroup,
};
pub use node::SamplerNode;
pub use voice::SamplerVoice;
