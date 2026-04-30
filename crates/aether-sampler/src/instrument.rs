//! Sampler instrument definition.
//!
//! An instrument is a collection of sample zones.
//! Each zone maps a note range + velocity range to an audio file.

use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::buffer::SampleBuffer;
use aether_midi::tuning::TuningTable;

/// Round-robin selection strategy for zone groups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundRobinMode {
    /// Cycle through zones in order.
    Sequential,
    /// Select a random zone on each note-on.
    Random,
    /// Select a random zone, but never repeat the previous zone.
    RandomNoRepeat,
}

/// A group of sample zones that share the same note/velocity range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneGroup {
    /// The zones in this group (round-robin variations).
    pub zones: Vec<SampleZone>,
    /// Selection strategy for this group.
    pub mode: RoundRobinMode,
}

/// Default seed for RoundRobinState RNG.
const DEFAULT_RR_SEED: u64 = 0x123456789ABCDEF0u64;

/// Round-robin state for zone selection (stored in SamplerNode, not in SamplerInstrument).
#[derive(Debug, Clone)]
pub struct RoundRobinState {
    /// Per-group sequential index. Key: group index in instrument.zone_groups.
    sequential_index: HashMap<usize, usize>,
    /// Per-group last-selected index (for RandomNoRepeat).
    last_selected: HashMap<usize, usize>,
    /// Simple LCG for allocation-free pseudo-random selection.
    rng_state: u64,
    /// The initial seed (for reset).
    seed: u64,
}

impl RoundRobinState {
    /// Create a new round-robin state with the default seed.
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_RR_SEED)
    }
}

impl Default for RoundRobinState {
    fn default() -> Self {
        Self::new()
    }
}

impl RoundRobinState {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            sequential_index: HashMap::new(),
            last_selected: HashMap::new(),
            rng_state: seed,
            seed,
        }
    }

    /// Reset the state to initial conditions.
    pub fn reset(&mut self) {
        self.sequential_index.clear();
        self.last_selected.clear();
        self.rng_state = self.seed;
    }

    /// Xorshift64 pseudo-random number generator.
    pub fn xorshift64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    /// Select a zone index from a group according to the mode.
    /// Returns the index (0..group_len) of the selected zone.
    pub fn select(
        &mut self,
        group_idx: usize,
        group_len: usize,
        mode: &RoundRobinMode,
    ) -> usize {
        if group_len == 0 {
            return 0;
        }

        let n = group_len;
        match mode {
            RoundRobinMode::Sequential => {
                let current = self.sequential_index.get(&group_idx).copied().unwrap_or(0);
                let selected = current % n;
                self.sequential_index.insert(group_idx, (current + 1) % n);
                selected
            }
            RoundRobinMode::Random => (self.xorshift64() % n as u64) as usize,
            RoundRobinMode::RandomNoRepeat => {
                if n == 1 {
                    0
                } else {
                    let last = self.last_selected.get(&group_idx).copied();
                    let mut selected = (self.xorshift64() % n as u64) as usize;
                    // Try up to N times to avoid the last selection
                    for _ in 0..n {
                        if Some(selected) != last {
                            break;
                        }
                        selected = (self.xorshift64() % n as u64) as usize;
                    }
                    // Fallback: if all attempts collided, just pick (last + 1) % n
                    if Some(selected) == last {
                        selected = (last.unwrap() + 1) % n;
                    }
                    self.last_selected.insert(group_idx, selected);
                    selected
                }
            }
        }
    }

    /// Internal helper for backward compatibility.
    fn next_random(&mut self) -> u64 {
        self.xorshift64()
    }

    /// Select a zone from a group according to the mode.
    pub fn select_zone<'a>(
        &mut self,
        group_idx: usize,
        zones: &'a [SampleZone],
        mode: &RoundRobinMode,
    ) -> Option<&'a SampleZone> {
        if zones.is_empty() {
            return None;
        }

        let n = zones.len();
        let idx = match mode {
            RoundRobinMode::Sequential => {
                let current = self.sequential_index.get(&group_idx).copied().unwrap_or(0);
                let selected = current % n;
                self.sequential_index.insert(group_idx, (current + 1) % n);
                selected
            }
            RoundRobinMode::Random => (self.next_random() % n as u64) as usize,
            RoundRobinMode::RandomNoRepeat => {
                if n == 1 {
                    0
                } else {
                    let last = self.last_selected.get(&group_idx).copied();
                    let mut selected = (self.next_random() % n as u64) as usize;
                    // Try up to N times to avoid the last selection
                    for _ in 0..n {
                        if Some(selected) != last {
                            break;
                        }
                        selected = (self.next_random() % n as u64) as usize;
                    }
                    // Fallback: if all attempts collided, just pick (last + 1) % n
                    if Some(selected) == last {
                        selected = (last.unwrap() + 1) % n;
                    }
                    self.last_selected.insert(group_idx, selected);
                    selected
                }
            }
        };

        zones.get(idx)
    }
}

