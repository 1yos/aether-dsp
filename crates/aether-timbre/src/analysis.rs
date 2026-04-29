//! Spectral analysis — extract the timbre fingerprint of an instrument.

use rustfft::{FftPlanner, num_complex::Complex};

/// The spectral envelope of an instrument — its timbre fingerprint.
/// Stored as magnitude values across frequency bins.
#[derive(Debug, Clone)]
pub struct SpectralEnvelope {
    /// Magnitude per frequency bin (linear scale).
    pub magnitudes: Vec<f32>,
    /// FFT size used.
    pub fft_size: usize,
    /// Sample rate of the source audio.
    pub sample_rate: f32,
}

impl SpectralEnvelope {
    /// Analyze a buffer of audio and extract its spectral envelope.
    pub fn analyze(audio: &[f32], sample_rate: f32, fft_size: usize) -> Self {
        let fft_size = fft_size.next_power_of_two();
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);

        // Average multiple frames for a stable envelope
        let hop = fft_size / 4;
        let num_frames = (audio.len().saturating_sub(fft_size)) / hop + 1;
        let _ = num_frames; // used for documentation purposes only
        let mut avg_magnitudes = vec![0.0f32; fft_size / 2 + 1];

        let window: Vec<f32> = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos()))
            .collect();

        let mut frame_count = 0;
        let mut offset = 0;
        while offset + fft_size <= audio.len() {
            let mut buf: Vec<Complex<f32>> = audio[offset..offset + fft_size]
                .iter()
                .zip(window.iter())
                .map(|(&s, &w)| Complex::new(s * w, 0.0))
                .collect();

            fft.process(&mut buf);

            for (i, mag) in avg_magnitudes.iter_mut().enumerate() {
                *mag += buf[i].norm();
            }
            frame_count += 1;
            offset += hop;
        }

        if frame_count > 0 {
            for m in avg_magnitudes.iter_mut() {
                *m /= frame_count as f32;
            }
        }

        // Smooth the envelope (moving average)
        let smoothed = smooth_envelope(&avg_magnitudes, 8);

        Self {
            magnitudes: smoothed,
            fft_size,
            sample_rate,
        }
    }

    /// Normalize to peak = 1.0.
    pub fn normalize(&mut self) {
        let peak = self.magnitudes.iter().cloned().fold(0.0f32, f32::max);
        if peak > 0.0 {
            for m in self.magnitudes.iter_mut() {
                *m /= peak;
            }
        }
    }

    /// Interpolate magnitude at a given frequency (Hz).
    pub fn magnitude_at_freq(&self, freq: f32) -> f32 {
        let bin_width = self.sample_rate / self.fft_size as f32;
        let bin = freq / bin_width;
        let bin_floor = bin as usize;
        let frac = bin - bin_floor as f32;
        let m0 = self.magnitudes.get(bin_floor).copied().unwrap_or(0.0);
        let m1 = self.magnitudes.get(bin_floor + 1).copied().unwrap_or(0.0);
        m0 + (m1 - m0) * frac
    }
}

fn smooth_envelope(mags: &[f32], window: usize) -> Vec<f32> {
    let n = mags.len();
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let start = i.saturating_sub(window / 2);
        let end = (i + window / 2 + 1).min(n);
        let sum: f32 = mags[start..end].iter().sum();
        out[i] = sum / (end - start) as f32;
    }
    out
}

/// A complete timbre profile — multiple envelopes at different pitches/velocities.
#[derive(Debug, Clone)]
pub struct TimbreProfile {
    pub name: String,
    pub description: String,
    /// Envelopes at different MIDI notes (for pitch-dependent timbre).
    pub envelopes: Vec<(u8, SpectralEnvelope)>, // (midi_note, envelope)
    pub sample_rate: f32,
    pub fft_size: usize,
}

impl TimbreProfile {
    pub fn new(name: &str, sample_rate: f32, fft_size: usize) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            envelopes: Vec::new(),
            sample_rate,
            fft_size,
        }
    }

    /// Add an envelope for a specific MIDI note.
    pub fn add_envelope(&mut self, note: u8, audio: &[f32]) {
        let mut env = SpectralEnvelope::analyze(audio, self.sample_rate, self.fft_size);
        env.normalize();
        self.envelopes.push((note, env));
    }

    /// Get the best matching envelope for a given MIDI note.
    pub fn envelope_for_note(&self, note: u8) -> Option<&SpectralEnvelope> {
        if self.envelopes.is_empty() { return None; }
        let mut best_dist = u8::MAX;
        for (n, _env) in &self.envelopes {
            let dist = note.abs_diff(*n);
            if dist < best_dist {
                best_dist = dist;
            }
        }
        // Return the envelope from the best match
        self.envelopes.iter()
            .min_by_key(|(n, _)| note.abs_diff(*n))
            .map(|(_, e)| e)
    }
}
