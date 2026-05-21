//! MPE (MIDI Polyphonic Expression) support.
//!
//! MPE allows per-note control of pitch bend, pressure (aftertouch), and timbre (CC74).
//! Each note is assigned to a separate MIDI channel (typically channels 2-15),
//! with channel 1 reserved for global messages.
//!
//! # MPE Zones
//!
//! MPE defines two zones:
//! - **Lower Zone:** Channels 1-8 (channel 1 = master, 2-8 = member channels)
//! - **Upper Zone:** Channels 9-16 (channel 16 = master, 9-15 = member channels)
//!
//! # Per-Note Expression
//!
//! Each note can have independent:
//! - **Pitch Bend:** ±48 semitones (configurable)
//! - **Pressure:** Channel aftertouch (0-127)
//! - **Timbre:** CC74 (0-127)

use std::collections::HashMap;

/// MPE configuration.
#[derive(Debug, Clone, Copy)]
pub struct MpeConfig {
    /// Lower zone size (number of member channels, 0-15).
    /// 0 = disabled, 15 = channels 2-16 are member channels.
    pub lower_zone_size: u8,

    /// Upper zone size (number of member channels, 0-15).
    /// 0 = disabled, 15 = channels 1-15 are member channels.
    pub upper_zone_size: u8,

    /// Pitch bend range in semitones (typically 48).
    pub pitch_bend_range: f32,
}

impl Default for MpeConfig {
    fn default() -> Self {
        Self {
            lower_zone_size: 15, // Use all channels 2-16
            upper_zone_size: 0,  // Upper zone disabled
            pitch_bend_range: 48.0,
        }
    }
}

/// Per-note expression data.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoteExpression {
    /// MIDI note number (0-127).
    pub note: u8,

    /// MIDI channel (1-16).
    pub channel: u8,

    /// Velocity (0-127).
    pub velocity: u8,

    /// Pitch bend in semitones (±pitch_bend_range).
    pub pitch_bend: f32,

    /// Channel pressure/aftertouch (0.0-1.0).
    pub pressure: f32,

    /// Timbre/brightness CC74 (0.0-1.0).
    pub timbre: f32,
}

/// MPE engine for managing per-note expression.
pub struct MpeEngine {
    config: MpeConfig,

    /// Active notes: (channel, note) -> NoteExpression
    active_notes: HashMap<(u8, u8), NoteExpression>,

    /// Channel allocation: note -> channel
    /// Used to assign notes to available channels in round-robin fashion.
    note_to_channel: HashMap<u8, u8>,

    /// Next channel to allocate (round-robin).
    next_channel: u8,
}

impl MpeEngine {
    /// Creates a new MPE engine with default configuration.
    pub fn new() -> Self {
        Self::with_config(MpeConfig::default())
    }

    /// Creates a new MPE engine with custom configuration.
    pub fn with_config(config: MpeConfig) -> Self {
        Self {
            config,
            active_notes: HashMap::new(),
            note_to_channel: HashMap::new(),
            next_channel: 2, // Start at channel 2 (first member channel)
        }
    }

    /// Gets the current MPE configuration.
    pub fn config(&self) -> &MpeConfig {
        &self.config
    }

    /// Sets the MPE configuration.
    pub fn set_config(&mut self, config: MpeConfig) {
        self.config = config;
    }

    /// Handles a note-on event.
    ///
    /// Assigns the note to an available channel and creates a NoteExpression.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0-127)
    ///
    /// # Returns
    ///
    /// The NoteExpression for this note.
    pub fn note_on(&mut self, note: u8, velocity: u8) -> NoteExpression {
        // Allocate channel for this note
        let channel = self.allocate_channel();
        self.note_to_channel.insert(note, channel);

        let expr = NoteExpression {
            note,
            channel,
            velocity,
            pitch_bend: 0.0,
            pressure: 0.0,
            timbre: 0.0,
        };

        self.active_notes.insert((channel, note), expr);
        expr
    }

    /// Handles a note-off event.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    ///
    /// # Returns
    ///
    /// The NoteExpression that was removed, or None if note wasn't active.
    pub fn note_off(&mut self, note: u8) -> Option<NoteExpression> {
        if let Some(&channel) = self.note_to_channel.get(&note) {
            self.note_to_channel.remove(&note);
            self.active_notes.remove(&(channel, note))
        } else {
            None
        }
    }

    /// Handles a pitch bend event for a specific channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `value` - Pitch bend value (0-16383, 8192 = center)
    pub fn pitch_bend(&mut self, channel: u8, value: u16) {
        // Convert 14-bit pitch bend to semitones
        let normalized = (value as f32 - 8192.0) / 8192.0; // -1.0 to +1.0
        let semitones = normalized * self.config.pitch_bend_range;

        // Update all notes on this channel
        for ((ch, _note), expr) in self.active_notes.iter_mut() {
            if *ch == channel {
                expr.pitch_bend = semitones;
            }
        }
    }

