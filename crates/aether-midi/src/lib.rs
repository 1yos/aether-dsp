//! Aether MIDI Engine
//!
//! Handles MIDI input from any connected keyboard or controller.
//! Converts raw MIDI bytes into typed MidiEvent messages.
//! Routes events to registered instruments via a lock-free channel.
//!
//! Supports:
//!   - Note On / Note Off with velocity
//!   - Pitch Bend (±2 semitones by default, configurable)
//!   - Aftertouch (channel and polyphonic)
//!   - Control Change (CC) — mod wheel, sustain pedal, expression, etc.
//!   - Program Change
//!   - Custom tuning tables (for non-Western instruments)

pub mod engine;
pub mod event;
pub mod mpe;
pub mod router;
pub mod smf;
pub mod tuning;

pub use engine::MidiEngine;
pub use event::{MidiEvent, MidiEventKind};
pub use mpe::{MpeConfig, MpeEngine, NoteExpression};
pub use router::MidiRouter;
pub use smf::{Division, MidiFile, MidiFormat, MidiTrack, TimedMidiEvent};
pub use tuning::TuningTable;
