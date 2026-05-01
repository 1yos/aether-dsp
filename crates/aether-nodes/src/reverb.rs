//! Freeverb algorithmic reverb.
//!
//! Classic Schroeder/Moorer design: 8 comb filters + 4 allpass filters.
//! Sounds like a real room. No heap allocation after initialization.
//!
//! Param layout:
//!   0 = room size (0.0 – 1.0)
//!   1 = damping   (0.0 – 1.0)
//!   2 = wet       (0.0 – 1.0)
//!   3 = width     (0.0 – 1.0, stereo spread)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

// Freeverb tuning constants (at 44.1kHz, scaled for other rates)
const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;
const SCALE_ROOM: f32 = 0.28;
const OFFSET_ROOM: f32 = 0.7;
const SCALE_DAMP: f32 = 0.4;
const FIXED_GAIN: f32 = 0.015;

struct CombFilter {
    buf: Vec<f32>,
    pos: usize,
    feedback: f32,
    damp1: f32,
    damp2: f32,
    filterstore: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size],
            pos: 0,
            feedback: 0.5,
            damp1: 0.5,
            damp2: 0.5,
            filterstore: 0.0,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buf[self.pos];
        self.filterstore = output * self.damp2 + self.filterstore * self.damp1;
        self.buf[self.pos] = input + self.filterstore * self.feedback;
        self.pos = (self.pos + 1) % self.buf.len();
        output
    }

    fn set_feedback(&mut self, v: f32) { self.feedback = v; }
    fn set_damp(&mut self, v: f32) { self.damp1 = v; self.damp2 = 1.0 - v; }
}

struct AllpassFilter {
    buf: Vec<f32>,
    pos: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self { buf: vec![0.0; size], pos: 0 }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let bufout = self.buf[self.pos];
        let output = -input + bufout;
        self.buf[self.pos] = input + bufout * 0.5;
        self.pos = (self.pos + 1) % self.buf.len();
        output
    }
}

pub struct Reverb {
    comb_l: [CombFilter; 8],
    comb_r: [CombFilter; 8],
    allpass_l: [AllpassFilter; 4],
    allpass_r: [AllpassFilter; 4],
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let scale = sample_rate / 44100.0;
        let scaled = |base: usize| ((base as f32 * scale) as usize).max(1);

        macro_rules! make_combs {
            ($spread:expr) => {
                [
                    CombFilter::new(scaled(COMB_TUNING[0] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[1] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[2] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[3] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[4] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[5] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[6] + $spread)),
                    CombFilter::new(scaled(COMB_TUNING[7] + $spread)),
                ]
            };
        }

        macro_rules! make_allpasses {
            ($spread:expr) => {
                [
                    AllpassFilter::new(scaled(ALLPASS_TUNING[0] + $spread)),
                    AllpassFilter::new(scaled(ALLPASS_TUNING[1] + $spread)),
                    AllpassFilter::new(scaled(ALLPASS_TUNING[2] + $spread)),
                    AllpassFilter::new(scaled(ALLPASS_TUNING[3] + $spread)),
                ]
            };
        }

        let mut r = Self {
            comb_l: make_combs!(0),
            comb_r: make_combs!(STEREO_SPREAD),
            allpass_l: make_allpasses!(0),
            allpass_r: make_allpasses!(STEREO_SPREAD),
        };
        r.set_params(0.5, 0.5);
        r
    }

    fn set_params(&mut self, room_size: f32, damping: f32) {
        let feedback = room_size * SCALE_ROOM + OFFSET_ROOM;
        let damp = damping * SCALE_DAMP;
        for c in self.comb_l.iter_mut().chain(self.comb_r.iter_mut()) {
            c.set_feedback(feedback);
            c.set_damp(damp);
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, input: f32) -> (f32, f32) {
        let input_gain = input * FIXED_GAIN;
        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;

        for c in &mut self.comb_l { out_l += c.process(input_gain); }
        for c in &mut self.comb_r { out_r += c.process(input_gain); }
        for a in &mut self.allpass_l { out_l = a.process(out_l); }
        for a in &mut self.allpass_r { out_r = a.process(out_r); }

        (out_l, out_r)
    }
}

impl DspNode for Reverb {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        for (i, out) in output.iter_mut().enumerate() {
            let room = params.get(0).current.clamp(0.0, 1.0);
            let damp = params.get(1).current.clamp(0.0, 1.0);
            let wet = params.get(2).current.clamp(0.0, 1.0);

            self.set_params(room, damp);
            let (wet_l, wet_r) = self.process_sample(input[i]);
            // Mono mix of stereo reverb output
            *out = input[i] * (1.0 - wet) + (wet_l + wet_r) * 0.5 * wet;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "Reverb"
    }
}
