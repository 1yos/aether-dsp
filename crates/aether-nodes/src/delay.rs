//! Feedback delay line.
//!
//! Uses a pre-allocated circular buffer. No heap activity in the RT path.
//!
//! Param layout:
//!   0 = delay time (seconds, max 2.0)
//!   1 = feedback   (0..0.99)
//!   2 = wet mix    (0..1)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

/// Maximum delay time in seconds. Determines pre-allocated buffer size.
const MAX_DELAY_SECS: f32 = 2.0;
const MAX_DELAY_SAMPLES: usize = (48_000.0 * MAX_DELAY_SECS) as usize;

pub struct DelayLine {
    buffer: Box<[f32; MAX_DELAY_SAMPLES]>,
    write_pos: usize,
}

impl DelayLine {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0.0f32; MAX_DELAY_SAMPLES]),
            write_pos: 0,
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, input: f32, delay_samples: usize, feedback: f32, wet: f32) -> f32 {
        let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - delay_samples) % MAX_DELAY_SAMPLES;
        let delayed = self.buffer[read_pos];
        self.buffer[self.write_pos] = input + delayed * feedback;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;
        input * (1.0 - wet) + delayed * wet
    }
}

impl Default for DelayLine {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for DelayLine {
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
            let delay_secs = params.get(0).current.clamp(0.0, MAX_DELAY_SECS);
            let feedback = params.get(1).current.clamp(0.0, 0.99);
            let wet = params.get(2).current.clamp(0.0, 1.0);
            let delay_samples =
                ((delay_secs * sample_rate) as usize).clamp(1, MAX_DELAY_SAMPLES - 1);

            *out = self.process_sample(input[i], delay_samples, feedback, wet);
            params.tick_all();
        }
    }

    fn type_name(&self) -> &'static str {
        "DelayLine"
    }
}
