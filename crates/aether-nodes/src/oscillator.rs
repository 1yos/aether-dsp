//! Band-limited wavetable oscillator.
//!
//! Param layout:
//!   0 = frequency (Hz)
//!   1 = amplitude (0..1)
//!   2 = waveform  (0=sine, 1=saw, 2=square, 3=triangle)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
struct OscState {
    phase: f32,
}

pub struct Oscillator {
    phase: f32,
}

impl Oscillator {
    pub fn new() -> Self {
        Self { phase: 0.0 }
    }

    #[inline(always)]
    fn generate_sample(&mut self, freq: f32, amp: f32, waveform: f32, sr: f32) -> f32 {
        let phase_inc = freq / sr;

        // BLEP anti-aliasing for discontinuous waveforms
        let sample = match waveform as u32 {
            0 => (self.phase * TAU).sin(),
            1 => {
                // Band-limited sawtooth using BLEP
                let mut saw = 2.0 * self.phase - 1.0;
                saw -= blep(self.phase, phase_inc);
                saw
            }
            2 => {
                // Band-limited square using BLEP
                let mut sq = if self.phase < 0.5 { 1.0f32 } else { -1.0f32 };
                sq += blep(self.phase, phase_inc);
                sq -= blep((self.phase + 0.5).fract(), phase_inc);
                sq
            }
            _ => {
                // Triangle (integrated square — already band-limited)
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
        };
        self.phase = (self.phase + phase_inc).fract();
        sample * amp
    }
}

/// Polynomial Band-Limited Step (BLEP) correction.
/// Reduces aliasing at waveform discontinuities.
/// `t` = current phase, `dt` = phase increment per sample.
#[inline(always)]
fn blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for Oscillator {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        for sample in output.iter_mut() {
            let freq = params.get(0).current.max(0.01);
            let amp = params.get(1).current.clamp(0.0, 1.0);
            let wave = params.get(2).current;
            *sample = self.generate_sample(freq, amp, wave, sample_rate);
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        StateBlob::from_value(&OscState { phase: self.phase })
    }

    fn restore_state(&mut self, state: StateBlob) {
        let s: OscState = state.to_value();
        self.phase = s.phase;
    }

    fn type_name(&self) -> &'static str {
        "Oscillator"
    }
}
