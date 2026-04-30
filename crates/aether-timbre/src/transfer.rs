//! Timbre transfer — apply a target instrument's spectral envelope to a source signal.

use rustfft::{FftPlanner, num_complex::Complex};
use crate::analysis::SpectralEnvelope;

/// Applies spectral envelope transfer in real time using overlap-add.
pub struct TimbreTransfer {
    fft_size: usize,
    hop_size: usize,
    /// Target spectral envelope to impose.
    target_envelope: Option<SpectralEnvelope>,
    /// Transfer amount: 0.0 = no transfer, 1.0 = full transfer.
    pub amount: f32,
    /// Input overlap buffer.
    #[allow(dead_code)]
    input_buffer: Vec<f32>,
    /// Output overlap-add buffer.
    #[allow(dead_code)]
    output_buffer: Vec<f32>,
    /// Analysis window.
    window: Vec<f32>,
    planner: FftPlanner<f32>,
}

impl TimbreTransfer {
    pub fn new(fft_size: usize) -> Self {
        let fft_size = fft_size.next_power_of_two();
        let hop_size = fft_size / 4;
        let window: Vec<f32> = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos()))
            .collect();
        Self {
            fft_size,
            hop_size,
            target_envelope: None,
            amount: 1.0,
            input_buffer: vec![0.0; fft_size * 2],
            output_buffer: vec![0.0; fft_size * 2],
            window,
            planner: FftPlanner::new(),
        }
    }

    /// Set the target spectral envelope.
    pub fn set_target(&mut self, envelope: SpectralEnvelope) {
        self.target_envelope = Some(envelope);
    }

    /// Clear the target (pass-through mode).
    pub fn clear_target(&mut self) {
        self.target_envelope = None;
    }

    /// Process a block of audio samples.
    /// Returns the processed output (same length as input).
    pub fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        if self.target_envelope.is_none() || self.amount < 0.001 {
            return input.to_vec();
        }

        let target = self.target_envelope.as_ref().unwrap();
        let fft = self.planner.plan_fft_forward(self.fft_size);
        let ifft = self.planner.plan_fft_inverse(self.fft_size);

        let mut output = vec![0.0f32; input.len()];

        // Simple single-frame processing for now (full overlap-add in v0.2)
        // Process in fft_size chunks
        let mut pos = 0;
        while pos + self.fft_size <= input.len() {
            // Apply window
            let mut buf: Vec<Complex<f32>> = input[pos..pos + self.fft_size]
                .iter()
                .zip(self.window.iter())
                .map(|(&s, &w)| Complex::new(s * w, 0.0))
                .collect();

            // Forward FFT
            fft.process(&mut buf);

            // Extract source envelope and apply target envelope
            let n_bins = self.fft_size / 2 + 1;
            let mut source_env = vec![0.0f32; n_bins];
            for i in 0..n_bins {
                source_env[i] = buf[i].norm().max(1e-10);
            }
            let smoothed_source = smooth(&source_env, 4);

            for i in 0..n_bins {
                let src_mag = smoothed_source[i];
                let tgt_mag = target.magnitudes.get(i).copied().unwrap_or(1.0).max(1e-10);
                let ratio = (tgt_mag / src_mag).powf(self.amount);
                // Apply ratio to both positive and negative frequency bins
                buf[i] *= ratio;
                if i > 0 && i < self.fft_size - i {
                    buf[self.fft_size - i] *= ratio;
                }
            }

            // Inverse FFT
            ifft.process(&mut buf);

            // Normalize and overlap-add
            let norm = 1.0 / self.fft_size as f32;
            for (j, s) in buf.iter().enumerate().take(self.fft_size) {
                if pos + j < output.len() {
                    let dry = input[pos + j];
                    let wet = s.re * norm;
                    output[pos + j] = dry * (1.0 - self.amount) + wet * self.amount;
                }
            }

            pos += self.hop_size;
        }

        // Copy remaining samples unchanged
        if pos < input.len() {
            output[pos..].copy_from_slice(&input[pos..]);
        }

        output
    }
}

fn smooth(v: &[f32], w: usize) -> Vec<f32> {
    let n = v.len();
    let mut out = vec![0.0f32; n];
    for (i, val) in out.iter_mut().enumerate() {
        let s = i.saturating_sub(w / 2);
        let e = (i + w / 2 + 1).min(n);
        *val = v[s..e].iter().sum::<f32>() / (e - s) as f32;
    }
    out
}
