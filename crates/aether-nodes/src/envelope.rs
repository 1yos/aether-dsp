//! ADSR envelope generator.
//!
//! Param layout:
//!   0 = attack  (seconds)
//!   1 = decay   (seconds)
//!   2 = sustain (0..1 level)
//!   3 = release (seconds)
//!   4 = gate    (0=off, 1=on)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
struct EnvState {
    level: f32,
    stage: u8,
}

pub struct AdsrEnvelope {
    level: f32,
    stage: Stage,
    prev_gate: f32,
}

impl AdsrEnvelope {
    pub fn new() -> Self {
        Self {
            level: 0.0,
            stage: Stage::Idle,
            prev_gate: 0.0,
        }
    }

    #[inline(always)]
    fn tick_sample(
        &mut self,
        gate: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        sr: f32,
    ) -> f32 {
        // Gate edge detection.
        if gate > 0.5 && self.prev_gate <= 0.5 {
            self.stage = Stage::Attack;
        } else if gate <= 0.5 && self.prev_gate > 0.5 {
            self.stage = Stage::Release;
        }
        self.prev_gate = gate;

        let attack_rate = if attack > 0.0 {
            1.0 / (attack * sr)
        } else {
            1.0
        };
        let decay_rate = if decay > 0.0 { 1.0 / (decay * sr) } else { 1.0 };
        let release_rate = if release > 0.0 {
            1.0 / (release * sr)
        } else {
            1.0
        };

        match self.stage {
            Stage::Idle => {}
            Stage::Attack => {
                self.level += attack_rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.level -= decay_rate;
                if self.level <= sustain {
                    self.level = sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {
                self.level = sustain;
            }
            Stage::Release => {
                self.level -= release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.level
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for AdsrEnvelope {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        // Input 0 can be an audio signal to modulate; if absent, output raw envelope.
        let _silence = [0.0f32; BUFFER_SIZE];
        let audio_in = inputs[0];

        for (i, out) in output.iter_mut().enumerate() {
            let attack = params.get(0).current.max(0.0);
            let decay = params.get(1).current.max(0.0);
            let sustain = params.get(2).current.clamp(0.0, 1.0);
            let release = params.get(3).current.max(0.0);
            let gate = params.get(4).current;

            let env = self.tick_sample(gate, attack, decay, sustain, release, sample_rate);
            *out = match audio_in {
                Some(buf) => buf[i] * env,
                None => env,
            };
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        StateBlob::from_value(&EnvState {
            level: self.level,
            stage: self.stage as u8,
        })
    }

    fn restore_state(&mut self, state: StateBlob) {
        let s: EnvState = state.to_value();
        self.level = s.level;
        self.stage = match s.stage {
            0 => Stage::Idle,
            1 => Stage::Attack,
            2 => Stage::Decay,
            3 => Stage::Sustain,
            _ => Stage::Release,
        };
    }

    fn type_name(&self) -> &'static str {
        "AdsrEnvelope"
    }
}
