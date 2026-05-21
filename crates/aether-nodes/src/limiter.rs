//! Limiter — brick-wall limiter with lookahead.
//!
//! Param layout:
//!   0 = threshold  (dB, -24 – 0)
//!   1 = release    (ms, 10 – 1000)
//!   2 = ceiling    (dB, -12 – 0)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

const LOOKAHEAD_MS: f32 = 5.0; // 5ms lookahead
const MAX_LOOKAHEAD_SAMPLES: usize = 512; // At 96kHz: 5ms = 480 samples

pub struct Limiter {
    /// Lookahead buffer (ring buffer).
    lookahead_buffer: [f32; MAX_LOOKAHEAD_SAMPLES],
    /// Write position in lookahead buffer.
    write_pos: usize,
    /// Gain reduction envelope.
    gain_env: f32,
    /// Lookahead size in samples.
    lookahead_samples: usize,
}

impl Limiter {
    pub fn new() -> Self {
        Self {
            lookahead_buffer: [0.0; MAX_LOOKAHEAD_SAMPLES],
            write_pos: 0,
            gain_env: 1.0,
            lookahead_samples: 240, // 5ms @ 48kHz
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

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for Limiter {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        let threshold_db = params.get(0).current.clamp(-24.0, 0.0);
        let release_ms = params.get(1).current.clamp(10.0, 1000.0);
        let ceiling_db = params.get(2).current.clamp(-12.0, 0.0);

        let threshold_linear = Self::db_to_linear(threshold_db);
        let ceiling_linear = Self::db_to_linear(ceiling_db);
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

        // Update lookahead size based on sample rate
        self.lookahead_samples =
            ((LOOKAHEAD_MS * 0.001 * sample_rate) as usize).min(MAX_LOOKAHEAD_SAMPLES);

        for i in 0..BUFFER_SIZE {
            let x = input[i];

            // Write to lookahead buffer
            self.lookahead_buffer[self.write_pos] = x;
            self.write_pos = (self.write_pos + 1) % self.lookahead_samples;

            // Read from lookahead buffer (delayed signal)
            let read_pos = self.write_pos; // Current position is oldest sample
            let delayed = self.lookahead_buffer[read_pos];

            // Peak detection on current input (lookahead)
            let peak = x.abs();

            // Calculate required gain reduction
            let target_gain = if peak > threshold_linear {
                // Calculate gain to bring peak down to threshold
                let reduction = threshold_linear / peak;
                reduction.min(1.0)
            } else {
                1.0 // No reduction needed
            };

            // Smooth gain envelope (instant attack, slow release)
            self.gain_env = if target_gain < self.gain_env {
                target_gain // Instant attack
            } else {
                release_coeff * self.gain_env + (1.0 - release_coeff) * target_gain
            };

            // Apply gain reduction and ceiling
            let limited = delayed * self.gain_env;
            output[i] = limited.clamp(-ceiling_linear, ceiling_linear);

            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "Limiter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_silence_passthrough() {
        let mut limiter = Limiter::new();
        let mut params = ParamBlock::new();
        // threshold=0dB, release=100ms, ceiling=0dB
        for &v in &[-0.0f32, 100.0, 0.0] {
            params.add(v);
        }
        let input = [0.0f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        limiter.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output {
            assert!(s.abs() < 1e-6, "silence should pass through");
        }
    }

    #[test]
    fn test_limiter_reduces_peaks() {
        let mut limiter = Limiter::new();
        let mut params = ParamBlock::new();
        // threshold=-6dB, release=100ms, ceiling=0dB
        for &v in &[-6.0f32, 100.0, 0.0] {
            params.add(v);
        }

        // Create signal with peaks above threshold
        let mut input = [0.0f32; BUFFER_SIZE];
        for i in 0..BUFFER_SIZE {
            input[i] = if i % 64 < 32 { 0.8 } else { 0.2 }; // Peaks at 0.8 (~-2dB)
        }

        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        limiter.process(&inputs, &mut output, &mut params, 48000.0);

        // After settling, peaks should be reduced
        let max_output = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        assert!(
            max_output < 0.8,
            "limiter should reduce peaks, got {max_output}"
        );
    }

    #[test]
    fn test_limiter_respects_ceiling() {
        let mut limiter = Limiter::new();
        let mut params = ParamBlock::new();
        // threshold=-12dB, release=50ms, ceiling=-3dB
        for &v in &[-12.0f32, 50.0, -3.0] {
            params.add(v);
        }

        let input = [1.0f32; BUFFER_SIZE]; // Very loud signal
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        limiter.process(&inputs, &mut output, &mut params, 48000.0);

        let ceiling_linear = 10.0f32.powf(-3.0 / 20.0); // -3dB in linear
        let max_output = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        assert!(
            max_output <= ceiling_linear + 0.01,
            "output should not exceed ceiling"
        );
    }
}
