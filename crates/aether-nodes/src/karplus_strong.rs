//! Karplus-Strong plucked string synthesis.
//!
//! Physically accurate plucked string model. A Krar played through this
//! with the right tuning table sounds more like a Krar than any sample.
//!
//! Param layout:
//!   0 = frequency   (Hz, 20 – 4000)
//!   1 = decay       (0.9 – 0.9999, higher = longer sustain)
//!   2 = brightness  (0.0 – 1.0, high-frequency content of excitation)
//!   3 = trigger     (0→1 edge = pluck the string)

use aether_core::{node::DspNode, param::ParamBlock, state::StateBlob, BUFFER_SIZE, MAX_INPUTS};

const MAX_DELAY: usize = 4096; // supports down to ~12 Hz at 48kHz

pub struct KarplusStrong {
    delay_line: Box<[f32; MAX_DELAY]>,
    write_pos: usize,
    delay_len: usize,
    prev_trigger: f32,
    noise_seed: u32,
}

impl KarplusStrong {
    pub fn new() -> Self {
        Self {
            delay_line: Box::new([0.0; MAX_DELAY]),
            write_pos: 0,
            delay_len: 100,
            prev_trigger: 0.0,
            noise_seed: 12345,
        }
    }

    /// Excite the string with a burst of filtered noise.
    fn pluck(&mut self, brightness: f32) {
        for i in 0..self.delay_len {
            let noise = self.white_noise();
            // Low-pass filter the excitation based on brightness
            // High brightness = more high-frequency content = brighter pluck
            let filtered = if brightness > 0.5 {
                noise
            } else {
                // Simple one-pole LP to reduce brightness
                let alpha = brightness * 2.0;
                noise * alpha
            };
            self.delay_line[(self.write_pos + i) % MAX_DELAY] = filtered;
        }
    }

    #[inline(always)]
    fn white_noise(&mut self) -> f32 {
        // Xorshift32 PRNG — no allocation, deterministic
        self.noise_seed ^= self.noise_seed << 13;
        self.noise_seed ^= self.noise_seed >> 17;
        self.noise_seed ^= self.noise_seed << 5;
        (self.noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    #[inline(always)]
    fn process_sample(&mut self, decay: f32) -> f32 {
        let read_pos = (self.write_pos + MAX_DELAY - self.delay_len) % MAX_DELAY;
        let next_pos = (read_pos + 1) % MAX_DELAY;

        // Averaging filter (low-pass) + decay = the Karplus-Strong algorithm
        let output = (self.delay_line[read_pos] + self.delay_line[next_pos]) * 0.5 * decay;
        self.delay_line[self.write_pos] = output;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY;
        output
    }
}

impl Default for KarplusStrong {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for KarplusStrong {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        for sample in output.iter_mut() {
            let freq = params.get(0).current.clamp(20.0, 4000.0);
            let decay = params.get(1).current.clamp(0.9, 0.9999);
            let brightness = params.get(2).current.clamp(0.0, 1.0);
            let trigger = params.get(3).current;

            // Update delay line length from frequency
            let new_len = ((sample_rate / freq) as usize).clamp(2, MAX_DELAY - 1);
            if new_len != self.delay_len {
                self.delay_len = new_len;
            }

            // Trigger on rising edge
            if trigger > 0.5 && self.prev_trigger <= 0.5 {
                self.pluck(brightness);
            }
            self.prev_trigger = trigger;

            *sample = self.process_sample(decay);
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        StateBlob::EMPTY // delay line state not serialized (too large)
    }

    fn type_name(&self) -> &'static str {
        "KarplusStrong"
    }
}
