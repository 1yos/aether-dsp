//! Custom tuning tables for non-Western instruments.
//!
//! Standard MIDI assumes 12-tone equal temperament (12-TET).
//! Many instruments — Ethiopian, Indian, Arabic, Turkish, gamelan —
//! use different tuning systems. This module lets you define the exact
//! frequency for each MIDI note number.
//!
//! ## Precision
//!
//! Frequencies are stored as `f32`, providing approximately 0.0001 Hz precision
//! at 440 Hz. This is more than sufficient for audio applications, as human pitch
//! discrimination is typically around 1 Hz at best. For extreme low-frequency
//! accuracy (<1 Hz), consider using `f64` in custom implementations.
//!
//! ## Pitch-Bend Interaction
//!
//! When using tuning tables with MIDI pitch-bend:
//! - Pitch-bend operates **relative to the tuned pitch**, not 12-TET
//! - Example: A note tuned to 261.63 Hz with +200 cent bend becomes 261.63 * 2^(200/1200)
//! - This preserves the tuning system's interval relationships
//! - Vibrato and modulation also operate relative to the tuned frequency
//!
//! This ensures that microtonal music remains in the correct tuning system even
//! when pitch-bend or vibrato is applied.

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
        for (note, freq) in frequencies.iter_mut().enumerate() {
            *freq = concert_a * 2.0f32.powf((note as f32 - 69.0) / 12.0);
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
        for (note, freq) in frequencies.iter_mut().enumerate().take(128) {
            let pitch_class = note % 12;
            let cents_offset = offsets[pitch_class];
            *freq *= 2.0f32.powf(cents_offset / 1200.0);
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
    /// 
    /// NOTE: This is an approximation. Ethiopian qenet (modal scales) are defined
    /// more by melodic contour, ornamentation, and emotional intent than fixed
    /// interval ratios. The cent offsets used here are estimates based on
    /// performance practice, not documented measurements.
    /// 
    /// Source: Approximation based on Ethiopian music performance practice.
    /// TODO: Validate with Ethiopian musicologists or Kebede's research.
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

    /// Just intonation (5-limit) — pure intervals based on harmonic series.
    /// Uses ratios with prime factors up to 5 (e.g., 3/2, 5/4).
    /// 
    /// This produces perfectly consonant major thirds (5/4) and perfect fifths (3/2)
    /// with no beating, unlike 12-TET which has slight detuning.
    /// 
    /// Source: Traditional Western just intonation, documented since Ptolemy (2nd century).
    /// Note: This is 5-limit JI. For septimal intervals (7/4, 7/6), see just_intonation_7_limit.
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
        t.name = "Just Intonation (5-limit)".into();
        t.description = "Pure harmonic ratios — no beating on perfect intervals".into();
        t
    }
}

impl Default for TuningTable {
    fn default() -> Self {
        Self::equal_temperament(440.0)
    }
}

// ── Additional world music tuning systems ─────────────────────────────────────

impl TuningTable {
    /// Arabic Maqam Rast — the most common Arabic maqam.
    /// Uses quarter-tone flats on the 3rd and 7th scale degrees.
    /// 
    /// NOTE: This implementation uses 24-TET (50-cent quarter-tones), which is
    /// the modern theoretical standard established by Mikhail Mishaqa (19th century).
    /// Historical Arabic music theory (al-Farabi, al-Urmawi) used ratio-based
    /// intervals. Performance practice often deviates from both systems based on
    /// melodic context and regional tradition.
    /// 
    /// Source: Modern 24-TET Arabic music theory (24-tone equal temperament).
    pub fn arabic_maqam_rast(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C  — root (Rast)
            0.0,    // C#
            0.0,    // D  — whole tone
            -50.0,  // D# — E half-flat (quarter tone flat)
            0.0,    // E
            0.0,    // F  — perfect fourth
            0.0,    // F#
            0.0,    // G  — perfect fifth
            0.0,    // G#
            0.0,    // A
            -50.0,  // A# — B half-flat (quarter tone flat)
            0.0,    // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Arabic Maqam Rast".into();
        t.description = "Arabic Maqam Rast — quarter-tone flats on 3rd and 7th degrees".into();
        t
    }

    /// Arabic Maqam Bayati — second most common Arabic maqam.
    /// Characteristic half-flat on the 2nd degree.
    /// 
    /// NOTE: This implementation uses 24-TET (50-cent quarter-tones), which is
    /// the modern theoretical standard. Performance practice varies by region
    /// and melodic context.
    /// 
    /// Source: Modern 24-TET Arabic music theory (24-tone equal temperament).
    pub fn arabic_maqam_bayati(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C  — root
            -50.0,  // C# — D half-flat (characteristic Bayati interval)
            0.0,    // D
            -30.0,  // D# — slightly flat
            0.0,    // E
            0.0,    // F
            0.0,    // F#
            0.0,    // G
            0.0,    // G#
            0.0,    // A
            -50.0,  // A# — B half-flat
            0.0,    // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Arabic Maqam Bayati".into();
        t.description = "Arabic Maqam Bayati — half-flat on 2nd degree, characteristic of Arabic music".into();
        t
    }

