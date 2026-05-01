//! Moog ladder filter — the filter that defined synthesizer sound.
//!
//! Huovilainen model: accurate self-oscillation, stable at high resonance,
//! correct at audio-rate cutoff modulation.
//!
//! Param layout:
//!   0 = cutoff     (Hz, 20 – 20000)
//!   1 = resonance  (0.0 – 4.0, self-oscillates above ~3.8)
//!   2 = drive      (0.0 – 1.0, input saturation)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};
use std::f32::consts::PI;

#[derive(Clone, Copy, Default)]
struct LadderState {
    stage: [f32; 4],
    stage_tanh: [f32; 4],
    delay: [f32; 6],
}

pub struct MoogLadder {
    state: LadderState,
    thermal: f32,
}

impl MoogLadder {
    pub fn new() -> Self {
        Self {
            state: LadderState::default(),
            thermal: 0.000_025, // thermal voltage
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, input: f32, cutoff: f32, resonance: f32, drive: f32, sr: f32) -> f32 {
        let f = cutoff / (sr * 0.5);
        let f = f.clamp(0.0, 1.0);

        // Huovilainen's nonlinear ladder
        let fc = f * PI;
        let fc2 = fc * fc;
        let fc3 = fc2 * fc;

        let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
        let acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;

        let f2 = (2.0 / 1.3) * f;
        let res4 = resonance * acr;

        // Input with drive saturation
        let inp = (input * (1.0 + drive * 3.0)).tanh();

        // Four-stage ladder with feedback
        let inp_sub = inp - res4 * self.state.delay[5];

        // Stage 1
        let t1 = self.state.stage[0] * fcr;
        let t2 = self.state.delay[0] * fcr;
        self.state.stage[0] = inp_sub * f2 - t1;
        self.state.delay[0] = self.state.stage[0] + t1;
        let out1 = self.state.delay[0].tanh();

        // Stage 2
        let t1 = self.state.stage[1] * fcr;
        let t2_2 = self.state.delay[1] * fcr;
        self.state.stage[1] = out1 * f2 - t1;
        self.state.delay[1] = self.state.stage[1] + t1;
        let out2 = self.state.delay[1].tanh();

        // Stage 3
        let t1 = self.state.stage[2] * fcr;
        let _t2_3 = self.state.delay[2] * fcr;
        self.state.stage[2] = out2 * f2 - t1;
        self.state.delay[2] = self.state.stage[2] + t1;
        let out3 = self.state.delay[2].tanh();

        // Stage 4
        let t1 = self.state.stage[3] * fcr;
        let _t2_4 = self.state.delay[3] * fcr;
        self.state.stage[3] = out3 * f2 - t1;
        self.state.delay[3] = self.state.stage[3] + t1;
        let out4 = self.state.delay[3];

        // Feedback
        self.state.delay[5] = (self.state.delay[4] + out4) * 0.5;
        self.state.delay[4] = out4;

        // Suppress unused variable warnings
        let _ = (t2, t2_2, fc3);

        out4
    }
}

impl Default for MoogLadder {
    fn default() -> Self { Self::new() }
}

impl DspNode for MoogLadder {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        for (i, out) in output.iter_mut().enumerate() {
            let cutoff    = params.get(0).current.clamp(20.0, sample_rate * 0.45);
            let resonance = params.get(1).current.clamp(0.0, 4.0);
            let drive     = params.get(2).current.clamp(0.0, 1.0);
            *out = self.process_sample(input[i], cutoff, resonance, drive, sample_rate);
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob { StateBlob::EMPTY }
    fn type_name(&self) -> &'static str { "MoogLadder" }
}