/// How a sample behaves during playback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArticulationType {
    /// Play once and stop (staccato, pluck).
    OneShot,
    /// Loop while key is held, release when key is lifted.
    SustainLoop {
        /// Frame to loop back to when reaching loop_end.
        loop_start: usize,
        /// Frame where the loop ends and jumps back to loop_start.
        loop_end: usize,
    },
    /// Play forward until key release, then crossfade to release sample.
    SustainRelease,
}

/// A single sample zone — maps a note/velocity range to an audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleZone {
    /// Unique identifier.
    pub id: String,
    /// Path to the audio file.
    pub file_path: String,
    /// The MIDI note this sample was recorded at (root pitch).
    pub root_note: u8,
    /// Lowest MIDI note this zone responds to.
    pub note_low: u8,
    /// Highest MIDI note this zone responds to.
    pub note_high: u8,
    /// Minimum velocity (0–127) this zone responds to.
    pub velocity_low: u8,
    /// Maximum velocity (0–127) this zone responds to.
    pub velocity_high: u8,
    /// Playback behavior.
    pub articulation: ArticulationType,
    /// Volume trim in dB (0.0 = no change).
    pub volume_db: f32,
    /// Tune offset in cents (0.0 = no change).
    pub tune_cents: f32,
    /// Optional separate release sample file.
    pub release_file: Option<String>,
}

impl SampleZone {
    /// Does this zone respond to the given note and velocity?
    pub fn matches(&self, note: u8, velocity: u8) -> bool {
        note >= self.note_low && note <= self.note_high
            && velocity >= self.velocity_low && velocity <= self.velocity_high
    }

    /// Pitch ratio to shift from root_note to target_note.
    pub fn pitch_ratio(&self, target_note: u8, tuning: &TuningTable) -> f32 {
        let root_freq = tuning.frequency(self.root_note);
        let target_freq = tuning.frequency(target_note);
        if root_freq > 0.0 {
            let cents_offset = self.tune_cents;
            (target_freq / root_freq) * 2.0f32.powf(cents_offset / 1200.0)
        } else {
            1.0
        }
    }

    /// Linear volume multiplier from volume_db.
    pub fn volume_linear(&self) -> f32 {
        10.0f32.powf(self.volume_db / 20.0)
    }
}

/// A complete sampler instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SamplerInstrumentRaw")]
pub struct SamplerInstrument {
    /// Instrument name.
    pub name: String,
    /// Cultural/geographic origin (e.g. "Ethiopian", "Indian", "Western").
    pub origin: String,
    /// Description.
    pub description: String,
    /// Author.
    pub author: String,
    /// Tuning system.
    pub tuning: TuningTable,
    /// All sample zones (legacy flat list, kept for backward compatibility).
    #[serde(default)]
    pub zones: Vec<SampleZone>,
    /// Zone groups (new grouped list with round-robin support).
    #[serde(default)]
    pub zone_groups: Vec<ZoneGroup>,
    /// ADSR envelope: attack, decay, sustain, release (seconds/level).
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    /// Maximum polyphony (simultaneous voices).
    pub max_voices: usize,
}

