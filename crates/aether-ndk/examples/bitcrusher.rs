//! Example: BitCrusher node — quantization + sample-rate reduction
//!
//! Run with: cargo run --example bitcrusher -p aether-ndk

use aether_ndk::prelude::*;

#[aether_node]
pub struct BitCrusher {
    #[param(name = "Bits",       min = 1.0, max = 16.0, default = 8.0)]
    bits: f32,
    #[param(name = "Rate Crush", min = 1.0, max = 32.0, default = 1.0)]
    rate_crush: f32,
    // Internal state
    held_sample: f32,
    counter: f32,
}

impl DspProcess for BitCrusher {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let input = inputs.get(0);
        for (i, out) in output.iter_mut().enumerate() {
            let bits  = params.get(0).current.clamp(1.0, 16.0);
            let crush = params.get(1).current.clamp(1.0, 32.0);

            // Sample-rate reduction
            self.counter += 1.0;
            if self.counter >= crush {
                self.counter = 0.0;
                // Bit quantization
                let levels = (2.0f32).powf(bits);
                self.held_sample = (input[i] * levels).round() / levels;
            }
            *out = self.held_sample;
            params.tick_all();
        }
    }

    fn capture_state(&self) -> StateBlob {
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct S { held: f32, counter: f32 }
        StateBlob::from_value(&S { held: self.held_sample, counter: self.counter })
    }

    fn restore_state(&mut self, state: StateBlob) {
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct S { held: f32, counter: f32 }
        let s: S = state.to_value();
        self.held_sample = s.held;
        self.counter = s.counter;
    }
}

fn main() {
    println!("Node: {}", BitCrusher::type_name());
    for def in BitCrusher::param_defs() {
        println!("  {} [{:.0}–{:.0}] default={:.0}", def.name, def.min, def.max, def.default);
    }
    println!("\nBitCrusher ready ✓");
}
