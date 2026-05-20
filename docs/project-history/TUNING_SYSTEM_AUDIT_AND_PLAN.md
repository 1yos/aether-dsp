# Tuning System Audit and Correction Plan

**Date:** May 20, 2026  
**Status:** Research Complete — Ready for Implementation  
**Scope:** Comprehensive audit of all tuning system implementations and documentation

---

## Executive Summary

After comprehensive research and code evaluation, this document identifies **2 critical documentation errors**, **4 musicological accuracy issues**, and **12 enhancement opportunities** in the AetherDSP tuning system implementation.

**Critical Finding:** Documentation claims 14 tuning systems exist, but only 9 are implemented. Two documented systems (Maqam Hijaz, Ethiopian Ambassel) are completely missing from code.

---

## Part 1: Critical Issues (Fix Immediately)

### 1.1 Missing Implementations (CRITICAL)

#### Issue: Maqam Hijaz Documented But Not Implemented

- **Severity:** Critical (breaks user trust)
- **Evidence:**
  - Listed in README.md line 104: "Arabic Maqam Hijaz | Augmented 2nd between 2nd and 3rd degrees"
  - Listed in CHANGELOG.md, RELEASE_NOTES, and 3 other docs
  - **NOT FOUND** in `crates/aether-midi/src/tuning.rs`
  - Grep search confirms: 0 matches in Rust code

**Research Findings:**

- Hijaz tetrachord structure: **semitone - augmented 2nd - semitone** (1-3-1 pattern)
- In cents from root: 0, 100, 400, 500 (characteristic augmented 2nd = 300 cents)
- Source: Wikipedia "Phrygian dominant scale", maqamworld.com
- Also called "Freygish" in Jewish music, "Phrygian dominant" in Western theory

**Action Required:**

```rust
/// Arabic Maqam Hijaz — characteristic augmented 2nd.
/// Tetrachord pattern: semitone - augmented 2nd - semitone (1-3-1).
pub fn arabic_maqam_hijaz(concert_a: f32) -> Self {
    let offsets = [
        0.0,    // C  — root
        0.0,    // C# — semitone
        0.0,    // D
        0.0,    // D# — augmented 2nd from D to E (300 cents)
        0.0,    // E
        0.0,    // F  — semitone
        0.0,    // F#
        0.0,    // G
        0.0,    // G#
        0.0,    // A
        0.0,    // A#
        0.0,    // B
    ];
    let mut t = Self::from_cents_offsets(concert_a, &offsets);
    t.name = "Arabic Maqam Hijaz".into();
    t.description = "Arabic Maqam Hijaz — augmented 2nd between 2nd and 3rd degrees (1-3-1 tetrachord)".into();
    t
}
```

---

#### Issue: Ethiopian Ambassel Documented But Not Implemented

- **Severity:** Critical (breaks user trust)
- **Evidence:**
  - Listed in README.md line 101: "Ethiopian Ambassel | Pentatonic with raised 4th"
  - Listed in CHANGELOG.md, RELEASE_NOTES, and 3 other docs
  - **NOT FOUND** in `crates/aether-midi/src/tuning.rs`
  - Grep search confirms: 0 matches in Rust code

**Research Findings:**

- Ambassel (also spelled Ambasel/Ambessel) is one of the 4 main Ethiopian qenet (modes)
- Structure from Scribd Ethiopian music scales document: **C - D - F - G - A** (pentatonic)
- Intervals: whole tone, minor 3rd, whole tone, whole tone
- In cents: 0, 200, 500, 700, 900
- Characteristic: Raised 4th (F instead of Eb), distinguishes it from Bati

**Action Required:**

```rust
/// Ethiopian Ambassel — pentatonic with raised 4th.
/// One of the four main Ethiopian qenet (modal scales).
pub fn ethiopian_ambassel(concert_a: f32) -> Self {
    let offsets = [
        0.0,    // C  — root
        0.0,    // C#
        0.0,    // D  — whole tone
        0.0,    // D#
        0.0,    // E
        0.0,    // F  — raised 4th (characteristic interval)
        0.0,    // F#
        0.0,    // G  — perfect 5th
        0.0,    // G#
        0.0,    // A  — major 6th
        0.0,    // A#
        0.0,    // B
    ];
    let mut t = Self::from_cents_offsets(concert_a, &offsets);
    t.name = "Ethiopian Ambassel".into();
    t.description = "Ethiopian Ambassel — pentatonic with raised 4th, one of four main qenet modes".into();
    t
}
```

