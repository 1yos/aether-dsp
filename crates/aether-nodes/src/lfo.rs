//! Low-Frequency Oscillator (LFO) — modulation source.
//!
//! Outputs a -1.0 to 1.0 modulation signal. Not audio-rate.
//!
//! Param layout:
//!   0 = rate     (Hz, 0.01 – 20.0)
//!   1 = depth    (0.0 – 1.0)
//!   2 = waveform (0=sine, 1=triangle, 2=square, 3=sample-and-hold, 4=random-smooth)
//!   3 = phase    (0.0 – 1.0, initial phase offset)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
struct LfoState {
    phase: f32,
    held_value: f32,
    smooth_target: f32,
    smooth_current: f32,
}

pub struct Lfo {
    phase: f32,
    held_value: f32,
    smooth_target: f32,
    smooth_current: f32,
    prev_phase: f32,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            held_value: 0.0,
            smooth_target: 0.0,
            smooth_current: 0.0,
            prev_phase: 0.0,
        }
    }

    #[inline(always)]
    fn next_sample(&mut self, rate: f32, depth: f32, waveform: u32, sr: f32) -> f32 {
        let phase_inc = rate / sr;
        let crossed_zero = self.phase < self.prev_phase; // wrapped around

        let raw = match waveform {
            0 => (self.phase * TAU).sin(),
            1 => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            2 => if self.phase < 0.5 { 1.0 } else { -1.0 },
            3 => {
                // Sample-and-hold: new random value on each cycle
                if crossed_zero {
                    self.held_value = pseudo_random(self.phase) * 2.0 - 1.0;
                }
                self.held_value
            }
            _ => {
                // Random smooth: interpolate toward new target each cycle
                if crossed_zero {
                    self.smooth_target = pseudo_random(self.phase) * 2.0 - 1.0;
                }
                let smooth_rate = rate / sr * 0.1;
                self.smooth_current += (self.smooth_target - self.smooth_current) * smooth_rate;
                self.smooth_current
            }
        };

        self.prev_phase = self.phase;
        self.phase = (self.phase + phase_inc).fract();
        raw * depth
    }
}

/// Simple deterministic pseudo-random from phase value.
#[inline(always)]
fn pseudo_random(seed: f32) -> f32 {
    let x = (seed * 127.1 + 311.7).sin() * 43758.5453;
    x - x.floor()
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for Lfo {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        for sample in output.iter_mut() {
            let rate = params.get(0).current.clamp(0.01, 20.0);
            let depth = params.get(1).current.clamp(0.0, 1.0);
            let waveform = params.get(2).current as u32;
            *sample = self.next_sample(rate, depth, waveform, sample_rate);
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        StateBlob::from_value(&LfoState {
            phase: self.phase,
            held_value: self.held_value,
            smooth_target: self.smooth_target,
            smooth_current: self.smooth_current,
        })
    }

    fn restore_state(&mut self, state: StateBlob) {
        let s: LfoState = state.to_value();
        self.phase = s.phase;
        self.held_value = s.held_value;
        self.smooth_target = s.smooth_target;
        self.smooth_current = s.smooth_current;
    }

    fn type_name(&self) -> &'static str {
        "Lfo"
    }
}