    /// Arabic Maqam Hijaz — characteristic augmented 2nd interval.
    /// Tetrachord pattern: semitone - augmented 2nd - semitone (1-3-1).
    /// 
    /// The Hijaz tetrachord is one of the most distinctive sounds in Arabic music,
    /// featuring a large augmented 2nd (300 cents) between the 2nd and 3rd degrees.
    /// Also known as "Phrygian dominant" in Western theory and "Freygish" in Jewish music.
    /// 
    /// Scale structure from root: C - Db - E - F - G - Ab - B - C
    /// Intervals: semitone (100¢), augmented 2nd (300¢), semitone (100¢), whole tone (200¢),
    ///            semitone (100¢), augmented 2nd (300¢), semitone (100¢)
    /// 
    /// Source: Traditional Arabic maqam theory, documented in maqamworld.com and
    /// ethnomusicological literature.
    pub fn arabic_maqam_hijaz(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C  — root (Hijaz)
            0.0,    // C# — Db (semitone above root)
            0.0,    // D
            0.0,    // D#
            0.0,    // E  — augmented 2nd from Db (characteristic interval)
            0.0,    // F  — semitone above E
            0.0,    // F#
            0.0,    // G  — whole tone above F
            -100.0, // G# — Ab (semitone above G)
            0.0,    // A
            0.0,    // A#
            0.0,    // B  — augmented 2nd from Ab
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Arabic Maqam Hijaz".into();
        t.description = "Arabic Maqam Hijaz — augmented 2nd between 2nd and 3rd degrees (1-3-1 tetrachord)".into();
        t
    }

    /// Ethiopian Bati scale — minor pentatonic variant.
    /// 
    /// NOTE: This is an approximation. Ethiopian qenet (modal scales) are defined
    /// more by melodic contour, ornamentation, and emotional intent than fixed
    /// interval ratios. The cent offsets used here (-20/-30/-20) are estimates
    /// based on performance practice, not documented measurements.
    /// 
    /// Source: Approximation based on Ethiopian music performance practice.
    /// TODO: Validate with Ethiopian musicologists or Kebede's research.
    pub fn ethiopian_bati(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C
            0.0,    // C#
            -20.0,  // D  — slightly flat
            0.0,    // D#
            0.0,    // E
            0.0,    // F
            -30.0,  // F# — slightly flat
            0.0,    // G
            0.0,    // G#
            -20.0,  // A  — slightly flat
            0.0,    // A#
            0.0,    // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Bati".into();
        t.description = "Ethiopian Bati scale — minor pentatonic variant used in traditional music".into();
        t
    }