/// Raw deserialization target for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SamplerInstrumentRaw {
    pub name: String,
    pub origin: String,
    pub description: String,
    pub author: String,
    pub tuning: TuningTable,
    #[serde(default)]
    pub zones: Vec<SampleZone>,
    #[serde(default)]
    pub zone_groups: Vec<ZoneGroup>,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub max_voices: usize,
}

impl From<SamplerInstrumentRaw> for SamplerInstrument {
    fn from(raw: SamplerInstrumentRaw) -> Self {
        let mut instrument = SamplerInstrument {
            name: raw.name,
            origin: raw.origin,
            description: raw.description,
            author: raw.author,
            tuning: raw.tuning,
            zones: raw.zones,
            zone_groups: raw.zone_groups,
            attack: raw.attack,
            decay: raw.decay,
            sustain: raw.sustain,
            release: raw.release,
            max_voices: raw.max_voices,
        };
        instrument.normalize();
        instrument
    }
}

impl SamplerInstrument {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            origin: String::new(),
            description: String::new(),
            author: String::new(),
            tuning: TuningTable::default(),
            zones: Vec::new(),
            zone_groups: Vec::new(),
            attack: 0.005,
            decay: 0.1,
            sustain: 0.8,
            release: 0.3,
            max_voices: 16,
        }
    }

    /// Normalize the instrument after deserialization.
    /// If zone_groups is empty, populate it from zones (each zone → ZoneGroup with Sequential mode).
    pub fn normalize(&mut self) {
        if self.zone_groups.is_empty() && !self.zones.is_empty() {
            self.zone_groups = self
                .zones
                .iter()
                .cloned()
                .map(|zone| ZoneGroup {
                    zones: vec![zone],
                    mode: RoundRobinMode::Sequential,
                })
                .collect();
        }
    }

    /// Find the best matching zone for a note + velocity.
    /// If multiple zones match, prefer the one whose root_note is closest.
    pub fn find_zone(&self, note: u8, velocity: u8) -> Option<&SampleZone> {
        let mut best: Option<&SampleZone> = None;
        let mut best_dist = u8::MAX;
        for zone in &self.zones {
            if zone.matches(note, velocity) {
                let dist = note.abs_diff(zone.root_note);
                if dist < best_dist {
                    best_dist = dist;
                    best = Some(zone);
                }
            }
        }
        best
    }

    /// Find a zone using round-robin selection.
    /// Returns the selected zone from the first matching zone group.
    pub fn find_zone_rr<'a>(
        &'a self,
        note: u8,
        velocity: u8,
        rr_state: &mut RoundRobinState,
    ) -> Option<&'a SampleZone> {
        for (group_idx, group) in self.zone_groups.iter().enumerate() {
            // Check if any zone in this group matches the note/velocity
            if group.zones.iter().any(|z| z.matches(note, velocity)) {
                return rr_state.select_zone(group_idx, &group.zones, &group.mode);
            }
        }
        None
    }

    /// Add a zone.
    pub fn add_zone(&mut self, zone: SampleZone) {
        self.zones.push(zone);
    }

    /// Save to JSON file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(path, json)
    }

    /// Load from JSON file.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

/// Loaded instrument — zones with their audio buffers in memory.
pub struct LoadedInstrument {
    pub instrument: SamplerInstrument,
    /// Maps zone id → loaded buffer.
    pub buffers: HashMap<String, SampleBuffer>,
    /// Maps zone id → release buffer (if any).
    pub release_buffers: HashMap<String, SampleBuffer>,
}

