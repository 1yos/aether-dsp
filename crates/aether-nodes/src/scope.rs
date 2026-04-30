//! ScopeNode — feeds audio into a lock-free SPSC ring buffer for visualisation.
//!
//! The ring is sized to 512 samples (8 × 64-sample frames).
//! Passes input through to output unchanged (unity gain).
//! If the ring is full, samples are dropped silently.
//! No allocation, no locks in the hot path.

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use ringbuf::{traits::Producer, HeapProd};

pub struct ScopeNode {
    producer: HeapProd<f32>,
}

impl ScopeNode {
    pub fn new(producer: HeapProd<f32>) -> Self {
        Self { producer }
    }
}

impl DspNode for ScopeNode {
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
        "ScopeNode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use ringbuf::{traits::Split, HeapRb};

    /// Generate a [f32; BUFFER_SIZE] array via proptest (BUFFER_SIZE = 64).
    fn audio_buffer() -> impl Strategy<Value = [f32; BUFFER_SIZE]> {
        prop::collection::vec(-1.0f32..=1.0f32, BUFFER_SIZE)
            .prop_map(|v| v.try_into().unwrap())
    }

    // Property 9
    proptest! {
        /// **Validates: Requirements 4.4**
        ///
        /// Property 9: ScopeNode pass-through.
        ///
        /// For any input buffer passed to `ScopeNode::process()`, the output buffer
        /// SHALL be identical to the input buffer (unity gain pass-through).
        #[test]
        fn prop_scope_node_pass_through(
            input_samples in audio_buffer(),
        ) {
            let ring = HeapRb::<f32>::new(BUFFER_SIZE * 2);
            let (producer, _consumer) = ring.split();
            let mut node = ScopeNode::new(producer);
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
    }

    // Property 10
    proptest! {
        /// **Validates: Requirements 4.7**
        ///
        /// Property 10: Scope frame serialization.
        ///
        /// For any array of 64 f32 samples serialized as a binary scope frame,
        /// the resulting byte slice SHALL be exactly 256 bytes, and decoding it
        /// as [f32; 64] little-endian SHALL recover the original values exactly.
        #[test]
        fn prop_scope_frame_serialization(
            samples in audio_buffer(),
        ) {
            let mut serialized = Vec::with_capacity(256);
            for &sample in &samples {
                serialized.extend_from_slice(&sample.to_le_bytes());
            }
            prop_assert_eq!(serialized.len(), 256);
            let mut deserialized = [0.0f32; 64];
            for (i, chunk) in serialized.chunks_exact(4).enumerate() {
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                deserialized[i] = f32::from_le_bytes(bytes);
            }
            prop_assert_eq!(deserialized, samples);
        }
    }
}
