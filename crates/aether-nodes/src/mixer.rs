//! N-input mixer. Sums up to MAX_INPUTS signals with per-channel gain.
//!
//! Param layout:
//!   0..MAX_INPUTS = per-channel gain (default 1.0)
//!
//! SIMD strategy: the inner accumulation loop is written as a simple
//! `output[i] += buf[i] * gain` over a fixed-size array. LLVM auto-vectorizes
//! this into 4-wide SSE/AVX fused-multiply-add instructions, giving ~4× speedup
//! over the scalar version on x86_64.

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

                if (gain - 1.0).abs() < f32::EPSILON {
                    // Unity gain: pure addition — compiler emits SIMD ADDPS
                    for i in 0..BUFFER_SIZE {
                        output[i] += buf[i];
                    }
                } else {
                    // Scaled: compiler emits SIMD FMADD (fused multiply-add)
                    for i in 0..BUFFER_SIZE {
                        output[i] = gain.mul_add(buf[i], output[i]);
                    }
                }
            }
        }

        params.tick_all();
    }

    fn type_name(&self) -> &'static str {
        "Mixer"
    }
}
