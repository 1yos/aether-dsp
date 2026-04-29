//! Custom tuning tables for non-Western instruments.
//!
//! Standard MIDI assumes 12-tone equal temperament (12-TET).
//! Many instruments — Ethiopian, Indian, Arabic, Turkish, gamelan —
//! use different tuning systems. This module lets you define the exact
//! frequency for each MIDI note number.

use serde::{Deserialize, Serialize};

/// Maps MIDI note numbers (0–127) to frequencies in Hz.
/// Stored as Vec<f32> for serde compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningTable {
    /// Frequency in Hz for each MIDI note 0–127.
    pub frequencies: Vec<f32>,
    /// Human-readable name.
    pub name: String,
    /// Description of the tuning system.
    pub description: String,
}

impl TuningTable {
    /// Standard 12-tone equal temperament.
    /// A4 (MIDI note 69) = concert_a Hz (typically 440.0).
    pub fn equal_temperament(concert_a: f32) -> Self {
        let mut frequencies = vec![0.0f32; 128];
        for note in 0..128u8 {
            frequencies[note as usize] = concert_a * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        }
        Self {
            frequencies,
            name: "12-TET".into(),
            description: "Standard 12-tone equal temperament, A4=440Hz".into(),
        }
    }

    /// Build a tuning table from cents offsets per semitone within an octave.
    /// `offsets` is a 12-element array of cent offsets from 12-TET for each
    /// pitch class (C, C#, D, D#, E, F, F#, G, G#, A, A#, B).
    pub fn from_cents_offsets(concert_a: f32, offsets: &[f32; 12]) -> Self {
        let base = Self::equal_temperament(concert_a);
        let mut frequencies = base.frequencies;
        for note in 0..128usize {
            let pitch_class = note % 12;
            let cents_offset = offsets[pitch_class];
            frequencies[note] *= 2.0f32.powf(cents_offset / 1200.0);
        }
        Self {
            frequencies,
            name: "Custom".into(),
            description: "Custom tuning with per-pitch-class cent offsets".into(),
        }
    }

    /// Build from explicit frequency list. Length must be 128.
    pub fn from_frequencies(freqs: Vec<f32>, name: &str, description: &str) -> Option<Self> {
        if freqs.len() != 128 {
            return None;
        }
        Some(Self {
            frequencies: freqs,
            name: name.into(),
            description: description.into(),
        })
    }

    /// Get frequency for a MIDI note number.
    #[inline]
    pub fn frequency(&self, note: u8) -> f32 {
        self.frequencies.get(note as usize).copied().unwrap_or(0.0)
    }

    /// Convert frequency to the nearest MIDI note + cents deviation.
    pub fn freq_to_note_cents(&self, freq: f32) -> (u8, f32) {
        let mut best_note = 0u8;
        let mut best_dist = f32::MAX;
        for (i, &f) in self.frequencies.iter().enumerate() {
            let dist = (freq - f).abs();
            if dist < best_dist {
                best_dist = dist;
                best_note = i as u8;
            }
        }
        let base_freq = self.frequencies[best_note as usize];
        let cents = if base_freq > 0.0 {
            1200.0 * (freq / base_freq).log2()
        } else {
            0.0
        };
        (best_note, cents)
    }

    /// Ethiopian Kiñit (pentatonic) approximation.
    /// Uses the Tizita major scale pattern.
    pub fn ethiopian_tizita(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C  — root
            -50.0,  // C# — slightly flat
            0.0,    // D
            -30.0,  // D# — slightly flat
            0.0,    // E
            0.0,    // F
            -20.0,  // F# — slightly flat
            0.0,    // G
            -40.0,  // G# — slightly flat
            0.0,    // A
            -30.0,  // A# — slightly flat
            0.0,    // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Tizita".into();
        t.description = "Approximation of Ethiopian Tizita major pentatonic scale".into();
        t
    }

    /// Just intonation (pure intervals based on harmonic series).
    pub fn just_intonation(concert_a: f32) -> Self {
        let ratios: [f32; 12] = [
            1.0, 16.0/15.0, 9.0/8.0, 6.0/5.0, 5.0/4.0, 4.0/3.0,
            45.0/32.0, 3.0/2.0, 8.0/5.0, 5.0/3.0, 9.0/5.0, 15.0/8.0,
        ];
        let tet_ratios: [f32; 12] = [
            1.0, 2.0f32.powf(1.0/12.0), 2.0f32.powf(2.0/12.0), 2.0f32.powf(3.0/12.0),
            2.0f32.powf(4.0/12.0), 2.0f32.powf(5.0/12.0), 2.0f32.powf(6.0/12.0),
            2.0f32.powf(7.0/12.0), 2.0f32.powf(8.0/12.0), 2.0f32.powf(9.0/12.0),
            2.0f32.powf(10.0/12.0), 2.0f32.powf(11.0/12.0),
        ];
        let offsets: [f32; 12] = std::array::from_fn(|i| {
            1200.0 * (ratios[i] / tet_ratios[i]).log2()
        });
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Just Intonation".into();
        t.description = "Pure harmonic ratios — no beating on perfect intervals".into();
        t
    }
}

impl Default for TuningTable {
    fn default() -> Self {
        Self::equal_temperament(440.0)
    }
}