    /// Handles a channel pressure (aftertouch) event.
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `value` - Pressure value (0-127)
    pub fn channel_pressure(&mut self, channel: u8, value: u8) {
        let normalized = value as f32 / 127.0;

        // Update all notes on this channel
        for ((ch, _note), expr) in self.active_notes.iter_mut() {
            if *ch == channel {
                expr.pressure = normalized;
            }
        }
    }

    /// Handles a CC74 (timbre/brightness) event.
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `value` - CC74 value (0-127)
    pub fn timbre(&mut self, channel: u8, value: u8) {
        let normalized = value as f32 / 127.0;

        // Update all notes on this channel
        for ((ch, _note), expr) in self.active_notes.iter_mut() {
            if *ch == channel {
                expr.timbre = normalized;
            }
        }
    }

    /// Gets the expression data for a specific note.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    ///
    /// # Returns
    ///
    /// The NoteExpression for this note, or None if not active.
    pub fn get_note_expression(&self, note: u8) -> Option<&NoteExpression> {
        if let Some(&channel) = self.note_to_channel.get(&note) {
            self.active_notes.get(&(channel, note))
        } else {
            None
        }
    }

    /// Gets all active note expressions.
    pub fn active_notes(&self) -> impl Iterator<Item = &NoteExpression> {
        self.active_notes.values()
    }

    /// Gets the number of active notes.
    pub fn active_note_count(&self) -> usize {
        self.active_notes.len()
    }

    /// Clears all active notes (panic button / all notes off).
    pub fn clear_all_notes(&mut self) {
        self.active_notes.clear();
        self.note_to_channel.clear();
        self.next_channel = 2;
    }

    /// Allocates a channel for a new note (round-robin).
    fn allocate_channel(&mut self) -> u8 {
        let max_channel = if self.config.lower_zone_size > 0 {
            1 + self.config.lower_zone_size
        } else {
            16
        };

        let channel = self.next_channel;
        self.next_channel += 1;
        if self.next_channel > max_channel {
            self.next_channel = 2; // Wrap around
        }

        channel
    }
}

impl Default for MpeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpe_note_on_off() {
        let mut mpe = MpeEngine::new();

        // Note on
        let expr = mpe.note_on(60, 100);
        assert_eq!(expr.note, 60);
        assert_eq!(expr.velocity, 100);
        assert_eq!(expr.channel, 2); // First member channel
        assert_eq!(mpe.active_note_count(), 1);

        // Note off
        let removed = mpe.note_off(60);
        assert!(removed.is_some());
        assert_eq!(mpe.active_note_count(), 0);
    }

    #[test]
    fn test_mpe_pitch_bend() {
        let mut mpe = MpeEngine::new();

        let expr = mpe.note_on(60, 100);
        let channel = expr.channel;

        // Apply pitch bend (+1 semitone)
        mpe.pitch_bend(channel, 8192 + 170); // ~+1 semitone

        let updated = mpe.get_note_expression(60).unwrap();
        assert!(updated.pitch_bend > 0.9 && updated.pitch_bend < 1.1);
    }

    #[test]
    fn test_mpe_pressure() {
        let mut mpe = MpeEngine::new();

        let expr = mpe.note_on(60, 100);
        let channel = expr.channel;

        // Apply pressure
        mpe.channel_pressure(channel, 64); // 50%

        let updated = mpe.get_note_expression(60).unwrap();
        assert!((updated.pressure - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_mpe_timbre() {
        let mut mpe = MpeEngine::new();

        let expr = mpe.note_on(60, 100);
        let channel = expr.channel;

        // Apply timbre
        mpe.timbre(channel, 127); // 100%

        let updated = mpe.get_note_expression(60).unwrap();
        assert_eq!(updated.timbre, 1.0);
    }

    #[test]
    fn test_mpe_channel_allocation() {
        let mut mpe = MpeEngine::new();

        // Allocate 3 notes
        let expr1 = mpe.note_on(60, 100);
        let expr2 = mpe.note_on(62, 100);
        let expr3 = mpe.note_on(64, 100);

        // Should get different channels
        assert_eq!(expr1.channel, 2);
        assert_eq!(expr2.channel, 3);
        assert_eq!(expr3.channel, 4);
    }

    #[test]
    fn test_mpe_clear_all() {
        let mut mpe = MpeEngine::new();

        mpe.note_on(60, 100);
        mpe.note_on(62, 100);
        mpe.note_on(64, 100);
        assert_eq!(mpe.active_note_count(), 3);

        mpe.clear_all_notes();
        assert_eq!(mpe.active_note_count(), 0);
    }
}
