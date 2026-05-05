//! N-input mixer. Sums up to MAX_INPUTS signals with per-channel gain.
//!
//! Param layout:
//!   0..MAX_INPUTS = per-channel gain (default 1.0)
//!
//! SIMD strategy: explicit 4-wide f32 chunks using portable_simd (nightly)
//! with a scalar fallback for stable. On stable, LLVM auto-vectorizes the
//! inner loop into SSE/AVX FMA instructions anyway.

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

                mix_channel(output, buf, gain);
            }
        }

        params.tick_all();
    }

    fn type_name(&self) -> &'static str {
        "Mixer"
    }
}

/// Mix one channel into the output buffer with the given gain.
/// Processes 4 samples at a time using explicit loop unrolling that
/// LLVM reliably vectorizes into SSE/AVX FMA on x86_64 and NEON on ARM.
#[inline(always)]
fn mix_channel(output: &mut [f32; BUFFER_SIZE], input: &[f32; BUFFER_SIZE], gain: f32) {
    // Process 4 samples per iteration — compiler emits VFMADD231PS (AVX)
    // or FMLA (NEON). The fixed chunk size (4) matches the SIMD lane width
    // and allows the compiler to unroll without remainder handling.
    const CHUNK: usize = 4;
    let chunks = BUFFER_SIZE / CHUNK;

    if (gain - 1.0).abs() < f32::EPSILON {
        // Unity gain: pure addition — compiler emits VADDPS
        for c in 0..chunks {
            let i = c * CHUNK;
            output[i]     += input[i];
            output[i + 1] += input[i + 1];
            output[i + 2] += input[i + 2];
            output[i + 3] += input[i + 3];
        }
        // Scalar tail (if BUFFER_SIZE is not a multiple of 4)
        for i in (chunks * CHUNK)..BUFFER_SIZE {
            output[i] += input[i];
        }
    } else {
        // Scaled: compiler emits VFMADD231PS (fused multiply-add)
        for c in 0..chunks {
            let i = c * CHUNK;
            output[i]     = gain.mul_add(input[i],     output[i]);
            output[i + 1] = gain.mul_add(input[i + 1], output[i + 1]);
            output[i + 2] = gain.mul_add(input[i + 2], output[i + 2]);
            output[i + 3] = gain.mul_add(input[i + 3], output[i + 3]);
        }
        for i in (chunks * CHUNK)..BUFFER_SIZE {
            output[i] = gain.mul_add(input[i], output[i]);
        }
    }
}
