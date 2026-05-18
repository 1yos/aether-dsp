//! MIDI Learn - Map MIDI CC controllers to parameters.
//!
//! Allows users to easily map physical MIDI controllers (knobs, faders, etc.)
//! to software parameters by entering "learn mode" and moving the controller.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A MIDI CC mapping to a parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMapping {
    /// MIDI channel (1-16).
    pub channel: u8,
    
    /// MIDI CC number (0-127).
    pub cc: u8,
    
    /// Parameter identifier (e.g., "filter_cutoff", "gain").
    pub param_id: String,
    
    /// Minimum parameter value.
    pub min_value: f32,
    
    /// Maximum parameter value.
    pub max_value: f32,
    
    /// Optional curve (linear, exponential, logarithmic).
    pub curve: MappingCurve,
}

/// Mapping curve type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingCurve {
    /// Linear mapping (default).
    Linear,
    /// Exponential curve (good for frequency).
    Exponential,
    /// Logarithmic curve (good for gain).
    Logarithmic,
}

impl Default for MappingCurve {
    fn default() -> Self {
        Self::Linear
    }
}

/// MIDI Learn engine.
pub struct MidiLearn {
    /// All active mappings: (channel, cc) -> MidiMapping
    mappings: HashMap<(u8, u8), MidiMapping>,
    
    /// Learn mode state.
    learn_mode: Option<LearnState>,
}

/// Learn mode state.
#[derive(Debug, Clone)]
struct LearnState {
    /// Parameter ID waiting to be mapped.
    param_id: String,
    
    /// Min/max values for the parameter.
    min_value: f32,
    max_value: f32,
    
    /// Curve type.
    curve: MappingCurve,
}

impl MidiLearn {
    /// Creates a new MIDI Learn engine.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            learn_mode: None,
        }
    }

    /// Enters learn mode for a parameter.
    ///
    /// The next MIDI CC message received will be mapped to this parameter.
    ///
    /// # Arguments
    ///
    /// * `param_id` - Parameter identifier
    /// * `min_value` - Minimum parameter value
    /// * `max_value` - Maximum parameter value
    /// * `curve` - Mapping curve type
    ///
    /// # Example
    ///
    /// ```
    /// use aether_midi::{MidiLearn, MappingCurve};
    ///
    /// let mut learn = MidiLearn::new();
    ///
    /// // Enter learn mode for filter cutoff
    /// learn.start_learn("filter_cutoff", 20.0, 20000.0, MappingCurve::Exponential);
    ///
    /// // User moves a MIDI controller...
    /// // The next CC message will be mapped to filter_cutoff
    /// ```
    pub fn start_learn(
        &mut self,
        param_id: impl Into<String>,
        min_value: f32,
        max_value: f32,
        curve: MappingCurve,
    ) {
        self.learn_mode = Some(LearnState {
            param_id: param_id.into(),
            min_value,
            max_value,
            curve,
        });
    }

    /// Exits learn mode without creating a mapping.
    pub fn cancel_learn(&mut self) {
        self.learn_mode = None;
    }

    /// Checks if currently in learn mode.
    pub fn is_learning(&self) -> bool {
        self.learn_mode.is_some()
    }

    /// Processes a MIDI CC message.
    ///
    /// If in learn mode, creates a mapping. Otherwise, applies existing mappings.
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `cc` - MIDI CC number (0-127)
    /// * `value` - MIDI CC value (0-127)
    ///
    /// # Returns
    ///
    /// If a mapping exists (or was just created), returns `Some((param_id, mapped_value))`.
    /// Otherwise returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_midi::{MidiLearn, MappingCurve};
    ///
    /// let mut learn = MidiLearn::new();
    ///
    /// // Enter learn mode
    /// learn.start_learn("gain", 0.0, 1.0, MappingCurve::Linear);
    ///
    /// // User moves CC 7 on channel 1
    /// let result = learn.process_cc(1, 7, 64);
    /// assert!(result.is_some());
    /// let (param_id, value) = result.unwrap();
    /// assert_eq!(param_id, "gain");
    /// assert!((value - 0.5).abs() < 0.01); // 64/127 ≈ 0.5
    ///
    /// // Now CC 7 is mapped to gain
    /// let result = learn.process_cc(1, 7, 127);
    /// assert!(result.is_some());
    /// let (param_id, value) = result.unwrap();
    /// assert_eq!(param_id, "gain");
    /// assert!((value - 1.0).abs() < 0.01);
    /// ```
    pub fn process_cc(&mut self, channel: u8, cc: u8, value: u8) -> Option<(String, f32)> {
        // If in learn mode, create mapping
        if let Some(learn_state) = self.learn_mode.take() {
            let mapping = MidiMapping {
                channel,
                cc,
                param_id: learn_state.param_id.clone(),
                min_value: learn_state.min_value,
                max_value: learn_state.max_value,
                curve: learn_state.curve,
            };

            self.mappings.insert((channel, cc), mapping.clone());
            
            // Apply the mapping immediately
            let mapped_value = Self::map_value(value, &mapping);
            return Some((mapping.param_id, mapped_value));
        }

        // Otherwise, apply existing mapping
        if let Some(mapping) = self.mappings.get(&(channel, cc)) {
            let mapped_value = Self::map_value(value, mapping);
            Some((mapping.param_id.clone(), mapped_value))
        } else {
            None
        }
    }

    /// Adds a mapping manually (without learn mode).
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `cc` - MIDI CC number (0-127)
    /// * `param_id` - Parameter identifier
    /// * `min_value` - Minimum parameter value
    /// * `max_value` - Maximum parameter value
    /// * `curve` - Mapping curve type
    pub fn add_mapping(
        &mut self,
        channel: u8,
        cc: u8,
        param_id: impl Into<String>,
        min_value: f32,
        max_value: f32,
        curve: MappingCurve,
    ) {
        let mapping = MidiMapping {
            channel,
            cc,
            param_id: param_id.into(),
            min_value,
            max_value,
            curve,
        };
        self.mappings.insert((channel, cc), mapping);
    }

    /// Removes a mapping.
    ///
    /// # Arguments
    ///
    /// * `channel` - MIDI channel (1-16)
    /// * `cc` - MIDI CC number (0-127)
    ///
    /// # Returns
    ///
    /// The removed mapping, or None if no mapping existed.
    pub fn remove_mapping(&mut self, channel: u8, cc: u8) -> Option<MidiMapping> {
        self.mappings.remove(&(channel, cc))
    }

    /// Gets a mapping.
    pub fn get_mapping(&self, channel: u8, cc: u8) -> Option<&MidiMapping> {
        self.mappings.get(&(channel, cc))
    }

    /// Gets all mappings.
    pub fn mappings(&self) -> impl Iterator<Item = &MidiMapping> {
        self.mappings.values()
    }

    /// Clears all mappings.
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
    }

    /// Saves mappings to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mappings: Vec<_> = self.mappings.values().collect();
        serde_json::to_string_pretty(&mappings)
    }

    /// Loads mappings from JSON.
    pub fn from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let mappings: Vec<MidiMapping> = serde_json::from_str(json)?;
        self.mappings.clear();
        for mapping in mappings {
            self.mappings.insert((mapping.channel, mapping.cc), mapping);
        }
        Ok(())
    }

    /// Maps a MIDI CC value (0-127) to a parameter value using the mapping.
    fn map_value(cc_value: u8, mapping: &MidiMapping) -> f32 {
        let normalized = cc_value as f32 / 127.0; // 0.0 to 1.0

        let curved = match mapping.curve {
            MappingCurve::Linear => normalized,
            MappingCurve::Exponential => {
                // Exponential curve: y = x^2
                normalized * normalized
            }
            MappingCurve::Logarithmic => {
                // Logarithmic curve: y = log(1 + 9x) / log(10)
                // Maps 0->0, 1->1, with logarithmic shape
                if normalized <= 0.0 {
                    0.0
                } else {
                    (1.0 + 9.0 * normalized).log10()
                }
            }
        };

        // Scale to parameter range
        mapping.min_value + curved * (mapping.max_value - mapping.min_value)
    }
}

