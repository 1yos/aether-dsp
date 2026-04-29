//! MIDI event types.

use serde::{Deserialize, Serialize};

/// A single MIDI event with timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiEvent {
    /// Sample-accurate timestamp (samples since engine start).
    pub timestamp: u64,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// The event kind.
    pub kind: MidiEventKind,
}

/// All MIDI event types Aether handles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiEventKind {
    /// Key pressed. note=0–127, velocity=0–127.
    NoteOn { note: u8, velocity: u8 },
    /// Key released. note=0–127, velocity=0–127 (release velocity).
    NoteOff { note: u8, velocity: u8 },
    /// Pitch wheel. value=-8192 to +8191 (0 = center).
    PitchBend { value: i16 },
    /// Channel aftertouch (pressure on any held key).
    ChannelPressure { pressure: u8 },
    /// Polyphonic aftertouch (pressure on a specific key).
    PolyPressure { note: u8, pressure: u8 },
    /// Control Change. cc=controller number, value=0–127.
    ControlChange { cc: u8, value: u8 },
    /// Program Change. program=0–127.
    ProgramChange { program: u8 },
    /// All notes off (panic).
    AllNotesOff,
    /// All sound off (immediate silence).
    AllSoundOff,
}

impl MidiEvent {
    /// Parse raw MIDI bytes into a MidiEvent.
    /// Returns None for unsupported or malformed messages.
    pub fn from_bytes(bytes: &[u8], timestamp: u64) -> Option<Self> {
        if bytes.is_empty() { return None; }
        let status = bytes[0];
        let channel = status & 0x0F;
        let kind_byte = status >> 4;

        let kind = match kind_byte {
            0x8 => {
                // Note Off
                if bytes.len() < 3 { return None; }
                MidiEventKind::NoteOff { note: bytes[1] & 0x7F, velocity: bytes[2] & 0x7F }
            }
            0x9 => {
                if bytes.len() < 3 { return None; }
                let vel = bytes[2] & 0x7F;
                if vel == 0 {
                    // Note On with velocity 0 = Note Off
                    MidiEventKind::NoteOff { note: bytes[1] & 0x7F, velocity: 0 }
                } else {
                    MidiEventKind::NoteOn { note: bytes[1] & 0x7F, velocity: vel }
                }
            }
            0xA => {
                if bytes.len() < 3 { return None; }
                MidiEventKind::PolyPressure { note: bytes[1] & 0x7F, pressure: bytes[2] & 0x7F }
            }
            0xB => {
                if bytes.len() < 3 { return None; }
                let cc = bytes[1] & 0x7F;
                let val = bytes[2] & 0x7F;
                match cc {
                    120 => MidiEventKind::AllSoundOff,
                    123 => MidiEventKind::AllNotesOff,
                    _ => MidiEventKind::ControlChange { cc, value: val },
                }
            }
            0xC => {
                if bytes.len() < 2 { return None; }
                MidiEventKind::ProgramChange { program: bytes[1] & 0x7F }
            }
            0xD => {
                if bytes.len() < 2 { return None; }
                MidiEventKind::ChannelPressure { pressure: bytes[1] & 0x7F }
            }
            0xE => {
                if bytes.len() < 3 { return None; }
                let lsb = bytes[1] as i16;
                let msb = bytes[2] as i16;
                let value = ((msb << 7) | lsb) - 8192;
                MidiEventKind::PitchBend { value }
            }
            _ => return None,
        };

        Some(MidiEvent { timestamp, channel, kind })
    }

    /// Is this a note-on event?
    pub fn is_note_on(&self) -> bool {
        matches!(self.kind, MidiEventKind::NoteOn { .. })
    }

    /// Is this a note-off event?
    pub fn is_note_off(&self) -> bool {
        matches!(self.kind, MidiEventKind::NoteOff { .. })
    }

    /// Get the MIDI note number if this is a note event.
    pub fn note(&self) -> Option<u8> {
        match self.kind {
            MidiEventKind::NoteOn { note, .. } | MidiEventKind::NoteOff { note, .. } => Some(note),
            _ => None,
        }
    }

    /// Get velocity (0–127) if this is a note event.
    pub fn velocity(&self) -> Option<u8> {
        match self.kind {
            MidiEventKind::NoteOn { velocity, .. } | MidiEventKind::NoteOff { velocity, .. } => Some(velocity),
            _ => None,
        }
    }

    /// Convert velocity to linear amplitude (0.0–1.0).
    pub fn velocity_linear(&self) -> f32 {
        self.velocity().map(|v| v as f32 / 127.0).unwrap_or(0.0)
    }
}

/// Standard MIDI CC numbers.
pub mod cc {
    pub const MOD_WHEEL: u8 = 1;
    pub const BREATH: u8 = 2;
    pub const FOOT_PEDAL: u8 = 4;
    pub const PORTAMENTO_TIME: u8 = 5;
    pub const VOLUME: u8 = 7;
    pub const BALANCE: u8 = 8;
    pub const PAN: u8 = 10;
    pub const EXPRESSION: u8 = 11;
    pub const SUSTAIN_PEDAL: u8 = 64;
    pub const PORTAMENTO: u8 = 65;
    pub const SOSTENUTO: u8 = 66;
    pub const SOFT_PEDAL: u8 = 67;
    pub const LEGATO: u8 = 68;
    pub const REVERB_SEND: u8 = 91;
    pub const CHORUS_SEND: u8 = 93;
}
