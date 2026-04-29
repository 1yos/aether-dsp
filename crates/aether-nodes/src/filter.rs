//! State-variable filter (SVF) — simultaneous LP/HP/BP outputs.
//!
//! Param layout:
//!   0 = cutoff frequency (Hz)
//!   1 = resonance Q (0.5..20)
//!   2 = mode (0=LP, 1=HP, 2=BP)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};

#[derive(Clone, Copy, Default)]
struct SvfState {
    ic1eq: f32,
    ic2eq: f32,
}

pub struct StateVariableFilter {
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    pub fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    /// Andy Simper's SVF (Cytomic) — topology-preserving transform.
    #[inline(always)]
    fn process_sample(&mut self, input: f32, cutoff: f32, q: f32, sr: f32) -> (f32, f32, f32) {
        let g = (std::f32::consts::PI * cutoff / sr).tan();
        let k = 1.0 / q.max(0.1);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        let lp = v2;
        let bp = v1;
        let hp = input - k * v1 - v2;
        (lp, hp, bp)
    }
}

impl Default for StateVariableFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for StateVariableFilter {
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
            let cutoff = params.get(0).current.clamp(20.0, sample_rate * 0.49);
            let q = params.get(1).current.clamp(0.5, 20.0);
            let mode = params.get(2).current as u32;

            let (lp, hp, bp) = self.process_sample(input[i], cutoff, q, sample_rate);
            *out = match mode {
                0 => lp,
                1 => hp,
                _ => bp,
            };
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        StateBlob::from_value(&SvfState {
            ic1eq: self.ic1eq,
            ic2eq: self.ic2eq,
        })
    }

    fn restore_state(&mut self, state: StateBlob) {
        let s: SvfState = state.to_value();
        self.ic1eq = s.ic1eq;
        self.ic2eq = s.ic2eq;
    }

    fn type_name(&self) -> &'static str {
        "StateVariableFilter"
    }
}