impl Default for MidiLearn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_learn_basic() {
        let mut learn = MidiLearn::new();
        
        // Enter learn mode
        learn.start_learn("gain", 0.0, 1.0, MappingCurve::Linear);
        assert!(learn.is_learning());

        // Process CC message - should create mapping
        let result = learn.process_cc(1, 7, 64);
        assert!(result.is_some());
        let (param_id, value) = result.unwrap();
        assert_eq!(param_id, "gain");
        assert!((value - 0.5).abs() < 0.01);

        // Should exit learn mode
        assert!(!learn.is_learning());

        // Mapping should now exist
        assert!(learn.get_mapping(1, 7).is_some());
    }

    #[test]
    fn test_midi_learn_cancel() {
        let mut learn = MidiLearn::new();
        
        learn.start_learn("gain", 0.0, 1.0, MappingCurve::Linear);
        assert!(learn.is_learning());

        learn.cancel_learn();
        assert!(!learn.is_learning());

        // No mapping should be created
        let result = learn.process_cc(1, 7, 64);
        assert!(result.is_none());
    }

    #[test]
    fn test_midi_learn_apply_mapping() {
        let mut learn = MidiLearn::new();
        
        // Create mapping manually
        learn.add_mapping(1, 7, "gain", 0.0, 1.0, MappingCurve::Linear);

        // Apply mapping
        let result = learn.process_cc(1, 7, 0);
        assert_eq!(result, Some(("gain".to_string(), 0.0)));

        let result = learn.process_cc(1, 7, 127);
        assert_eq!(result, Some(("gain".to_string(), 1.0)));

        let result = learn.process_cc(1, 7, 64);
        let (_, value) = result.unwrap();
        assert!((value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_midi_learn_exponential_curve() {
        let mut learn = MidiLearn::new();
        
        learn.add_mapping(1, 7, "cutoff", 20.0, 20000.0, MappingCurve::Exponential);

        // Exponential curve: more resolution at low end
        let result = learn.process_cc(1, 7, 64);
        let (_, value) = result.unwrap();
        // 64/127 ≈ 0.5, squared ≈ 0.25
        // 20 + 0.25 * 19980 ≈ 5015
        assert!(value > 4000.0 && value < 6000.0);
    }

    #[test]
    fn test_midi_learn_remove_mapping() {
        let mut learn = MidiLearn::new();
        
        learn.add_mapping(1, 7, "gain", 0.0, 1.0, MappingCurve::Linear);
        assert!(learn.get_mapping(1, 7).is_some());

        let removed = learn.remove_mapping(1, 7);
        assert!(removed.is_some());
        assert!(learn.get_mapping(1, 7).is_none());
    }

    #[test]
    fn test_midi_learn_json_serialization() {
        let mut learn = MidiLearn::new();
        
        learn.add_mapping(1, 7, "gain", 0.0, 1.0, MappingCurve::Linear);
        learn.add_mapping(1, 74, "cutoff", 20.0, 20000.0, MappingCurve::Exponential);

        // Save to JSON
        let json = learn.to_json().unwrap();
        assert!(json.contains("gain"));
        assert!(json.contains("cutoff"));

        // Load from JSON
        let mut learn2 = MidiLearn::new();
        learn2.from_json(&json).unwrap();

        assert!(learn2.get_mapping(1, 7).is_some());
        assert!(learn2.get_mapping(1, 74).is_some());
    }
}
