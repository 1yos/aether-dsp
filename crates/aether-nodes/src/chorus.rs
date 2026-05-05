//! Chorus / Flanger — BBD-style modulated delay.
//!
//! Uses two modulated delay lines (L/R) with slightly different LFO phases
//! to create stereo width. Outputs to a mono mix (summed L+R * 0.5).
//!
//! Param layout:
//!   0 = rate      (Hz, 0.1 – 10.0)
//!   1 = depth     (0.0 – 1.0, modulation depth in ms: 0–20ms)
//!   2 = feedback  (0.0 – 0.95, feedback amount)
//!   3 = wet       (0.0 – 1.0, dry/wet mix)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use std::f32::consts::TAU;

/// Maximum delay line length in samples (at 48kHz: ~42ms).
const MAX_DELAY_SAMPLES: usize = 2048;

pub struct Chorus {
    /// Delay buffer for left channel.
    buf_l: Box<[f32; MAX_DELAY_SAMPLES]>,
    /// Delay buffer for right channel.
    buf_r: Box<[f32; MAX_DELAY_SAMPLES]>,
    /// Write position.
    write_pos: usize,
    /// LFO phase for left channel.
    phase_l: f32,
    /// LFO phase for right channel (offset by 90°).
    phase_r: f32,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            buf_l: Box::new([0.0f32; MAX_DELAY_SAMPLES]),
            buf_r: Box::new([0.0f32; MAX_DELAY_SAMPLES]),
            write_pos: 0,
            phase_l: 0.0,
            phase_r: 0.25, // 90° offset for stereo width
        }
    }

    /// Linear interpolation read from a circular buffer.
    #[inline(always)]
    fn read_interp(buf: &[f32; MAX_DELAY_SAMPLES], write_pos: usize, delay_samples: f32) -> f32 {
        let delay_int = delay_samples as usize;
        let frac = delay_samples - delay_int as f32;

        let pos0 = (write_pos + MAX_DELAY_SAMPLES - delay_int) % MAX_DELAY_SAMPLES;
        let pos1 = (write_pos + MAX_DELAY_SAMPLES - delay_int - 1) % MAX_DELAY_SAMPLES;

        buf[pos0] * (1.0 - frac) + buf[pos1] * frac
    }
}

impl Default for Chorus {
    fn default() -> Self { Self::new() }
}

impl DspNode for Chorus {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        let rate     = params.get(0).current.clamp(0.1, 10.0);
        let depth    = params.get(1).current.clamp(0.0, 1.0);
        let feedback = params.get(2).current.clamp(0.0, 0.95);
        let wet      = params.get(3).current.clamp(0.0, 1.0);

        let phase_inc = rate / sample_rate;

        // Base delay: 5ms center, depth modulates ±10ms
        let base_delay = 0.005 * sample_rate;
        let mod_depth  = depth * 0.010 * sample_rate;

        for (i, out) in output.iter_mut().enumerate() {
            let dry = input[i];

            // LFO modulation
            let lfo_l = (self.phase_l * TAU).sin();
            let lfo_r = (self.phase_r * TAU).sin();

            let delay_l = (base_delay + lfo_l * mod_depth).clamp(1.0, (MAX_DELAY_SAMPLES - 2) as f32);
            let delay_r = (base_delay + lfo_r * mod_depth).clamp(1.0, (MAX_DELAY_SAMPLES - 2) as f32);

            // Read from delay lines
            let delayed_l = Self::read_interp(&self.buf_l, self.write_pos, delay_l);
            let delayed_r = Self::read_interp(&self.buf_r, self.write_pos, delay_r);

            // Write to delay lines with feedback
            self.buf_l[self.write_pos] = dry + delayed_l * feedback;
            self.buf_r[self.write_pos] = dry + delayed_r * feedback;

            // Advance write position
            self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

            // Mix L+R and blend dry/wet
            let wet_signal = (delayed_l + delayed_r) * 0.5;
            *out = dry * (1.0 - wet) + wet_signal * wet;

            // Advance LFO phases
            self.phase_l = (self.phase_l + phase_inc).fract();
            self.phase_r = (self.phase_r + phase_inc).fract();

            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str { "Chorus" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chorus_dry_passthrough() {
        let mut chorus = Chorus::new();
        let mut params = ParamBlock::new();
        for &v in &[1.0f32, 0.5, 0.0, 0.0] { params.add(v); } // wet=0 → dry passthrough
        let input = [0.5f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        chorus.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output {
            assert!((s - 0.5).abs() < 1e-5, "wet=0 should pass dry signal unchanged");
        }
    }

    #[test]
    fn test_chorus_bounded_output() {
        let mut chorus = Chorus::new();
        let mut params = ParamBlock::new();
        for &v in &[2.0f32, 1.0, 0.9, 1.0] { params.add(v); }
        let input = [1.0f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        chorus.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output {
            assert!(s.abs() < 10.0, "chorus output should be bounded, got {s}");
        }
    }
}
