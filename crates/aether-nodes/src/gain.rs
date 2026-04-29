//! Gain node — scales input by a linear gain factor.
//!
//! Param layout:
//!   0 = gain (linear, 0..4)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Gain;

impl DspNode for Gain {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);
        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * params.get(0).current;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "Gain"
    }
}