---

### 1.2 Documentation Cleanup Required

**Files to Update:**

1. `README.md` — Either implement missing tunings OR remove from table
2. `crates/aether-midi/CHANGELOG.md` — Update tuning count (9 actual, not 14)
3. `docs/releases/RELEASE_NOTES_v0.1.5.md` — Update tuning list
4. `docs/project-history/CRATES_IO_PUBLISHED_CONTENT.md` — Update claims
5. `docs/design/IMPLEMENTATION_PLAN.md` — Mark Hijaz/Ambassel as TODO

**Recommendation:** Implement both tunings (30 minutes work) rather than removing from docs, since they're already promised to users.

---

## Part 2: Musicological Accuracy Issues (Fix Soon)

### 2.1 Gamelan Slendro Octave Stretching

**Current Implementation (Line 267-278 in tuning.rs):**

```rust
pub fn gamelan_slendro(_concert_a: f32) -> Self {
    let step = 1200.0 / 5.0; // 240 cents per step
    let mut frequencies = vec![0.0f32; 128];
    for (note, freq) in frequencies.iter_mut().enumerate() {
        let slendro_step = (note as f32 / 2.4).floor();
        let cents_from_c0 = slendro_step * step;
        *freq = 16.352 * 2.0f32.powf(cents_from_c0 / 1200.0);
    }
    // ...
}
```

**Problem:** Uses exact 2:1 octave ratio (1200 cents). Real gamelan instruments have stretched octaves.

**Research Findings:**

- Source: "On the Tuning and Stretched Octave of Javanese Gamelans" (JHU Muse, 2016)
- Source: "Ombak and octave stretching in Balinese gamelan" (ResearchGate, 2020)
- Finding: "Both Javanese scales are considerably stretched" relative to Western tuning
- Measurement: Octaves range from ~1210-1215 cents (not 1200)
- Cause: Inharmonic overtones of bronze/iron bars create perceptual "pseudo-octave"

**Severity:** Medium (musicologically incorrect, but not immediately noticeable)

**Correction Plan:**

1. Add `stretch_ratio` parameter to tuning system
2. Default Slendro to 1210 cents per octave (1.75% stretch)
3. Document that this is an approximation of measured ensembles
4. Consider adding multiple Slendro variants (Javanese vs Balinese)

**Implementation:**

```rust
/// Javanese Gamelan Slendro — 5-tone scale with stretched octave.
/// Note: Real gamelan ensembles have stretched octaves (~1210 cents)
/// due to inharmonic overtones. This approximation uses 1210-cent octaves.
pub fn gamelan_slendro_stretched(concert_a: f32) -> Self {
    let octave_cents = 1210.0; // Stretched octave (measured from ensembles)
    let step = octave_cents / 5.0; // ~242 cents per step
    // ... rest of implementation
}
```

---

### 2.2 Ethiopian Bati Intervals Undocumented

**Current Implementation (Line 221-233 in tuning.rs):**

```rust
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
    // ...
}
```

**Problems:**

1. No source attribution for -20/-30/-20 cent offsets
2. Description says "minor pentatonic variant" but uses 12-note offsets
3. No reference to Ethiopian musicology literature

**Research Findings:**

- Source: Chromatone.center Ethiopian scales page
- Finding: "Ethiopian music is largely pentatonic... with microtonal or flexible intonation"
- Finding: "Kinit modes are not defined strictly by interval relationships... but by melodic contours"
- Source: Scribd "Ethiopian Music Scales" document
- Bati Major structure: **C - E - F - G - B** (intervals: M3, m2, M2, M3, m2)
- Bati Minor structure: Not clearly documented in accessible sources

**Severity:** Medium (approximation without source, but not necessarily wrong)

**Correction Plan:**

