//! Instrument synthesizer — generate synthetic samples for the instrument maker
//! when you don't have the actual instrument to record.
//!
//! Uses timbre transfer to reshape an existing sound source into the target timbre.

use crate::analysis::TimbreProfile;
use crate::transfer::TimbreTransfer;

/// Generates synthetic instrument samples using timbre transfer.
pub struct InstrumentSynthesizer {
    pub profile: TimbreProfile,
    transfer: TimbreTransfer,
}

impl InstrumentSynthesizer {
    pub fn new(profile: TimbreProfile) -> Self {
        let fft_size = 2048;
        Self {
            transfer: TimbreTransfer::new(fft_size),
            profile,
        }
    }

    /// Generate a synthetic sample for a given MIDI note.
    ///
    /// `source_audio` — audio from any instrument (guitar, piano, voice, etc.)
    /// `note` — the MIDI note to generate
    /// `sample_rate` — sample rate in Hz (unused currently, reserved for future pitch shifting)
    ///
    /// Returns a SampleBuffer with the synthesized audio.
    pub fn synthesize_note(
        &mut self,
        source_audio: &[f32],
        note: u8,
        _sample_rate: f32,
    ) -> Vec<f32> {
        // Get the target envelope for this note
        if let Some(envelope) = self.profile.envelope_for_note(note) {
            self.transfer.set_target(envelope.clone());
            self.transfer.amount = 1.0;
        }

        // Apply timbre transfer
        self.transfer.process_block(source_audio)
    }

    /// Generate a complete set of samples for all notes in a range.
    /// Returns a map of note → synthesized audio.
    pub fn synthesize_range(
        &mut self,
        source_audio: &[f32],
        note_low: u8,
        note_high: u8,
        _sample_rate: f32,
    ) -> Vec<(u8, Vec<f32>)> {
        let mut results = Vec::new();
        for note in note_low..=note_high {
            let audio = self.synthesize_note(source_audio, note, _sample_rate);
            results.push((note, audio));
        }
        results
    }
}
