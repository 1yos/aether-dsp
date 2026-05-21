//! Stereo Panner — constant-power panning.
//!
//! Param layout:
//!   0 = pan  (-1.0 = left, 0.0 = center, 1.0 = right)
//!   1 = width (0.0 = mono, 1.0 = full stereo, >1.0 = wide)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Panner {
    // No state needed for constant-power panning
}

impl Panner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Panner {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for Panner {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input_l = inputs[0].unwrap_or(&silence);
        let input_r = inputs[1].unwrap_or(&silence);

        let pan = params.get(0).current.clamp(-1.0, 1.0);
        let width = params.get(1).current.clamp(0.0, 2.0);

        // Constant-power panning law (sin/cos for equal loudness)
        let pan_angle = (pan + 1.0) * 0.5 * std::f32::consts::FRAC_PI_2; // Map [-1,1] to [0, π/2]
        let left_gain = pan_angle.cos();
        let right_gain = pan_angle.sin();

        for i in 0..BUFFER_SIZE {
            // Mix input channels
            let mono = (input_l[i] + input_r[i]) * 0.5;
            let side = (input_l[i] - input_r[i]) * 0.5;

            // Apply width control
            let mid = mono;
            let side_scaled = side * width;

            // Apply panning
            let left = (mid + side_scaled) * left_gain;
            let right = (mid - side_scaled) * right_gain;

            // For mono output, mix to center
            output[i] = (left + right) * 0.5;

            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "Panner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panner_center() {
        let mut panner = Panner::new();
        let mut params = ParamBlock::new();
        // pan=0.0 (center), width=1.0 (full stereo)
        for &v in &[0.0f32, 1.0] {
            params.add(v);
        }

        let input_l = [0.5f32; BUFFER_SIZE];
        let input_r = [0.5f32; BUFFER_SIZE];
        let mut inputs = [None; MAX_INPUTS];
        inputs[0] = Some(&input_l);
        inputs[1] = Some(&input_r);
        let mut output = [0.0f32; BUFFER_SIZE];

        panner.process(&inputs, &mut output, &mut params, 48000.0);

        // Center panning should preserve signal (approximately)
        assert!(
            output[0] > 0.2 && output[0] < 0.8,
            "center pan should preserve level, got {}",
            output[0]
        );
    }

    #[test]
    fn test_panner_hard_left() {
        let mut panner = Panner::new();
        let mut params = ParamBlock::new();
        // pan=-1.0 (hard left), width=1.0
        for &v in &[-1.0f32, 1.0] {
            params.add(v);
        }

        let input_l = [1.0f32; BUFFER_SIZE];
        let input_r = [0.0f32; BUFFER_SIZE];
        let mut inputs = [None; MAX_INPUTS];
        inputs[0] = Some(&input_l);
        inputs[1] = Some(&input_r);
        let mut output = [0.0f32; BUFFER_SIZE];

        panner.process(&inputs, &mut output, &mut params, 48000.0);

        // Hard left should favor left channel
        assert!(output[0] > 0.3, "hard left should pass left channel");
    }

    #[test]
    fn test_panner_mono_width() {
        let mut panner = Panner::new();
        let mut params = ParamBlock::new();
        // pan=0.0 (center), width=0.0 (mono)
        for &v in &[0.0f32, 0.0] {
            params.add(v);
        }

        let input_l = [0.5f32; BUFFER_SIZE];
        let input_r = [0.5f32; BUFFER_SIZE];
        let mut inputs = [None; MAX_INPUTS];
        inputs[0] = Some(&input_l);
        inputs[1] = Some(&input_r);
        let mut output = [0.0f32; BUFFER_SIZE];

        panner.process(&inputs, &mut output, &mut params, 48000.0);

        // Mono width should collapse to center (approximately)
        assert!(
            output[0] > 0.2 && output[0] < 0.8,
            "mono width should collapse stereo, got {}",
            output[0]
        );
    }
}
