//! Noise Gate — attenuates signal below threshold.
//!
//! Param layout:
//!   0 = threshold  (dB, -80 – 0)
//!   1 = ratio      (1:1 – inf:1, represented as 1.0 – 100.0)
//!   2 = attack     (ms, 0.1 – 100)
//!   3 = release    (ms, 10 – 2000)
//!   4 = hold       (ms, 0 – 500)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Gate {
    /// RMS envelope follower state.
    rms_env: f32,
    /// Gain envelope (smoothed).
    gain_env: f32,
    /// Hold counter (samples remaining in hold state).
    hold_counter: usize,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            rms_env: 0.0,
            gain_env: 0.0,
            hold_counter: 0,
        }
    }

    #[inline(always)]
    fn db_to_linear(db: f32) -> f32 {
        10.0f32.powf(db / 20.0)
    }

    #[inline(always)]
    fn linear_to_db(linear: f32) -> f32 {
        if linear <= 1e-10 {
            return -200.0;
        }
        20.0 * linear.log10()
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for Gate {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        let threshold_db = params.get(0).current.clamp(-80.0, 0.0);
        let ratio = params.get(1).current.clamp(1.0, 100.0);
        let attack_ms = params.get(2).current.clamp(0.1, 100.0);
        let release_ms = params.get(3).current.clamp(10.0, 2000.0);
        let hold_ms = params.get(4).current.clamp(0.0, 500.0);

        let threshold_linear = Self::db_to_linear(threshold_db);
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
        let hold_samples = (hold_ms * 0.001 * sample_rate) as usize;

        for i in 0..BUFFER_SIZE {
            let x = input[i];

            // RMS envelope follower
            let x2 = x * x;
            self.rms_env = if x2 > self.rms_env {
                attack_coeff * self.rms_env + (1.0 - attack_coeff) * x2
            } else {
                release_coeff * self.rms_env + (1.0 - release_coeff) * x2
            };
            let rms_linear = self.rms_env.sqrt();

            // Gate logic with hold
            let target_gain = if rms_linear > threshold_linear {
                // Signal above threshold: gate open
                self.hold_counter = hold_samples;
                1.0
            } else if self.hold_counter > 0 {
                // In hold state: keep gate open
                self.hold_counter -= 1;
                1.0
            } else {
                // Signal below threshold: apply attenuation
                let rms_db = Self::linear_to_db(rms_linear);
                let attenuation_db = (rms_db - threshold_db) * (1.0 / ratio - 1.0);
                Self::db_to_linear(attenuation_db).max(0.0)
            };

            // Smooth gain envelope
            self.gain_env = if target_gain > self.gain_env {
                attack_coeff * self.gain_env + (1.0 - attack_coeff) * target_gain
            } else {
                release_coeff * self.gain_env + (1.0 - release_coeff) * target_gain
            };

            output[i] = x * self.gain_env;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "Gate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_silence_attenuation() {
        let mut gate = Gate::new();
        let mut params = ParamBlock::new();
        // threshold=-40dB, ratio=10, attack=1ms, release=100ms, hold=0ms
        for &v in &[-40.0f32, 10.0, 1.0, 100.0, 0.0] {
            params.add(v);
        }
        let input = [0.0f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        gate.process(&inputs, &mut output, &mut params, 48000.0);
        // Silence should be attenuated (close to zero)
        for s in &output {
            assert!(s.abs() < 1e-6, "silence should be attenuated");
        }
    }

    #[test]
    fn test_gate_passes_loud_signal() {
        let mut gate = Gate::new();
        let mut params = ParamBlock::new();
        // threshold=-40dB, ratio=10, attack=1ms, release=100ms, hold=0ms
        for &v in &[-40.0f32, 10.0, 1.0, 100.0, 0.0] {
            params.add(v);
        }
        let input = [0.5f32; BUFFER_SIZE]; // Loud signal (~-6dB)
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        gate.process(&inputs, &mut output, &mut params, 48000.0);
        // After settling, loud signal should pass through
        let last = output[BUFFER_SIZE - 1].abs();
        assert!(
            last > 0.3,
            "loud signal should pass through gate, got {last}"
        );
    }

    #[test]
    fn test_gate_attenuates_quiet_signal() {
        let mut gate = Gate::new();
        let mut params = ParamBlock::new();
        // threshold=-40dB, ratio=10, attack=1ms, release=100ms, hold=0ms
        for &v in &[-40.0f32, 10.0, 1.0, 100.0, 0.0] {
            params.add(v);
        }
        let input = [0.0001f32; BUFFER_SIZE]; // Very quiet signal (~-80dB)
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        gate.process(&inputs, &mut output, &mut params, 48000.0);
        // After settling, quiet signal should be attenuated (less than input)
        let last = output[BUFFER_SIZE - 1].abs();
        assert!(last < 0.01, "quiet signal should be attenuated, got {last}");
    }
}
