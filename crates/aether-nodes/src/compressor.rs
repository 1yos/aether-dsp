//! RMS Compressor — dynamics processor.
//!
//! Param layout:
//!   0 = threshold  (dB, -60 – 0)
//!   1 = ratio      (1:1 – 20:1)
//!   2 = attack     (ms, 0.1 – 200)
//!   3 = release    (ms, 10 – 2000)
//!   4 = makeup     (dB, 0 – 24)
//!   5 = knee       (dB, 0 – 12, soft-knee width)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Compressor {
    /// RMS envelope follower state.
    rms_env: f32,
    /// Gain reduction envelope (smoothed).
    gain_env: f32,
}

impl Compressor {
    pub fn new() -> Self {
        Self {
            rms_env: 0.0,
            gain_env: 1.0,
        }
    }

    #[inline(always)]
    fn db_to_linear(db: f32) -> f32 {
        10.0f32.powf(db / 20.0)
    }

    #[inline(always)]
    fn linear_to_db(linear: f32) -> f32 {
        if linear <= 1e-10 { return -200.0; }
        20.0 * linear.log10()
    }
}

impl Default for Compressor {
    fn default() -> Self { Self::new() }
}

impl DspNode for Compressor {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        let threshold_db = params.get(0).current.clamp(-60.0, 0.0);
        let ratio        = params.get(1).current.clamp(1.0, 20.0);
        let attack_ms    = params.get(2).current.clamp(0.1, 200.0);
        let release_ms   = params.get(3).current.clamp(10.0, 2000.0);
        let makeup_db    = params.get(4).current.clamp(0.0, 24.0);
        let knee_db      = params.get(5).current.clamp(0.0, 12.0);

        let attack_coeff  = (-1.0 / (attack_ms  * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
        let makeup_linear = Self::db_to_linear(makeup_db);

        for (i, out) in output.iter_mut().enumerate() {
            let x = input[i];

            // RMS envelope follower (squared signal, smoothed)
            let x2 = x * x;
            self.rms_env = if x2 > self.rms_env {
                attack_coeff  * self.rms_env + (1.0 - attack_coeff)  * x2
            } else {
                release_coeff * self.rms_env + (1.0 - release_coeff) * x2
            };
            let rms_db = Self::linear_to_db(self.rms_env.sqrt());

            // Gain computer with soft knee
            let gain_reduction_db = if knee_db > 0.0 {
                let knee_start = threshold_db - knee_db * 0.5;
                let knee_end   = threshold_db + knee_db * 0.5;
                if rms_db <= knee_start {
                    0.0
                } else if rms_db >= knee_end {
                    (rms_db - threshold_db) * (1.0 / ratio - 1.0)
                } else {
                    // Soft knee interpolation
                    let t = (rms_db - knee_start) / knee_db;
                    t * t * 0.5 * (1.0 / ratio - 1.0) * knee_db
                }
            } else {
                if rms_db > threshold_db {
                    (rms_db - threshold_db) * (1.0 / ratio - 1.0)
                } else {
                    0.0
                }
            };

            let target_gain = Self::db_to_linear(gain_reduction_db);

            // Smooth gain envelope
            self.gain_env = if target_gain < self.gain_env {
                attack_coeff  * self.gain_env + (1.0 - attack_coeff)  * target_gain
            } else {
                release_coeff * self.gain_env + (1.0 - release_coeff) * target_gain
            };

            *out = x * self.gain_env * makeup_linear;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str { "Compressor" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_silence_passthrough() {
        let mut comp = Compressor::new();
        let mut params = ParamBlock::new();
        // threshold=0dB, ratio=1, attack=1ms, release=100ms, makeup=0dB, knee=0
        for &v in &[-20.0f32, 2.0, 1.0, 100.0, 0.0, 0.0] { params.add(v); }
        let input = [0.0f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        comp.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output { assert!(s.abs() < 1e-6, "silence should pass through as silence"); }
    }

    #[test]
    fn test_compressor_reduces_loud_signal() {
        let mut comp = Compressor::new();
        let mut params = ParamBlock::new();
        // threshold=-20dB, ratio=4, attack=1ms, release=100ms, makeup=0dB, knee=0
        for &v in &[-20.0f32, 4.0, 1.0, 100.0, 0.0, 0.0] { params.add(v); }
        let input = [0.5f32; BUFFER_SIZE]; // ~-6dB, above threshold
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        comp.process(&inputs, &mut output, &mut params, 48000.0);
        // After settling, output should be quieter than input
        let last = output[BUFFER_SIZE - 1].abs();
        assert!(last < 0.5, "compressor should reduce gain above threshold, got {last}");
    }
}
