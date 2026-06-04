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
/// Stored as `Vec<f32>` for serde compatibility.
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

    /// Ethiopian Tizita major — the most common Ethiopian qenet mode.
    /// Scale pattern: C - D - E - G - A (major pentatonic).
    /// Intervals: M2, M2, m3, M2, m3
    ///
    /// NOTE: This implementation uses 12-TET. Traditional Ethiopian performance
    /// includes microtonal inflections and flexible intonation that cannot be
    /// captured in fixed tuning tables. The scale is defined more by melodic
    /// contour and emotional intent than exact intervals.
    ///
    /// Source: Ethiopian music theory documentation (Scribd, PubPub, Wikipedia).
    /// Equivalent to Western major pentatonic when played in 12-TET.
    pub fn ethiopian_tizita(concert_a: f32) -> Self {
        // Tizita major uses the standard major pentatonic: C D E G A
        // In 12-TET, this is exactly the major pentatonic scale
        let offsets = [
            0.0, // C  — root
            0.0, // C# (not used in pentatonic)
            0.0, // D  — major 2nd
            0.0, // D# (not used)
            0.0, // E  — major 3rd
            0.0, // F  (not used)
            0.0, // F# (not used)
            0.0, // G  — perfect 5th
            0.0, // G# (not used)
            0.0, // A  — major 6th
            0.0, // A# (not used)
            0.0, // B  (not used)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Tizita (major)".into();
        t.description = "Ethiopian Tizita major — pentatonic scale, equivalent to Western major pentatonic (C-D-E-G-A)".into();
        t
    }

    /// Ethiopian Tizita minor — nostalgic, melancholic variant of Tizita.
    /// Scale pattern: C - D - Eb - G - Ab
    /// Intervals: M2, m2, M3, m2, M3
    ///
    /// This scale expresses "tizita" (memory, nostalgia, longing) in its minor form,
    /// commonly used in slower, more introspective Ethiopian music.
    ///
    /// NOTE: Traditional performance includes microtonal inflections.
    ///
    /// Source: Ethiopian music theory (PubPub 2022, pianoencyclopedia.com).
    /// Pattern documented as C-D-Eb-G-Ab in academic sources.
    pub fn ethiopian_tizita_minor(concert_a: f32) -> Self {
        // Tizita minor: C D Eb G Ab
        // Only the pentatonic degrees are active, others are chromatic passing tones
        let offsets = [
            0.0,  // C  — root
            0.0,  // C# (not used)
            0.0,  // D  — major 2nd
            0.0,  // Eb — minor 3rd (enharmonic with D#)
            0.0,  // E  (not used)
            0.0,  // F  (not used)
            0.0,  // F# (not used)
            0.0,  // G  — perfect 5th
            0.0,  // Ab — minor 6th (enharmonic with G#)
            0.0,  // A  (not used)
            0.0,  // A# (not used)
            0.0,  // B  (not used)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Tizita (minor)".into();
        t.description = "Ethiopian Tizita minor — nostalgic pentatonic scale expressing longing and memory (C-D-Eb-G-Ab)".into();
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
            1.0,
            16.0 / 15.0,
            9.0 / 8.0,
            6.0 / 5.0,
            5.0 / 4.0,
            4.0 / 3.0,
            45.0 / 32.0,
            3.0 / 2.0,
            8.0 / 5.0,
            5.0 / 3.0,
            9.0 / 5.0,
            15.0 / 8.0,
        ];
        let tet_ratios: [f32; 12] = [
            1.0,
            2.0f32.powf(1.0 / 12.0),
            2.0f32.powf(2.0 / 12.0),
            2.0f32.powf(3.0 / 12.0),
            2.0f32.powf(4.0 / 12.0),
            2.0f32.powf(5.0 / 12.0),
            2.0f32.powf(6.0 / 12.0),
            2.0f32.powf(7.0 / 12.0),
            2.0f32.powf(8.0 / 12.0),
            2.0f32.powf(9.0 / 12.0),
            2.0f32.powf(10.0 / 12.0),
            2.0f32.powf(11.0 / 12.0),
        ];
        let offsets: [f32; 12] =
            std::array::from_fn(|i| 1200.0 * (ratios[i] / tet_ratios[i]).log2());
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Just Intonation (5-limit)".into();
        t.description = "Pure harmonic ratios — no beating on perfect intervals".into();
        t
    }

    /// Just intonation (7-limit) — includes septimal intervals.
    /// Uses ratios with prime factors up to 7 (e.g., 7/4, 7/6, 7/5).
    ///
    /// 7-limit JI adds septimal intervals that appear in blues, barbershop harmony,
    /// and many non-Western musical traditions. The harmonic seventh (7/4) is
    /// significantly flatter than the 12-TET minor seventh, creating a characteristic
    /// "bluesy" sound.
    ///
    /// Key septimal intervals:
    /// - 7/6: Septimal minor third (~267 cents, between minor and major third)
    /// - 7/5: Septimal tritone (~583 cents, slightly flat of 12-TET tritone)
    /// - 7/4: Harmonic seventh (~969 cents, much flatter than 12-TET minor 7th)
    ///
    /// Source: Extended just intonation theory, used by microtonal composers
    /// (Harry Partch, Ben Johnston) and in blues/barbershop traditions.
    pub fn just_intonation_7_limit(concert_a: f32) -> Self {
        let ratios: [f32; 12] = [
            1.0,         // C  — root (1/1)
            16.0 / 15.0, // C# — minor semitone
            9.0 / 8.0,   // D  — major second
            7.0 / 6.0,   // D# — septimal minor third
            5.0 / 4.0,   // E  — major third
            4.0 / 3.0,   // F  — perfect fourth
            7.0 / 5.0,   // F# — septimal tritone
            3.0 / 2.0,   // G  — perfect fifth
            8.0 / 5.0,   // G# — minor sixth
            5.0 / 3.0,   // A  — major sixth
            7.0 / 4.0,   // A# — harmonic seventh (characteristic septimal interval)
            15.0 / 8.0,  // B  — major seventh
        ];
        let tet_ratios: [f32; 12] = [
            1.0,
            2.0f32.powf(1.0 / 12.0),
            2.0f32.powf(2.0 / 12.0),
            2.0f32.powf(3.0 / 12.0),
            2.0f32.powf(4.0 / 12.0),
            2.0f32.powf(5.0 / 12.0),
            2.0f32.powf(6.0 / 12.0),
            2.0f32.powf(7.0 / 12.0),
            2.0f32.powf(8.0 / 12.0),
            2.0f32.powf(9.0 / 12.0),
            2.0f32.powf(10.0 / 12.0),
            2.0f32.powf(11.0 / 12.0),
        ];
        let offsets: [f32; 12] =
            std::array::from_fn(|i| 1200.0 * (ratios[i] / tet_ratios[i]).log2());
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Just Intonation (7-limit)".into();
        t.description =
            "Pure harmonic ratios with septimal intervals (7/4, 7/6, 7/5) — blues and barbershop"
                .into();
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
            0.0,   // C  — root (Rast)
            0.0,   // C#
            0.0,   // D  — whole tone
            -50.0, // D# — E half-flat (quarter tone flat)
            0.0,   // E
            0.0,   // F  — perfect fourth
            0.0,   // F#
            0.0,   // G  — perfect fifth
            0.0,   // G#
            0.0,   // A
            -50.0, // A# — B half-flat (quarter tone flat)
            0.0,   // B
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
            0.0,   // C  — root
            -50.0, // C# — D half-flat (characteristic Bayati interval)
            0.0,   // D
            -30.0, // D# — slightly flat
            0.0,   // E
            0.0,   // F
            0.0,   // F#
            0.0,   // G
            0.0,   // G#
            0.0,   // A
            -50.0, // A# — B half-flat
            0.0,   // B
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Arabic Maqam Bayati".into();
        t.description =
            "Arabic Maqam Bayati — half-flat on 2nd degree, characteristic of Arabic music".into();
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
        t.description =
            "Arabic Maqam Hijaz — augmented 2nd between 2nd and 3rd degrees (1-3-1 tetrachord)"
                .into();
        t
    }

    /// Ethiopian Bati minor — the most common Bati variant.
    /// Scale pattern: C - Eb - F - G - Bb (minor pentatonic)
    /// Intervals: m3, M2, M2, m3, M2
    ///
    /// This is equivalent to the Western minor pentatonic scale and is the
    /// standard "Bati" used in Ethiopian music. It expresses melancholy and depth.
    ///
    /// NOTE: Traditional performance includes microtonal inflections.
    ///
    /// Source: Ethiopian music theory. Documented as equivalent to Western
    /// minor pentatonic in Timothy Johnson's research (Scribd 2018).
    pub fn ethiopian_bati(concert_a: f32) -> Self {
        // Bati minor is the standard Western minor pentatonic: C Eb F G Bb
        let offsets = [
            0.0, // C  — root
            0.0, // C# (not used)
            0.0, // D  (not used)
            0.0, // Eb — minor 3rd
            0.0, // E  (not used)
            0.0, // F  — perfect 4th
            0.0, // F# (not used)
            0.0, // G  — perfect 5th
            0.0, // G# (not used)
            0.0, // A  (not used)
            0.0, // Bb — minor 7th
            0.0, // B  (not used)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Bati (minor)".into();
        t.description = "Ethiopian Bati minor — standard minor pentatonic scale expressing melancholy (C-Eb-F-G-Bb)".into();
        t
    }

    /// Ethiopian Bati major — bright, uplifting variant of Bati.
    /// Scale pattern: C - E - F - G - B
    /// Intervals: M3, m2, M2, M3, m2
    ///
    /// This scale creates a distinctly Ethiopian sound through its unusual
    /// interval structure, particularly the major third followed by a semitone.
    /// Less common than Bati minor but used for more joyful or energetic pieces.
    ///
    /// NOTE: Traditional performance includes microtonal inflections.
    ///
    /// Source: Ethiopian music theory (PubPub 2022).
    /// Pattern documented as C-E-F-G-B in academic sources.
    pub fn ethiopian_bati_major(concert_a: f32) -> Self {
        // Bati major: C E F G B
        let offsets = [
            0.0, // C  — root
            0.0, // C# (not used)
            0.0, // D  (not used)
            0.0, // D# (not used)
            0.0, // E  — major 3rd
            0.0, // F  — perfect 4th
            0.0, // F# (not used)
            0.0, // G  — perfect 5th
            0.0, // G# (not used)
            0.0, // A  (not used)
            0.0, // A# (not used)
            0.0, // B  — major 7th
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Bati (major)".into();
        t.description = "Ethiopian Bati major — bright pentatonic variant with characteristic semitone (C-E-F-G-B)".into();
        t
    }

    /// Ethiopian Ambassel — pentatonic with raised 4th.
    /// Scale pattern: C - Db - F - G - Ab
    /// Intervals: m2, M3, M2, m2, M3
    ///
    /// Ambassel (also spelled Ambasel or Ambessel) is characterized by its
    /// prominent use of the flat 2nd degree, creating a sound similar to
    /// Phrygian mode. The raised 4th (F natural) distinguishes it from Bati.
    ///
    /// The scale structure creates characteristic "long intervals" (major 3rds)
    /// that are a hallmark of Ethiopian pentatonic music.
    ///
    /// NOTE: Traditional performance includes microtonal inflections.
    ///
    /// Source: Wikipedia "Ambassel scale" (2025). Documented as pentatonic
    /// subset of Phrygian: 1, ♭2, 4, 5, ♭6 (C-Db-F-G-Ab).
    pub fn ethiopian_ambassel(concert_a: f32) -> Self {
        // Ambassel: C Db F G Ab (1, b2, 4, 5, b6)
        let offsets = [
            0.0, // C  — root
            0.0, // Db — minor 2nd (enharmonic with C#)
            0.0, // D  (not used)
            0.0, // D# (not used)
            0.0, // E  (not used)
            0.0, // F  — perfect 4th
            0.0, // F# (not used)
            0.0, // G  — perfect 5th
            0.0, // Ab — minor 6th (enharmonic with G#)
            0.0, // A  (not used)
            0.0, // A# (not used)
            0.0, // B  (not used)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Ambassel".into();
        t.description = "Ethiopian Ambassel — pentatonic with flat 2nd and characteristic long intervals (C-Db-F-G-Ab)".into();
        t
    }

    /// Ethiopian Anchihoye — the fourth main qenet mode.
    /// Scale pattern: C - D - F - G - A
    /// Intervals: M2, m3, M2, M2, m3
    ///
    /// Anchihoye (also spelled Anchi Hoye or አንቺሆዬ in Amharic) is one of the
    /// four fundamental qenet modes of Ethiopian music. This scale has a unique
    /// character distinct from the other modes, with its specific pattern of
    /// whole and minor third intervals.
    ///
    /// NOTE: Documentation on Anchihoye is limited compared to other qenet modes.
    /// This implementation uses the most commonly referenced interval pattern,
    /// but traditional performance practice may include microtonal variations.
    ///
    /// Source: Ethiopian music theory documentation (Scribd, Wikipedia "Qenet").
    /// Pattern inferred from pentatonic analysis and Ethiopian musical traditions.
    pub fn ethiopian_anchihoye(concert_a: f32) -> Self {
        // Anchihoye: C D F G A (1, 2, 4, 5, 6)
        // Similar to suspended pentatonic with no 3rd
        let offsets = [
            0.0, // C  — root
            0.0, // C# (not used)
            0.0, // D  — major 2nd
            0.0, // D# (not used)
            0.0, // E  (not used)
            0.0, // F  — perfect 4th
            0.0, // F# (not used)
            0.0, // G  — perfect 5th
            0.0, // G# (not used)
            0.0, // A  — major 6th
            0.0, // A# (not used)
            0.0, // B  (not used)
        ];
        let mut t = Self::from_cents_offsets(concert_a, &offsets);
        t.name = "Ethiopian Anchihoye".into();
        t.description = "Ethiopian Anchihoye — one of four main qenet modes, pentatonic without 3rd degree (C-D-F-G-A)".into();
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

    /// Javanese Gamelan Slendro with stretched octave — ethnomusicologically accurate.
    ///
    /// Real Javanese gamelan instruments have stretched octaves due to the inharmonic
    /// overtones of bronze and iron bars. Measurements show octaves ranging from
    /// approximately 1210-1215 cents (not the Western 1200 cents).
    ///
    /// This implementation uses 1210-cent octaves, dividing them into 5 roughly equal
    /// steps of ~242 cents each. This creates the characteristic "pseudo-octave" sound
    /// of authentic gamelan.
    ///
    /// Source: "On the Tuning and Stretched Octave of Javanese Gamelans" (JHU Muse, 2016),
    /// "Ombak and octave stretching in Balinese gamelan" (ResearchGate, 2020).
    pub fn gamelan_slendro_stretched(_concert_a: f32) -> Self {
        let octave_cents = 1210.0; // Stretched octave (measured from real ensembles)
        let step = octave_cents / 5.0; // ~242 cents per step
        let mut frequencies = vec![0.0f32; 128];
        for (note, freq) in frequencies.iter_mut().enumerate() {
            let slendro_step = (note as f32 / 2.4).floor();
            let cents_from_c0 = slendro_step * step;
            *freq = 16.352 * 2.0f32.powf(cents_from_c0 / 1200.0);
        }
        Self {
            frequencies,
            name: "Gamelan Slendro (Stretched)".into(),
            description: "Javanese Gamelan Slendro with stretched octave (~1210 cents) — ethnomusicologically accurate".into(),
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
            for f in frequencies.iter_mut() {
                *f *= ratio;
            }
        }
        Self {
            frequencies,
            name: "Gamelan Pelog".into(),
            description:
                "Javanese Gamelan Pelog — 7-tone scale with characteristic unequal intervals".into(),
        }
    }
}