1. Research Ashenafi Kebede's work (Ethiopian musicologist, 1970s-1990s)
2. Consult with Ethiopian musicians or AAU music department
3. Add source attribution comment or mark as "approximation"
4. Consider implementing both Bati Major and Bati Minor as separate functions

**Interim Fix:**

```rust
/// Ethiopian Bati — minor pentatonic variant.
///
/// NOTE: This is an approximation. Ethiopian qenet (modal scales) are defined
/// more by melodic contour and ornamentation than fixed interval ratios.
/// The cent offsets used here (-20/-30/-20) are estimates based on performance
/// practice, not documented measurements.
///
/// TODO: Validate with Ethiopian musicologists or Kebede's research.
pub fn ethiopian_bati(concert_a: f32) -> Self {
    // ... existing implementation with added documentation
}
```

---

### 2.3 Arabic Quarter-Tones: 24-TET vs Historical Ratios

**Current Implementation:**

- Maqam Rast: Uses -50 cent offsets (24-TET approximation)
- Maqam Bayati: Uses -50 cent offsets (24-TET approximation)

**Research Findings:**

- Source: Wikipedia "Arab tone system"
- Modern practice: "24-tone equal temperament... quarter tone (50 cents)"
- Historical practice: Al-Farabi and Al-Urmawi used ratio-based systems
- Source: 53music.us "Arabic Theorists"
- Finding: "Musicians themselves tend to instinctively hit the 'sweet spots' which actually land near the quarter tone only when evoking 11-limit harmonies"
- Conclusion: 24-TET is a **modern theoretical framework**, not historical practice

**Severity:** Low (24-TET is widely used in modern Arabic music education)

**Recommendation:**

