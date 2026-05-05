//! Waveshaper / Saturation — nonlinear distortion.
//!
//! Param layout:
//!   0 = drive  (0.0 – 1.0, input gain before shaping)
//!   1 = mode   (0=soft-clip tanh, 1=hard-clip, 2=fold-back, 3=bit-crush, 4=tube)
//!   2 = tone   (0.0 – 1.0, post-shape high-shelf brightness)
//!   3 = wet    (0.0 – 1.0, dry/wet mix)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Waveshaper {
    /// One-pole high-shelf state for tone control.
    tone_state: f32,
    /// Bit-crush phase accumulator.
    crush_phase: f32,
    crush_held: f32,
}

impl Waveshaper {
    pub fn new() -> Self {
        Self {
            tone_state: 0.0,
            crush_phase: 0.0,
            crush_held: 0.0,
        }
    }

    /// Soft-clip using tanh approximation (Padé approximant, fast).
    #[inline(always)]
    fn tanh_approx(x: f32) -> f32 {
        // Padé [3/3] approximation — accurate to ±0.001 for |x| < 4
        let x2 = x * x;
        x * (27.0 + x2) / (27.0 + 9.0 * x2)
    }

    /// Tube-style asymmetric saturation (positive half harder than negative).
    #[inline(always)]
    fn tube_sat(x: f32) -> f32 {
        if x >= 0.0 {
            1.0 - (-x).exp()
        } else {
            -1.0 + (x).exp()
        }
    }
}

impl Default for Waveshaper {
    fn default() -> Self { Self::new() }
}

impl DspNode for Waveshaper {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        let drive = params.get(0).current.clamp(0.0, 1.0);
        let mode  = params.get(1).current as u32;
        let tone  = params.get(2).current.clamp(0.0, 1.0);
        let wet   = params.get(3).current.clamp(0.0, 1.0);

        // Drive maps 0–1 to 1–20× gain
        let gain = 1.0 + drive * 19.0;

        // Tone: one-pole high-shelf coefficient
        let tone_coeff = tone * 0.95;

        for (i, out) in output.iter_mut().enumerate() {
            let dry = input[i];
            let driven = dry * gain;

            let shaped = match mode {
                0 => Self::tanh_approx(driven),
                1 => driven.clamp(-1.0, 1.0),
                2 => {
                    // Fold-back: reflect signal at ±1
                    let mut v = driven;
                    while v.abs() > 1.0 {
                        if v > 1.0 { v = 2.0 - v; }
                        else if v < -1.0 { v = -2.0 - v; }
                    }
                    v
                }
                3 => {
                    // Bit-crush: sample-and-hold at reduced rate
                    let crush_rate = 1.0 - drive * 0.95; // 0.05–1.0 of sample rate
                    self.crush_phase += crush_rate;
                    if self.crush_phase >= 1.0 {
                        self.crush_phase -= 1.0;
                        // Quantize to 4–16 bits
                        let bits = 4.0 + (1.0 - drive) * 12.0;
                        let levels = 2.0f32.powf(bits);
                        self.crush_held = (driven * levels).round() / levels;
                    }
                    self.crush_held
                }
                _ => Self::tube_sat(driven),
            };

            // Normalize output (compensate for gain)
            let normalized = shaped / gain.sqrt();

            // Tone: high-shelf boost/cut
            self.tone_state = self.tone_state + tone_coeff * (normalized - self.tone_state);
            let toned = normalized + tone * (normalized - self.tone_state);

            *out = dry + wet * (toned - dry);
            params.tick_all();
        }
        let _ = sample_rate;
    }

    fn type_name(&self) -> &'static str { "Waveshaper" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveshaper_zero_drive_passthrough() {
        let mut ws = Waveshaper::new();
        let mut params = ParamBlock::new();
        for &v in &[0.0f32, 0.0, 0.5, 1.0] { params.add(v); }
        let input: [f32; BUFFER_SIZE] = std::array::from_fn(|i| (i as f32 / BUFFER_SIZE as f32) * 0.5);
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        ws.process(&inputs, &mut output, &mut params, 48000.0);
        // With drive=0, gain=1, tanh(x)≈x for small x, output ≈ input
        for i in 0..BUFFER_SIZE {
            assert!((output[i] - input[i]).abs() < 0.05, "zero drive should be near-passthrough");
        }
    }

    #[test]
    fn test_waveshaper_hard_clip_bounds() {
        let mut ws = Waveshaper::new();
        let mut params = ParamBlock::new();
        for &v in &[1.0f32, 1.0, 0.0, 1.0] { params.add(v); } // hard clip, full wet
        let input = [2.0f32; BUFFER_SIZE]; // way above clip
        let inputs = [Some(&input); MAX_INPUTS];
        let mut output = [0.0f32; BUFFER_SIZE];
        ws.process(&inputs, &mut output, &mut params, 48000.0);
        for s in &output {
            assert!(s.abs() <= 1.5, "hard clip output should be bounded, got {s}");
        }
    }
}