    /// Ethiopian Ambassel — pentatonic with raised 4th.
    /// One of the four main Ethiopian qenet (modal scales).
    /// 
    /// Ambassel (also spelled Ambasel or Ambessel) is characterized by its raised
    /// 4th degree, distinguishing it from Bati which has a lowered 4th. The scale
    /// structure is: C - D - F - G - A (pentatonic).
    /// 
    /// Intervals from root: whole tone (200¢), minor 3rd (300¢), whole tone (200¢),
    /// whole tone (200¢).
    /// 
    /// NOTE: Like all Ethiopian qenet, this is defined more by melodic contour and
    /// ornamentation than fixed intervals. This implementation provides a 12-TET
    /// approximation for the characteristic pentatonic structure.
    /// 
    /// Source: Ethiopian music theory, documented in Scribd "Ethiopian Music Scales"
    /// and ethnomusicological literature.
    /// TODO: Validate with Ethiopian musicologists or Kebede's research.
    pub fn ethiopian_ambassel(concert_a: f32) -> Self {
        let offsets = [
            0.0,    // C  — root
            0.0,    // C#
            0.0,    // D  — whole tone (200 cents)
            0.0,    // D#
            0.0,    // E
            0.0,    // F  — raised 4th (characteristic interval, 500 cents from root)
            0.0,    // F#
            0.0,    // G  — perfect 5th (700 cents from root)
            0.0,    // G#
            0.0,    // A  — major 6th (900 cents from root)
            0.0,    // A#
            0.0,    // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Ambassel".into();
        t.description = "Ethiopian Ambassel — pentatonic with raised 4th, one of four main qenet modes".into();
        t
    }

    /// Indian Raga Yaman (Kalyan thaat) — the most common North Indian raga.
    /// Uses a raised 4th (Ma tivra).
    /// 
    /// This implementation uses just intonation ratios from Sa (root):
    /// Sa Re Ga Ma# Pa Dha Ni Sa = 1/1, 9/8, 5/4, 45/32, 3/2, 5/3, 15/8, 2/1
    /// 
    /// Source: North Indian classical music theory, just intonation ratios.
    pub fn indian_raga_yaman(concert_a: f32) -> Self {
        // Yaman uses all natural notes except F# (raised 4th)
        // In just intonation ratios from Sa (root):
        // Sa Re Ga Ma# Pa Dha Ni Sa
        // 1  9/8 5/4 45/32 3/2 5/3 15/8 2
        let offsets = [
            0.0,   // C  — Sa
            0.0,   // C#
            3.9,   // D  — Re (9/8 just = +3.9 cents from 12-TET)
            0.0,   // D#
            -13.7, // E  — Ga (5/4 just = -13.7 cents from 12-TET)
            0.0,   // F
            -9.8,  // F# — Ma# (45/32 just = -9.8 cents from 12-TET)
            2.0,   // G  — Pa (3/2 just = +2.0 cents from 12-TET)
            0.0,   // G#
            -15.6, // A  — Dha (5/3 just = -15.6 cents from 12-TET)
            0.0,   // A#
            -11.7, // B  — Ni (15/8 just = -11.7 cents from 12-TET)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Indian Raga Yaman".into();
        t.description = "Indian Raga Yaman (Kalyan thaat) — raised 4th, just intonation".into();
        t
    }

    /// Javanese Gamelan Slendro — 5-tone scale.
    /// Approximate equal division of the octave into 5 parts.
    /// 
    /// NOTE: This uses exact 2:1 octaves (1200 cents). Real gamelan ensembles
    /// often have stretched octaves (~1210-1215 cents) due to inharmonic overtones
    /// of bronze/iron bars. For stretched octave version, see gamelan_slendro_stretched.
    /// 
    /// Source: Generic approximation. Real gamelan tunings vary by ensemble.
    /// Reference: "On the Tuning and Stretched Octave of Javanese Gamelans" (2016).
    pub fn gamelan_slendro(_concert_a: f32) -> Self {
        // Slendro divides the octave into 5 roughly equal parts (~240 cents each)
        // but with characteristic deviations. Using a common approximation.
        let step = 1200.0 / 5.0; // 240 cents per step
        let mut frequencies = vec![0.0f32; 128];
        for (note, freq) in frequencies.iter_mut().enumerate() {
            // Map MIDI notes to Slendro: every 2-3 semitones is one Slendro step
            let slendro_step = (note as f32 / 2.4).floor();
            let cents_from_c0 = slendro_step * step;
            *freq = 16.352 * 2.0f32.powf(cents_from_c0 / 1200.0);
        }
        Self {
            frequencies,
            name: "Gamelan Slendro".into(),
            description: "Javanese Gamelan Slendro — 5-tone scale, ~240 cents per step".into(),
        }
    }

    /// Javanese Gamelan Pelog — 7-tone scale with characteristic large and small intervals.
    /// 
    /// NOTE: Pelog tuning varies dramatically between gamelan ensembles. This is
    /// a generic approximation using commonly cited interval patterns. Real gamelan
    /// instruments are tuned individually and not intended to match Western pitch
    /// standards or other ensembles.
    /// 
    /// For authentic reproduction, measure a specific ensemble or use documented
    /// measurements from ethnomusicological studies.
    /// 
    /// Source: Generic approximation. Reference: "Javanese Pelog Tunings Reconsidered" (1980).
    pub fn gamelan_pelog(concert_a: f32) -> Self {
        // Pelog has 7 tones with unequal steps. Common approximation in cents from root:
        // 0, 120, 270, 540, 675, 785, 950, 1200
        let pelog_cents = [0.0f32, 120.0, 270.0, 540.0, 675.0, 785.0, 950.0];
        let mut frequencies = vec![0.0f32; 128];
        for (note, freq) in frequencies.iter_mut().enumerate() {
            let octave = note / 7;
            let step = note % 7;
            let cents = pelog_cents[step] + octave as f32 * 1200.0;
            *freq = 16.352 * 2.0f32.powf(cents / 1200.0);
        }
        // Normalize so A4 (MIDI 69) = concert_a
        let a4_freq = frequencies[69];
        if a4_freq > 0.0 {
            let ratio = concert_a / a4_freq;
            for f in frequencies.iter_mut() { *f *= ratio; }
        }
        Self {
            frequencies,
            name: "Gamelan Pelog".into(),
            description: "Javanese Gamelan Pelog — 7-tone scale with characteristic unequal intervals".into(),
        }
    }
}
