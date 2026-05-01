//! Formant filter — shapes audio to sound like a vowel.
//!
//! Essential for Ney, Bansuri, and any wind instrument synthesis.
//! Uses three parallel bandpass filters tuned to vowel formant frequencies.
//!
//! Param layout:
//!   0 = vowel  (0=A, 1=E, 2=I, 3=O, 4=U, fractional = morph between vowels)
//!   1 = shift  (semitones, -12 to +12, transposes all formants)
//!   2 = wet    (0.0 – 1.0)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

/// Formant frequencies (Hz) for each vowel: [F1, F2, F3]
/// Based on average male voice formants.
const VOWEL_FORMANTS: [[f32; 3]; 5] = [
    [800.0,  1200.0, 2500.0], // A
    [400.0,  2000.0, 2800.0], // E
    [350.0,  2800.0, 3300.0], // I
    [450.0,  800.0,  2830.0], // O
    [325.0,  700.0,  2700.0], // U
];

/// Bandwidths (Hz) for each formant
const FORMANT_BW: [f32; 3] = [80.0, 90.0, 120.0];

struct BandpassFilter {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl BandpassFilter {
    fn new() -> Self { Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 } }

    #[inline(always)]
    fn process(&mut self, input: f32, freq: f32, bw: f32, sr: f32) -> f32 {
        let r = 1.0 - std::f32::consts::PI * bw / sr;
        let cos_w = 2.0 * r * (2.0 * std::f32::consts::PI * freq / sr).cos();
        let a0 = 1.0 - r * r;
        let y = a0 * input + cos_w * self.y1 - r * r * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        self.x2 = self.x1;
        self.x1 = input;
        y
    }
}

pub struct FormantFilter {
    bp: [[BandpassFilter; 3]; 2], // two sets for morphing
}

impl FormantFilter {
    pub fn new() -> Self {
        Self {
            bp: [
                [BandpassFilter::new(), BandpassFilter::new(), BandpassFilter::new()],
                [BandpassFilter::new(), BandpassFilter::new(), BandpassFilter::new()],
            ],
        }
    }

    fn get_formants(vowel_idx: usize, shift_semitones: f32) -> [f32; 3] {
        let v = vowel_idx.min(4);
        let shift_ratio = 2.0f32.powf(shift_semitones / 12.0);
        [
            VOWEL_FORMANTS[v][0] * shift_ratio,
            VOWEL_FORMANTS[v][1] * shift_ratio,
            VOWEL_FORMANTS[v][2] * shift_ratio,
        ]
    }
}

impl Default for FormantFilter {
    fn default() -> Self { Self::new() }
}

impl DspNode for FormantFilter {
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
            let vowel_f = params.get(0).current.clamp(0.0, 4.0);
            let shift   = params.get(1).current.clamp(-12.0, 12.0);
            let wet     = params.get(2).current.clamp(0.0, 1.0);

            let v0 = vowel_f as usize;
            let v1 = (v0 + 1).min(4);
            let frac = vowel_f.fract();

            let f0 = Self::get_formants(v0, shift);
            let f1 = Self::get_formants(v1, shift);

            // Process through two sets of formant filters and interpolate
            let mut wet_signal = 0.0f32;
            for k in 0..3 {
                let freq0 = f0[k] * (1.0 - frac) + f1[k] * frac;
                wet_signal += self.bp[0][k].process(input[i], freq0, FORMANT_BW[k], sample_rate);
            }
            wet_signal *= 0.333; // normalize 3 filters

            *out = input[i] * (1.0 - wet) + wet_signal * wet;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str { "FormantFilter" }
}
