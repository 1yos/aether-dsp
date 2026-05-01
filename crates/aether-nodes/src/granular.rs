//! Granular synthesis node.
//!
//! Takes any audio input and produces a cloud of grains.
//! Feed a Krar recording → get textures no sample library can produce.
//! Feed a Djembe hit → get rhythmic clouds.
//!
//! Param layout:
//!   0 = grain_size   (ms, 10 – 500)
//!   1 = density      (grains/sec, 1 – 50)
//!   2 = pitch_scatter (semitones, 0 – 2)
//!   3 = position     (0.0 – 1.0, position in input buffer)
//!   4 = pos_scatter  (0.0 – 1.0)
//!   5 = wet          (0.0 – 1.0)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

const MAX_GRAIN_SAMPLES: usize = 48_000 / 2; // 500ms at 48kHz
const MAX_GRAINS: usize = 64;
const INPUT_BUF_SIZE: usize = 48_000 * 4; // 4 seconds of input

struct Grain {
    active: bool,
    pos: f64,       // current read position in input buffer
    speed: f64,     // playback speed (pitch)
    age: usize,     // samples since grain started
    duration: usize, // grain duration in samples
    amplitude: f32,
}

impl Grain {
    fn new() -> Self {
        Self { active: false, pos: 0.0, speed: 1.0, age: 0, duration: 1024, amplitude: 0.0 }
    }

    #[inline(always)]
    fn envelope(&self) -> f32 {
        // Hann window envelope
        let t = self.age as f32 / self.duration as f32;
        let hann = 0.5 * (1.0 - (std::f32::consts::TAU * t).cos());
        hann * self.amplitude
    }

    #[inline(always)]
    fn next_sample(&mut self, input_buf: &[f32]) -> f32 {
        if !self.active { return 0.0; }
        let env = self.envelope();
        let idx = self.pos as usize % input_buf.len();
        let frac = (self.pos - self.pos.floor()) as f32;
        let s0 = input_buf[idx];
        let s1 = input_buf[(idx + 1) % input_buf.len()];
        let sample = s0 + (s1 - s0) * frac;
        self.pos += self.speed;
        self.age += 1;
        if self.age >= self.duration { self.active = false; }
        sample * env
    }
}

pub struct Granular {
    grains: [Grain; MAX_GRAINS],
    input_buf: Box<[f32; INPUT_BUF_SIZE]>,
    write_pos: usize,
    samples_since_last_grain: usize,
    rng: u32,
}

impl Granular {
    pub fn new() -> Self {
        Self {
            grains: std::array::from_fn(|_| Grain::new()),
            input_buf: Box::new([0.0; INPUT_BUF_SIZE]),
            write_pos: 0,
            samples_since_last_grain: 0,
            rng: 0xDEAD_BEEF,
        }
    }

    fn rand_f32(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as f32 / u32::MAX as f32)
    }

    fn spawn_grain(&mut self, grain_size_ms: f32, pitch_scatter: f32, position: f32, pos_scatter: f32, sr: f32) {
        let duration = ((grain_size_ms / 1000.0) * sr) as usize;
        let duration = duration.clamp(64, MAX_GRAIN_SAMPLES);

        // Find an inactive grain slot
        let slot = self.grains.iter().position(|g| !g.active);
        let slot = match slot { Some(s) => s, None => return };

        // Position in input buffer
        let pos_center = (position + (self.rand_f32() - 0.5) * pos_scatter).clamp(0.0, 1.0);
        let buf_pos = (pos_center * INPUT_BUF_SIZE as f32) as usize;

        // Pitch: scatter in semitones
        let semitone_offset = (self.rand_f32() - 0.5) * 2.0 * pitch_scatter;
        let speed = 2.0f64.powf(semitone_offset as f64 / 12.0);

        self.grains[slot] = Grain {
            active: true,
            pos: buf_pos as f64,
            speed,
            age: 0,
            duration,
            amplitude: 0.7 + self.rand_f32() * 0.3,
        };
    }
}

impl Default for Granular {
    fn default() -> Self { Self::new() }
}

impl DspNode for Granular {
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
            let grain_size  = params.get(0).current.clamp(10.0, 500.0);
            let density     = params.get(1).current.clamp(1.0, 50.0);
            let pitch_scat  = params.get(2).current.clamp(0.0, 2.0);
            let position    = params.get(3).current.clamp(0.0, 1.0);
            let pos_scat    = params.get(4).current.clamp(0.0, 1.0);
            let wet         = params.get(5).current.clamp(0.0, 1.0);

            // Write input into circular buffer
            self.input_buf[self.write_pos] = input[i];
            self.write_pos = (self.write_pos + 1) % INPUT_BUF_SIZE;

            // Spawn grains at the requested density
            let samples_per_grain = (sample_rate / density) as usize;
            self.samples_since_last_grain += 1;
            if self.samples_since_last_grain >= samples_per_grain {
                self.samples_since_last_grain = 0;
                self.spawn_grain(grain_size, pitch_scat, position, pos_scat, sample_rate);
            }

            // Sum all active grains
            let mut wet_signal = 0.0f32;
            for grain in self.grains.iter_mut() {
                wet_signal += grain.next_sample(&*self.input_buf);
            }

            *out = input[i] * (1.0 - wet) + wet_signal * wet;
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str { "Granular" }
}