impl LoadedInstrument {
    /// Load all audio files for an instrument.
    pub fn load(instrument: SamplerInstrument, base_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffers = HashMap::new();
        let mut release_buffers = HashMap::new();

        for zone in &instrument.zones {
            let path = base_dir.join(&zone.file_path);
            let buf = SampleBuffer::load_wav(&path)?;
            buffers.insert(zone.id.clone(), buf);

            if let Some(ref rel_path) = zone.release_file {
                let rpath = base_dir.join(rel_path);
                if rpath.exists() {
                    let rbuf = SampleBuffer::load_wav(&rpath)?;
                    release_buffers.insert(zone.id.clone(), rbuf);
                }
            }
        }

        Ok(Self { instrument, buffers, release_buffers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_round_robin_mode_derives() {
        let mode = RoundRobinMode::Sequential;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_zone_group_creation() {
        let zone = SampleZone {
            id: "test".into(),
            file_path: "test.wav".into(),
            root_note: 60,
            note_low: 60,
            note_high: 60,
            velocity_low: 0,
            velocity_high: 127,
            articulation: ArticulationType::OneShot,
            volume_db: 0.0,
            tune_cents: 0.0,
            release_file: None,
        };

        let group = ZoneGroup {
            zones: vec![zone],
            mode: RoundRobinMode::Sequential,
        };

        assert_eq!(group.zones.len(), 1);
        assert_eq!(group.mode, RoundRobinMode::Sequential);
    }

    #[test]
    fn test_round_robin_state_sequential() {
        let mut state = RoundRobinState::with_seed(12345);
        
        let zones = vec![
            SampleZone {
                id: "zone1".into(),
                file_path: "test1.wav".into(),
                root_note: 60,
                note_low: 60,
                note_high: 60,
                velocity_low: 0,
                velocity_high: 127,
                articulation: ArticulationType::OneShot,
                volume_db: 0.0,
                tune_cents: 0.0,
                release_file: None,
            },
            SampleZone {
                id: "zone2".into(),
                file_path: "test2.wav".into(),
                root_note: 60,
                note_low: 60,
                note_high: 60,
                velocity_low: 0,
                velocity_high: 127,
                articulation: ArticulationType::OneShot,
                volume_db: 0.0,
                tune_cents: 0.0,
                release_file: None,
            },
        ];

        // Sequential mode should cycle through zones
        let z1 = state.select_zone(0, &zones, &RoundRobinMode::Sequential);
        assert_eq!(z1.unwrap().id, "zone1");
        
        let z2 = state.select_zone(0, &zones, &RoundRobinMode::Sequential);
        assert_eq!(z2.unwrap().id, "zone2");
        
        let z3 = state.select_zone(0, &zones, &RoundRobinMode::Sequential);
        assert_eq!(z3.unwrap().id, "zone1");
    }

    #[test]
    fn test_sampler_instrument_normalize() {
        let mut instrument = SamplerInstrument::new("test");
        
        let zone = SampleZone {
            id: "zone1".into(),
            file_path: "test.wav".into(),
            root_note: 60,
            note_low: 60,
            note_high: 60,
            velocity_low: 0,
            velocity_high: 127,
            articulation: ArticulationType::OneShot,
            volume_db: 0.0,
            tune_cents: 0.0,
            release_file: None,
        };
        
        instrument.zones.push(zone);
        instrument.normalize();
        
        assert_eq!(instrument.zone_groups.len(), 1);
        assert_eq!(instrument.zone_groups[0].zones.len(), 1);
        assert_eq!(instrument.zone_groups[0].mode, RoundRobinMode::Sequential);
    }

    #[test]
    fn test_find_zone_rr() {
        let mut instrument = SamplerInstrument::new("test");
        
        let zone1 = SampleZone {
            id: "zone1".into(),
            file_path: "test1.wav".into(),
            root_note: 60,
            note_low: 60,
            note_high: 60,
            velocity_low: 0,
            velocity_high: 127,
            articulation: ArticulationType::OneShot,
            volume_db: 0.0,
            tune_cents: 0.0,
            release_file: None,
        };
        
        let zone2 = SampleZone {
            id: "zone2".into(),
            file_path: "test2.wav".into(),
            root_note: 60,
            note_low: 60,
            note_high: 60,
            velocity_low: 0,
            velocity_high: 127,
            articulation: ArticulationType::OneShot,
            volume_db: 0.0,
            tune_cents: 0.0,
            release_file: None,
        };
        
        instrument.zone_groups.push(ZoneGroup {
            zones: vec![zone1, zone2],
            mode: RoundRobinMode::Sequential,
        });
        
        let mut rr_state = RoundRobinState::with_seed(12345);
        
        let z1 = instrument.find_zone_rr(60, 100, &mut rr_state);
        assert!(z1.is_some());
        assert_eq!(z1.unwrap().id, "zone1");
        
        let z2 = instrument.find_zone_rr(60, 100, &mut rr_state);
        assert!(z2.is_some());
        assert_eq!(z2.unwrap().id, "zone2");
    }

    // Property 13
    proptest! {
        /// **Validates: Requirements 6.4, 6.11**
        ///
        /// Feature: aether-engine-upgrades, Property 13: Sequential round-robin full-cycle
        ///
        /// Property 13: Sequential round-robin full-cycle.
        ///
        /// For any `ZoneGroup` with `RoundRobinMode::Sequential` and N zones, after exactly N
        /// consecutive note-on events on that group, each zone SHALL have been selected exactly
        /// once (full permutation cycle).
        #[test]
        fn prop_sequential_round_robin_full_cycle(
            n in 1usize..=16,
        ) {
            // Create a ZoneGroup with N zones in Sequential mode
            let mut zones = Vec::new();
            for i in 0..n {
                zones.push(SampleZone {
                    id: format!("zone_{}", i),
                    file_path: format!("sample_{}.wav", i),
                    root_note: 60,
                    note_low: 60,
                    note_high: 60,
                    velocity_low: 0,
                    velocity_high: 127,
                    articulation: ArticulationType::OneShot,
                    volume_db: 0.0,
                    tune_cents: 0.0,
                    release_file: None,
                });
            }

            let group = ZoneGroup {
                zones,
                mode: RoundRobinMode::Sequential,
            };

            // Create a round-robin state
            let mut rr_state = RoundRobinState::with_seed(12345);

            // Fire N note-ons and collect the selected zone indices
            let mut selected_indices = Vec::new();
            for _ in 0..n {
                let idx = rr_state.select(0, n, &group.mode);
                selected_indices.push(idx);
            }

            // Assert each zone index appears exactly once
            let mut sorted_indices = selected_indices.clone();
            sorted_indices.sort_unstable();
            
            // Check that we have exactly N selections
            prop_assert_eq!(selected_indices.len(), n);
            
            // Check that the sorted indices form the sequence [0, 1, 2, ..., n-1]
            let expected: Vec<usize> = (0..n).collect();
            prop_assert_eq!(sorted_indices, expected);
            
            // Additionally verify that each index appears exactly once (no duplicates)
            for i in 0..n {
                let count = selected_indices.iter().filter(|&&x| x == i).count();
                prop_assert_eq!(count, 1, "Zone index {} should appear exactly once, but appeared {} times", i, count);
            }
        }
    }

    // Property 14
    proptest! {
        /// **Validates: Requirements 6.6, 6.12**
        ///
        /// Feature: aether-engine-upgrades, Property 14: RandomNoRepeat never repeats consecutively
        ///
        /// Property 14: RandomNoRepeat never repeats consecutively.
        ///
        /// For any `ZoneGroup` with `RoundRobinMode::RandomNoRepeat` and N ≥ 2 zones, for any
        /// sequence of M consecutive note-on events on that group, no two adjacent events in the
        /// sequence SHALL select the same zone.
        #[test]
        fn prop_random_no_repeat_no_consecutive_repeats(
            n in 2usize..=16,
            seed in any::<u64>(),
        ) {
            // Create a round-robin state with the random seed
            let mut rr_state = RoundRobinState::with_seed(seed);

            // Fire 100 note-ons and collect the selected zone indices
            let mut selected_indices = Vec::new();
            for _ in 0..100 {
                let idx = rr_state.select(0, n, &RoundRobinMode::RandomNoRepeat);
                selected_indices.push(idx);
            }

            // Assert no two adjacent selections are the same
            for i in 0..selected_indices.len() - 1 {
                let current = selected_indices[i];
                let next = selected_indices[i + 1];
                prop_assert_ne!(
                    current,
                    next,
                    "RandomNoRepeat violated: zone {} was selected twice in a row at positions {} and {}",
                    current,
                    i,
                    i + 1
                );
            }

            // Additional sanity check: all indices should be in valid range [0, n)
            for (i, &idx) in selected_indices.iter().enumerate() {
                prop_assert!(
                    idx < n,
                    "Invalid zone index {} at position {} (should be < {})",
                    idx,
                    i,
                    n
                );
            }
        }
    }

    // Property 15
    proptest! {
        /// **Validates: Requirements 6.7, 6.8**
        ///
        /// Feature: aether-engine-upgrades, Property 15: Backward-compatible instrument loading
        ///
        /// Property 15: Backward-compatible instrument loading.
        ///
        /// For any valid legacy instrument JSON (flat `zones` array, no `zone_groups` field),
        /// deserializing it SHALL succeed and each zone SHALL be treated as a `ZoneGroup` of
        /// size one with `RoundRobinMode::Sequential`, producing identical zone-selection
        /// behavior to the pre-upgrade `find_zone` method.
        #[test]
        fn prop_backward_compatible_instrument_loading(
            zone_count in 1usize..=10,
            note_ranges in prop::collection::vec((0u8..=127u8, 0u8..=127u8), 1..=10),
            velocity_ranges in prop::collection::vec((0u8..=127u8, 0u8..=127u8), 1..=10),
        ) {
            // Generate random legacy instrument JSON (flat zones array, no zone_groups field)
            let mut zones = Vec::new();
            for i in 0..zone_count {
                let (note_low, note_high_offset) = note_ranges[i % note_ranges.len()];
                let note_high = note_low.saturating_add(note_high_offset % 12);
                let root_note = note_low + (note_high - note_low) / 2;
                
                let (vel_low, vel_high_offset) = velocity_ranges[i % velocity_ranges.len()];
                let vel_high = vel_low.saturating_add(vel_high_offset);
                
                zones.push(SampleZone {
                    id: format!("zone_{}", i),
                    file_path: format!("sample_{}.wav", i),
                    root_note,
                    note_low,
                    note_high,
                    velocity_low: vel_low,
                    velocity_high: vel_high,
                    articulation: ArticulationType::OneShot,
                    volume_db: 0.0,
                    tune_cents: 0.0,
                    release_file: None,
                });
            }

            // Create legacy instrument JSON (no zone_groups field)
            let legacy_json = serde_json::json!({
                "name": "Legacy Instrument",
                "origin": "Test",
                "description": "Test legacy instrument",
                "author": "Test",
                "tuning": {
                    "name": "12-TET",
                    "reference_note": 69,
                    "reference_freq": 440.0,
                    "cents_map": []
                },
                "zones": zones,
                "attack": 0.005,
                "decay": 0.1,
                "sustain": 0.8,
                "release": 0.3,
                "max_voices": 16
            });

            // Deserialize
            let instrument: SamplerInstrument = serde_json::from_value(legacy_json)
                .expect("Failed to deserialize legacy instrument");

            // Assert: each zone is wrapped in a ZoneGroup of size 1 with Sequential mode
            prop_assert_eq!(instrument.zone_groups.len(), zones.len());
            for (i, group) in instrument.zone_groups.iter().enumerate() {
                prop_assert_eq!(group.zones.len(), 1);
                prop_assert_eq!(group.mode, RoundRobinMode::Sequential);
                prop_assert_eq!(group.zones[0].id, format!("zone_{}", i));
            }

            // Assert: find_zone_rr returns the same zone as the legacy find_zone for any note/velocity
            let mut rr_state = RoundRobinState::with_seed(12345);
            for note in 0u8..=127 {
                for velocity in [1u8, 64, 127] {
                    let legacy_zone = instrument.find_zone(note, velocity);
                    let rr_zone = instrument.find_zone_rr(note, velocity, &mut rr_state);
                    
                    // Both should return the same result (Some or None)
                    prop_assert_eq!(legacy_zone.is_some(), rr_zone.is_some());
                    
                    // If both return Some, they should be the same zone
                    if let (Some(legacy), Some(rr)) = (legacy_zone, rr_zone) {
                        prop_assert_eq!(legacy.id, rr.id);
                    }
                }
            }
        }
    }
}
