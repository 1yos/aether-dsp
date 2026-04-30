//! RecordNode — taps audio into a lock-free SPSC ring buffer.
//!
//! Passes input through to output unchanged (unity gain).
//! If the ring is full, samples are dropped silently.
//! No allocation, no locks in the hot path.

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use ringbuf::{traits::Producer, HeapProd};

pub struct RecordNode {
    producer: HeapProd<f32>,
}

impl RecordNode {
    pub fn new(producer: HeapProd<f32>) -> Self {
        Self { producer }
    }
}

impl DspNode for RecordNode {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let silence = [0.0f32; BUFFER_SIZE];
        let input = inputs[0].unwrap_or(&silence);

        // Attempt to push; silently drop if ring is full.
        let _ = self.producer.push_slice(input);

        // Pass-through: copy input to output.
        output.copy_from_slice(input);
    }

    fn type_name(&self) -> &'static str {
        "RecordNode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use ringbuf::{traits::{Consumer, Split}, HeapRb};

    /// Generate a [f32; BUFFER_SIZE] array via proptest (BUFFER_SIZE = 64).
    fn audio_buffer() -> impl Strategy<Value = [f32; BUFFER_SIZE]> {
        prop::collection::vec(-1.0f32..=1.0f32, BUFFER_SIZE)
            .prop_map(|v| v.try_into().unwrap())
    }

    // Property 3
    proptest! {
        /// **Validates: Requirements 2.11**
        ///
        /// Property 3: RecordNode pass-through.
        ///
        /// For any input buffer passed to `RecordNode::process()`, the output buffer
        /// SHALL be identical to the input buffer (unity gain pass-through).
        #[test]
        fn prop_record_node_pass_through(
            input_samples in audio_buffer(),
        ) {
            let ring = HeapRb::<f32>::new(BUFFER_SIZE * 2);
            let (producer, _consumer) = ring.split();
            let mut node = RecordNode::new(producer);
            let input_buffer: [f32; BUFFER_SIZE] = input_samples;
            let mut output_buffer = [0.0f32; BUFFER_SIZE];
            let mut params = ParamBlock::default();
            let inputs: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] = {
                let mut arr: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] = [None; MAX_INPUTS];
                arr[0] = Some(&input_buffer);
                arr
            };
            node.process(&inputs, &mut output_buffer, &mut params, 48000.0);
            prop_assert_eq!(output_buffer, input_buffer);
        }

        // Property 4
        /// **Validates: Requirements 2.2**
        ///
        /// Property 4: RecordNode ring buffer round-trip.
        ///
        /// For any input buffer passed to `RecordNode::process()` when the ring buffer
        /// has capacity, draining the ring buffer SHALL yield the same 64 samples.
        #[test]
        fn prop_record_node_ring_buffer_round_trip(
            input_samples in audio_buffer(),
        ) {
            let ring = HeapRb::<f32>::new(BUFFER_SIZE * 2);
            let (producer, mut consumer) = ring.split();
            let mut node = RecordNode::new(producer);
            let input_buffer: [f32; BUFFER_SIZE] = input_samples;
            let mut output_buffer = [0.0f32; BUFFER_SIZE];
            let mut params = ParamBlock::default();
            let inputs: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] = {
                let mut arr: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] = [None; MAX_INPUTS];
                arr[0] = Some(&input_buffer);
                arr
            };
            node.process(&inputs, &mut output_buffer, &mut params, 48000.0);
            let mut drained_samples = [0.0f32; BUFFER_SIZE];
            let drained_count = consumer.pop_slice(&mut drained_samples);
            prop_assert_eq!(drained_count, BUFFER_SIZE);
            prop_assert_eq!(drained_samples, input_buffer);
        }
    }
}
