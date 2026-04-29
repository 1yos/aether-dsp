//! N-input mixer. Sums up to MAX_INPUTS signals with per-channel gain.
//!
//! Param layout:
//!   0..MAX_INPUTS = per-channel gain (default 1.0)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

pub struct Mixer;

impl DspNode for Mixer {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        output.fill(0.0);
        for (slot, maybe_input) in inputs.iter().enumerate() {
            if let Some(buf) = maybe_input {
                let gain = if slot < params.count {
                    params.get(slot).current
                } else {
                    1.0
                };
                for (i, out) in output.iter_mut().enumerate() {
                    *out += buf[i] * gain;
                }
            }
        }
        params.tick_all();
    }

    fn type_name(&self) -> &'static str {
        "Mixer"
    }
}