- Keep current 24-TET implementations (they're correct for modern practice)
- Add historical ratio-based variants as separate functions (future enhancement)
- Document that these are 24-TET approximations

**Documentation Fix:**

```rust
/// Arabic Maqam Rast — the most common Arabic maqam.
/// Uses quarter-tone flats on the 3rd and 7th scale degrees.
///
/// NOTE: This implementation uses 24-TET (50-cent quarter-tones), which is
/// the modern theoretical standard. Historical Arabic music theory (al-Farabi,
/// al-Urmawi) used ratio-based intervals. Performance practice often deviates
/// from both systems based on melodic context.
pub fn arabic_maqam_rast(concert_a: f32) -> Self {
    // ... existing implementation
}
```

---

### 2.4 Gamelan Pelog: No Ensemble Source

**Current Implementation (Line 281-297 in tuning.rs):**

```rust
pub fn gamelan_pelog(concert_a: f32) -> Self {
    // Pelog has 7 tones with unequal steps. Common approximation in cents from root:
    // 0, 120, 270, 540, 675, 785, 950, 1200
    let pelog_cents = [0.0f32, 120.0, 270.0, 540.0, 675.0, 785.0, 950.0];
    // ...
}
```

**Problem:** No source for these specific cent values. Pelog varies dramatically between ensembles.

**Research Findings:**

- Source: ResearchGate "Javanese Pelog Tunings Reconsidered" (1980)
- Finding: "The absolute pitches of various ensembles differ substantially"
- Finding: "Within an ensemble, tones of a given name vary greatly in pitch when produced on different instruments"
- Source: MIT "Exploring the Many Tunings of Balinese Gamelan" (2023)
- Finding: "Indonesian gamelan and Western tuning systems are based on fundamentally different concepts"

**Severity:** Low (any single Pelog tuning is inherently an approximation)

**Correction Plan:**

1. Add comment noting this is a generic approximation
2. Consider adding multiple Pelog variants from documented ensembles
3. Add source attribution if specific ensemble measurements are used

**Documentation Fix:**

```rust
/// Javanese Gamelan Pelog — 7-tone scale with characteristic unequal intervals.
///
/// NOTE: Pelog tuning varies dramatically between gamelan ensembles. This is
/// a generic approximation using commonly cited interval patterns. Real gamelan
/// instruments are tuned individually and not intended to match Western pitch
/// standards or other ensembles.
///
/// For authentic reproduction, measure a specific ensemble or use documented
/// measurements from ethnomusicological studies.
pub fn gamelan_pelog(concert_a: f32) -> Self {
    // ...
}
```

---

## Part 3: Code Quality Issues

### 3.1 Just Intonation: Only 5-Limit Implemented

**Current Implementation:**

```rust
pub fn just_intonation(concert_a: f32) -> Self {
    let ratios: [f32; 12] = [
        1.0, 16.0/15.0, 9.0/8.0, 6.0/5.0, 5.0/4.0, 4.0/3.0,
        45.0/32.0, 3.0/2.0, 8.0/5.0, 5.0/3.0, 9.0/5.0, 15.0/8.0,
    ];
    // ...
}
```

**Analysis:**

- Uses ratios with prime factors up to 5 (5-limit JI)
- Common intervals: 5/4 (major third), 3/2 (perfect fifth)
- Missing: 7-limit intervals like 7/4 (harmonic seventh), 7/6 (septimal minor third)

**Research Findings:**

- 5-limit JI: Traditional Western just intonation
- 7-limit JI: Adds septimal intervals, used in blues, barbershop, non-Western music
- 11-limit and higher: Microtonal composers (Ben Johnston, Harry Partch)

**Severity:** Low (5-limit is correct for what it claims to be)

**Enhancement Opportunity:**

```rust
/// Just intonation (5-limit) — pure intervals based on harmonic series.
/// Uses ratios with prime factors up to 5 (3/2, 5/4, etc.).
pub fn just_intonation_5_limit(concert_a: f32) -> Self {
    // ... existing implementation
}

/// Just intonation (7-limit) — includes septimal intervals.
/// Adds intervals like 7/4 (harmonic seventh) and 7/6 (septimal minor third).
/// Used in blues, barbershop harmony, and many non-Western traditions.
pub fn just_intonation_7_limit(concert_a: f32) -> Self {
    let ratios: [f32; 12] = [
        1.0, 16.0/15.0, 9.0/8.0, 7.0/6.0, 5.0/4.0, 4.0/3.0,
        7.0/5.0, 3.0/2.0, 8.0/5.0, 5.0/3.0, 7.0/4.0, 15.0/8.0,
    ];
    // ...
}
```

---

### 3.2 Ethiopian Tizita: No Source Attribution

**Current Implementation:**

```rust
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
    // ...
}
```

**Research Findings:**

- Tizita is one of the four main Ethiopian qenet
- Structure from research: **C - D - E - G - A** (pentatonic major)
- Intervals: whole tone, whole tone, minor 3rd, whole tone
- In cents: 0, 200, 400, 700, 900

**Problem:** Current implementation uses 12-note cent offsets, but Tizita is pentatonic. The offsets don't match documented pentatonic structure.

**Severity:** Medium (implementation may not match actual Ethiopian practice)

**Correction Needed:**

1. Verify Tizita structure with Ethiopian sources
2. Either fix offsets or document why 12-note mapping is used
3. Add source attribution

---

## Part 4: API and Architecture Issues

### 4.1 TuningTable IS Public API ✅

**Verification:**

```rust
// crates/aether-midi/src/lib.rs line 18
pub use tuning::TuningTable;
```

**Status:** Already correct. No action needed.

---

### 4.2 Pitch-Bend Interaction: Undefined Behavior

**Current State:**

- Oscillator uses tuning table to map MIDI note → frequency
- No documented behavior for pitch-bend + microtonal tuning interaction

**Problem Scenario:**

```
User plays MIDI note 60 (C) in Maqam Hijaz
Oscillator outputs frequency from tuning table: 261.63 Hz
User applies +200 cent pitch-bend
Question: Does bend operate in:
  A) Cents relative to tuned pitch (261.63 Hz + 200 cents)?
  B) 12-TET cents (landing on D in 12-TET, breaking the maqam)?
```

**Severity:** Medium (undefined behavior, could break microtonal music)

**Recommendation:**

- Pitch-bend should operate **relative to the tuned pitch** (option A)
- Document this behavior clearly
- Add test cases

**Implementation Note:**

```rust
// In MIDI engine or oscillator:
// When pitch-bend is applied:
let base_freq = tuning_table.frequency(midi_note);
let bent_freq = base_freq * 2.0f32.powf(pitch_bend_cents / 1200.0);
// This preserves the tuning system's intervals
```

---

### 4.3 f32 Precision Limits: Undocumented

**Current Implementation:**

- All frequencies stored as `f32`
- Precision: ~7 decimal digits (~0.0001 Hz at 440 Hz)

**Analysis:**

- At 20 Hz: precision = 0.000002 Hz (excellent)
- At 20,000 Hz: precision = 0.002 Hz (still excellent)
- Conclusion: f32 is sufficient for all audio applications

**Severity:** Very Low (not a real problem, just undocumented)

**Documentation Fix:**

```rust
/// Maps MIDI note numbers (0–127) to frequencies in Hz.
///
/// Frequencies are stored as f32, providing ~0.0001 Hz precision at 440 Hz.
/// This is more than sufficient for audio applications (human pitch
/// discrimination is ~1 Hz at best). For extreme low-frequency accuracy
/// (<1 Hz), consider using f64 in custom implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningTable {
    /// Frequency in Hz for each MIDI note 0–127.
    pub frequencies: Vec<f32>,
    // ...
}
```

---

## Part 5: Missing Features (Enhancements)

### 5.1 Scala (.scl) File Import

**Status:** Not implemented  
**Priority:** High (interoperability with microtonal ecosystem)  
**Effort:** Medium (2-3 hours)

**Benefit:** Access to 5000+ community-defined scales from Scala archive.

**Implementation Sketch:**

```rust
impl TuningTable {
    /// Load tuning from Scala (.scl) file format.
    ///
    /// Scala is the universal format for microtonal tunings, with thousands
    /// of scales available at http://www.huygens-fokker.org/scala/
    pub fn from_scala_file(path: &Path, concert_a: f32) -> Result<Self, ScalaError> {
        // Parse .scl format:
        // - Line 1: Description
        // - Line 2: Number of notes
        // - Lines 3+: Cents or ratios
        // ...
    }
}
```

---

### 5.2 Source Attribution System

**Status:** Not implemented  
**Priority:** High (builds trust, enables corrections)  
**Effort:** Low (1 hour)

**Implementation:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningTable {
    pub frequencies: Vec<f32>,
    pub name: String,
    pub description: String,
    /// Source attribution: research paper, recording, or "approximation"
    pub source: Option<String>,
    /// ISO 639-1 language code for cultural context
    pub culture: Option<String>,
}
```

**Example Usage:**

```rust
let mut t = Self::from_cents_offsets(concert_a, &offsets);
t.name = "Arabic Maqam Rast".into();
t.description = "Quarter-tone flats on 3rd and 7th degrees".into();
t.source = Some("24-TET approximation (modern practice)".into());
t.culture = Some("ar".into()); // Arabic
```

---

### 5.3 Multiple Regional Variants

**Status:** Only single variant per tradition  
**Priority:** Medium  
**Effort:** Medium (research required)

**Examples:**

- Pelog: Add 3-4 variants from documented ensembles
- Rast: Add Turkish Rast vs Arabic Rast
- JI: Add 5-limit, 7-limit, 11-limit variants
- Slendro: Add Javanese vs Balinese variants

---

### 5.4 Tuning-Aware Filter Tracking

**Status:** Not implemented  
**Priority:** Low (advanced feature)  
**Effort:** High (requires filter redesign)

**Problem:** When using Moog filter with microtonal tuning, key-tracking still uses 12-TET intervals.

**Solution:** Pass tuning table to filter nodes, use tuned frequency for key-tracking.

---

## Part 6: Strategic Recommendations

### 6.1 Consult Ethiopian Musicians

**Priority:** HIGHEST  
**Effort:** Low (1-2 meetings)  
**Impact:** Validates or corrects all Ethiopian tuning implementations

**Action Items:**

1. Contact Addis Ababa University music department
2. Arrange consultation with krar or washint player
3. Record actual instrument tunings with spectrum analyzer
4. Update Tizita/Bati/Ambassel implementations based on findings

---

### 6.2 Add Ethiopian Instrument Samples

**Priority:** High (completes the Ethiopian feature set)  
**Effort:** High (requires recording or licensing)

**Current State:**

- 3 Ethiopian tuning systems implemented
- 0 Ethiopian instrument samples
- Only Western drum/piano samples exist

**Recommendation:** Record or license basic krar multi-samples to make Ethiopian tunings actually usable.

---

### 6.3 Create CONTRIBUTING Guide for Tunings

**Priority:** Medium  
**Effort:** Low (2 hours)

**Content:**

- JSON format specification
- Source attribution requirements
- How to submit new tunings
- Validation checklist

---

## Part 7: Implementation Priority Matrix

### Phase 1: Critical Fixes (Do Now)

1. ✅ Implement `arabic_maqam_hijaz()` function
2. ✅ Implement `ethiopian_ambassel()` function
3. ✅ Update all documentation to match actual implementation
4. ✅ Add source attribution comments to all existing tunings
5. ✅ Document pitch-bend behavior
6. ✅ Document f32 precision limits

**Estimated Time:** 4 hours  
**Impact:** Fixes broken promises, builds trust

---

### Phase 2: Musicological Corrections (Do Soon)

1. ⚠️ Research and fix Gamelan Slendro octave stretching
2. ⚠️ Consult Ethiopian musicians about Bati/Tizita intervals
3. ⚠️ Add documentation about 24-TET vs historical Arabic tunings
4. ⚠️ Add ensemble source notes to Pelog implementation

**Estimated Time:** 8 hours (including research)  
**Impact:** Improves authenticity and accuracy

---

### Phase 3: Enhancements (Do Later)

1. 🔧 Implement Scala (.scl) file import
2. 🔧 Add 7-limit Just Intonation
3. 🔧 Add source attribution fields to TuningTable struct
4. 🔧 Add multiple regional variants (Pelog, Rast, etc.)
5. 🔧 Create CONTRIBUTING guide for tunings

**Estimated Time:** 16 hours  
**Impact:** Expands capabilities, community engagement

---

### Phase 4: Advanced Features (Future)

1. 🚀 Tuning-aware filter tracking
2. 🚀 Microtonal MIDI export with pitch-bend
3. 🚀 Visual scale reference UI
4. 🚀 Record Ethiopian instrument samples
5. 🚀 VST3/CLAP plugin wrapper

**Estimated Time:** 40+ hours  
**Impact:** Professional-grade feature completeness

---

## Part 8: Research Sources

### Academic Papers

1. "On the Tuning and Stretched Octave of Javanese Gamelans" (JHU Muse, 2016)
2. "Ombak and octave stretching in Balinese gamelan" (ResearchGate, 2020)
3. "Javanese Pelog Tunings Reconsidered" (ResearchGate, 1980)
4. "Exploring the Many Tunings of Balinese Gamelan" (MIT, 2023)

### Online Resources

1. Wikipedia: "Arab tone system", "Phrygian dominant scale", "Quarter tone"
2. Maqamworld.com: Arabic maqam reference
3. Chromatone.center: Ethiopian scales documentation
4. Scribd: "Ethiopian Music Scales" document
5. 53music.us: "Arabic Theorists" historical context

### Recommended Further Reading

1. Ashenafi Kebede: Ethiopian musicology research (1970s-1990s)
2. Al-Farabi: Historical Arabic music theory
3. Mikhail Mishaqa: "Virtual founder of 24-TET quarter-tone scale"
4. Roger Vetter: "A Retrospect on a Century of Gamelan Tone Measurements" (1989)

---

## Conclusion

The AetherDSP tuning system has a solid foundation but suffers from:

1. **Documentation drift** (claiming features that don't exist)
2. **Missing source attribution** (makes validation impossible)
3. **Approximations without disclosure** (breaks trust with expert users)

**Recommended Immediate Action:**

1. Implement the 2 missing tunings (Hijaz, Ambassel) — 2 hours
2. Add source attribution comments to all tunings — 1 hour
3. Update all documentation to match reality — 1 hour

**Total Time to Fix Critical Issues:** 4 hours

This will bring the project from "promising but incomplete" to "trustworthy and extensible."

---

**Next Steps:** Proceed with Phase 1 implementation?
