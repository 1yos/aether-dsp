//! Parametric EQ — 3-band equalizer with adjustable frequency, gain, and Q.
//!
//! Param layout:
//!   0 = low_freq    (Hz, 20 – 500)
//!   1 = low_gain    (dB, -24 – 24)
//!   2 = mid_freq    (Hz, 200 – 5000)
//!   3 = mid_gain    (dB, -24 – 24)
//!   4 = mid_q       (0.1 – 10.0)
//!   5 = high_freq   (Hz, 2000 – 20000)
//!   6 = high_gain   (dB, -24 – 24)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

/// Biquad filter coefficients.
#[derive(Debug, Clone, Copy)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

/// Biquad filter state.
#[derive(Debug, Clone, Copy)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    #[inline(always)]
    fn process(&mut self, x: f32, coeffs: &BiquadCoeffs) -> f32 {
        let y = coeffs.b0 * x + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
              - coeffs.a1 * self.y1 - coeffs.a2 * self.y2;
        
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        
        y
    }
}

impl BiquadCoeffs {
    /// Peaking EQ filter (bell curve).
    fn peaking(freq: f32, gain_db: f32, q: f32, sample_rate: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Low shelf filter.
    fn low_shelf(freq: f32, gain_db: f32, sample_rate: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.75 - 1.0) + 2.0).sqrt();

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;
        let sqrt_a = a.sqrt();

        let b0 = a * (a_plus_1 - a_minus_1 * cos_omega + 2.0 * sqrt_a * alpha);
        let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_omega);
        let b2 = a * (a_plus_1 - a_minus_1 * cos_omega - 2.0 * sqrt_a * alpha);
        let a0 = a_plus_1 + a_minus_1 * cos_omega + 2.0 * sqrt_a * alpha;
        let a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_omega);
        let a2 = a_plus_1 + a_minus_1 * cos_omega - 2.0 * sqrt_a * alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// High shelf filter.
    fn high_shelf(freq: f32, gain_db: f32, sample_rate: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.75 - 1.0) + 2.0).sqrt();

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;
        let sqrt_a = a.sqrt();

        let b0 = a * (a_plus_1 + a_minus_1 * cos_omega + 2.0 * sqrt_a * alpha);
        let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_omega);
        let b2 = a * (a_plus_1 + a_minus_1 * cos_omega - 2.0 * sqrt_a * alpha);
        let a0 = a_plus_1 - a_minus_1 * cos_omega + 2.0 * sqrt_a * alpha;
        let a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_omega);
        let a2 = a_plus_1 - a_minus_1 * cos_omega - 2.0 * sqrt_a * alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

pub struct ParametricEq {
    low_state: BiquadState,
    mid_state: BiquadState,
    high_state: BiquadState,
    low_coeffs: BiquadCoeffs,
    mid_coeffs: BiquadCoeffs,
    high_coeffs: BiquadCoeffs,
    last_sample_rate: f32,
}

impl ParametricEq {
    pub fn new() -> Self {
        Self {
            low_state: BiquadState::new(),
            mid_state: BiquadState::new(),
            high_state: BiquadState::new(),
            low_coeffs: BiquadCoeffs::low_shelf(100.0, 0.0, 48000.0),
            mid_coeffs: BiquadCoeffs::peaking(1000.0, 0.0, 1.0, 48000.0),
            high_coeffs: BiquadCoeffs::high_shelf(5000.0, 0.0, 48000.0),
            last_sample_rate: 48000.0,
        }
    }
}

impl Default for ParametricEq {
    fn default() -> Self { Self::new() }
}

impl DspNode for ParametricEq {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        // Get parameters with validation
        let low_freq  = params.get(0).current.clamp(20.0, 500.0);
        let low_gain  = params.get(1).current.clamp(-24.0, 24.0);
        let mid_freq  = params.get(2).current.clamp(200.0, 5000.0);
        let mid_gain  = params.get(3).current.clamp(-24.0, 24.0);
        let mid_q     = params.get(4).current.clamp(0.1, 10.0);
        let high_freq = params.get(5).current.clamp(2000.0, 20000.0);
        let high_gain = params.get(6).current.clamp(-24.0, 24.0);

        // Update coefficients if sample rate changed or at start of buffer
        if (sample_rate - self.last_sample_rate).abs() > 0.1 {
            self.last_sample_rate = sample_rate;
            self.low_coeffs = BiquadCoeffs::low_shelf(low_freq, low_gain, sample_rate);
            self.mid_coeffs = BiquadCoeffs::peaking(mid_freq, mid_gain, mid_q, sample_rate);
            self.high_coeffs = BiquadCoeffs::high_shelf(high_freq, high_gain, sample_rate);
        }

        for i in 0..BUFFER_SIZE {
            let x = input[i];
            
            // Process through all three bands
            let y1 = self.low_state.process(x, &self.low_coeffs);
            let y2 = self.mid_state.process(y1, &self.mid_coeffs);
            let y3 = self.high_state.process(y2, &self.high_coeffs);
            
            output[i] = y3;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str { "ParametricEq" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_silence_passthrough() {
        let mut eq = ParametricEq::new();
        let mut params = ParamBlock::new();
        // Flat EQ (0dB gain on all bands)
        for &v in &[100.0f32, 0.0, 1000.0, 0.0, 1.0, 5000.0, 0.0] { params.add(v); }
        let input = [0.0f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        eq.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output { assert!(s.abs() < 1e-6, "silence should pass through"); }
    }

    #[test]
    fn test_eq_processes_signal() {
        let mut eq = ParametricEq::new();
        let mut params = ParamBlock::new();
        // Boost mid frequencies
        for &v in &[100.0f32, 0.0, 1000.0, 12.0, 1.0, 5000.0, 0.0] { params.add(v); }
        let input = [0.1f32; BUFFER_SIZE];
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        eq.process(&inputs, &mut output, &mut params, 48000.0);
        // Output should be non-zero
        assert!(output[BUFFER_SIZE - 1].abs() > 0.0, "EQ should process signal");
    }
}
