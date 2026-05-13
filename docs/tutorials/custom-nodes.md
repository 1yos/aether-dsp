# Tutorial: Creating Custom DSP Nodes

**Level:** Intermediate  
**Time:** 20-30 minutes  
**Prerequisites:** Completed [First Synth Tutorial](first-synth.md)

In this tutorial, you'll create custom DSP nodes using the Node Development Kit (NDK):

- Tremolo effect (amplitude modulation)
- Distortion effect (waveshaping)
- Custom filter (simple lowpass)

---

## Table of Contents

1. [Setup](#setup)
2. [Simple Tremolo Effect](#simple-tremolo-effect)
3. [Distortion Effect](#distortion-effect)
4. [Custom Filter](#custom-filter)
5. [Testing Your Nodes](#testing-your-nodes)
6. [Publishing](#publishing)

---

## Setup

### Step 1: Create a New Project

```bash
cargo new custom-dsp-nodes --lib
cd custom-dsp-nodes
```

### Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aether-ndk = "0.1"
aether-ndk-macro = "0.1"

[dev-dependencies]
aetherdsp-nodes = "0.2.3"
```

---

## Simple Tremolo Effect

Tremolo modulates amplitude to create a "wobbling" effect.

### Step 1: Create the Node

Edit `src/lib.rs`:

```rust
use aether_ndk::prelude::*;

/// Tremolo effect - amplitude modulation
#[aether_node]
pub struct Tremolo {
    /// Modulation rate in Hz
    #[param(name = "Rate", min = 0.1, max = 20.0, default = 4.0)]
    rate: f32,

    /// Modulation depth (0.0 = no effect, 1.0 = full)
    #[param(name = "Depth", min = 0.0, max = 1.0, default = 0.5)]
    depth: f32,

    // Internal state
    phase: f32,
}

impl DspProcess for Tremolo {
    fn process(&mut self, inputs: &[Option<&[f32]>], output: &mut [f32], sample_rate: f32) {
        // Get input (or silence)
        let input = inputs[0].unwrap_or(&[]);

        // Phase increment per sample
        let phase_inc = self.rate / sample_rate;

        for i in 0..output.len() {
            // Generate LFO (sine wave)
            let lfo = (self.phase * std::f32::consts::TAU).sin();

            // Map LFO from [-1, 1] to modulation range
            let mod_amount = 1.0 - (self.depth * (1.0 - lfo) * 0.5);

            // Apply modulation
            output[i] = if i < input.len() {
                input[i] * mod_amount
            } else {
                0.0
            };

            // Advance phase
            self.phase = (self.phase + phase_inc).fract();
        }
    }
}
```

### Step 2: Test It

Create `examples/tremolo_test.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_nodes::oscillator::Oscillator;
use custom_dsp_nodes::Tremolo;

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    // Add oscillator
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();

    // Add tremolo
    let tremolo = Box::new(Tremolo::new());
    let tremolo_id = sched.graph.add_node(tremolo).unwrap();

    // Connect: Oscillator → Tremolo
    sched.graph.connect(osc_id, tremolo_id, 0);
    sched.graph.set_output_node(tremolo_id);

    // Render to WAV
    use hound::{WavWriter, WavSpec};

    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("tremolo_test.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    // Render 3 seconds
    for _ in 0..(48_000 * 3 / 64) {
        sched.process_block_simple(&mut output);

        for &sample in output.iter().take(64) {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude).unwrap();
        }
    }

    writer.finalize().unwrap();
    println!("Rendered tremolo_test.wav");
}
```

Add to `Cargo.toml`:

```toml
[dev-dependencies]
aetherdsp-nodes = "0.2.3"
hound = "3"
```

Run it:

```bash
cargo run --example tremolo_test
```

**Listen to `tremolo_test.wav`** - you should hear the amplitude wobbling!

---

## Distortion Effect

Distortion adds harmonics by clipping or shaping the waveform.

### Step 1: Create the Node

Add to `src/lib.rs`:

```rust
/// Distortion effect - waveshaping
#[aether_node]
pub struct Distortion {
    /// Drive amount (1.0 = clean, 10.0 = heavy distortion)
    #[param(name = "Drive", min = 1.0, max = 10.0, default = 2.0)]
    drive: f32,

    /// Output level compensation
    #[param(name = "Level", min = 0.0, max = 1.0, default = 0.5)]
    level: f32,
}

impl DspProcess for Distortion {
    fn process(&mut self, inputs: &[Option<&[f32]>], output: &mut [f32], _sample_rate: f32) {
        let input = inputs[0].unwrap_or(&[]);

        for i in 0..output.len() {
            if i < input.len() {
                // Apply drive
                let driven = input[i] * self.drive;

                // Soft clipping (tanh)
                let clipped = driven.tanh();

                // Compensate output level
                output[i] = clipped * self.level;
            } else {
                output[i] = 0.0;
            }
        }
    }
}
```

### Step 2: Test It

Create `examples/distortion_test.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_nodes::oscillator::Oscillator;
use custom_dsp_nodes::Distortion;

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    // Add oscillator
    let osc = Box::new(Oscillator::new(220.0)); // Lower frequency shows distortion better
    let osc_id = sched.graph.add_node(osc).unwrap();

    // Add distortion
    let mut distortion = Distortion::new();
    distortion.drive = 5.0; // Heavy distortion
    let dist_id = sched.graph.add_node(Box::new(distortion)).unwrap();

    // Connect
    sched.graph.connect(osc_id, dist_id, 0);
    sched.graph.set_output_node(dist_id);

    // Render to WAV
    use hound::{WavWriter, WavSpec};

    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("distortion_test.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    // Render 2 seconds
    for _ in 0..(48_000 * 2 / 64) {
        sched.process_block_simple(&mut output);

        for &sample in output.iter().take(64) {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude).unwrap();
        }
    }

    writer.finalize().unwrap();
    println!("Rendered distortion_test.wav");
}
```

Run it:

```bash
cargo run --example distortion_test
```

**Listen to `distortion_test.wav`** - you should hear rich harmonics!

---

## Custom Filter

Let's create a simple one-pole lowpass filter.

### Step 1: Create the Node

Add to `src/lib.rs`:

```rust
/// Simple one-pole lowpass filter
#[aether_node]
pub struct SimpleFilter {
    /// Cutoff frequency in Hz
    #[param(name = "Cutoff", min = 20.0, max = 20000.0, default = 1000.0)]
    cutoff: f32,

    // Internal state
    z1: f32,
}

impl DspProcess for SimpleFilter {
    fn process(&mut self, inputs: &[Option<&[f32]>], output: &mut [f32], sample_rate: f32) {
        let input = inputs[0].unwrap_or(&[]);

        // Calculate filter coefficient
        let omega = std::f32::consts::TAU * self.cutoff / sample_rate;
        let alpha = omega / (omega + 1.0);

        for i in 0..output.len() {
            let x = if i < input.len() { input[i] } else { 0.0 };

            // One-pole lowpass: y[n] = alpha * x[n] + (1 - alpha) * y[n-1]
            self.z1 = alpha * x + (1.0 - alpha) * self.z1;
            output[i] = self.z1;
        }
    }
}
```

### Step 2: Test It

Create `examples/filter_test.rs`:

```rust
use aetherdsp_core::scheduler::Scheduler;
use aetherdsp_nodes::oscillator::Oscillator;
use custom_dsp_nodes::SimpleFilter;

fn main() {
    let mut sched = Scheduler::new(48_000.0);

    // Add oscillator (sawtooth has lots of harmonics)
    let osc = Box::new(Oscillator::new(440.0));
    let osc_id = sched.graph.add_node(osc).unwrap();

    // Add filter
    let mut filter = SimpleFilter::new();
    filter.cutoff = 800.0; // Low cutoff to hear the effect
    let filter_id = sched.graph.add_node(Box::new(filter)).unwrap();

    // Connect
    sched.graph.connect(osc_id, filter_id, 0);
    sched.graph.set_output_node(filter_id);

    // Render to WAV
    use hound::{WavWriter, WavSpec};

    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("filter_test.wav", spec).unwrap();
    let mut output = vec![0.0f32; 128];

    // Render 2 seconds
    for _ in 0..(48_000 * 2 / 64) {
        sched.process_block_simple(&mut output);

        for &sample in output.iter().take(64) {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude).unwrap();
        }
    }

    writer.finalize().unwrap();
    println!("Rendered filter_test.wav");
}
```

Run it:

```bash
cargo run --example filter_test
```

**Listen to `filter_test.wav`** - you should hear a muffled tone!

---

## Testing Your Nodes

### Unit Tests

Add to `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tremolo_creates() {
        let tremolo = Tremolo::new();
        assert_eq!(tremolo.rate, 4.0);
        assert_eq!(tremolo.depth, 0.5);
    }

    #[test]
    fn test_tremolo_processes() {
        let mut tremolo = Tremolo::new();
        let input = vec![1.0f32; 64];
        let mut output = vec![0.0f32; 64];

        tremolo.process(&[Some(&input)], &mut output, 48_000.0);

        // Output should be modulated (not all 1.0)
        assert!(output.iter().any(|&x| x != 1.0));
    }

    #[test]
    fn test_distortion_clips() {
        let mut dist = Distortion::new();
        dist.drive = 10.0;

        let input = vec![1.0f32; 64];
        let mut output = vec![0.0f32; 64];

        dist.process(&[Some(&input)], &mut output, 48_000.0);

        // Output should be clipped (< 1.0 due to tanh)
        assert!(output.iter().all(|&x| x.abs() < 1.0));
    }
}
```

Run tests:

```bash
cargo test
```

### Property Tests

Add to `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1"
```

Add to `src/lib.rs`:

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn tremolo_never_explodes(rate in 0.1f32..20.0, depth in 0.0f32..1.0) {
            let mut tremolo = Tremolo::new();
            tremolo.rate = rate;
            tremolo.depth = depth;

            let input = vec![0.5f32; 64];
            let mut output = vec![0.0f32; 64];

            tremolo.process(&[Some(&input)], &mut output, 48_000.0);

            // Output should never be NaN or infinite
            assert!(output.iter().all(|&x| x.is_finite()));
        }
    }
}
```

---

## Publishing

### Step 1: Add Metadata

Edit `Cargo.toml`:

```toml
[package]
name = "custom-dsp-nodes"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT"
description = "Custom DSP nodes for AetherDSP"
repository = "https://github.com/yourusername/custom-dsp-nodes"
keywords = ["audio", "dsp", "effects"]
categories = ["multimedia::audio"]
```

### Step 2: Add README

Create `README.md`:

```markdown
# Custom DSP Nodes

Custom effects for AetherDSP:

- Tremolo - Amplitude modulation
- Distortion - Waveshaping
- SimpleFilter - One-pole lowpass

## Usage

\`\`\`rust
use custom_dsp_nodes::Tremolo;

let tremolo = Box::new(Tremolo::new());
\`\`\`

## License

MIT
```

### Step 3: Publish

```bash
cargo publish
```

---

## Next Steps

### Advanced Techniques

1. **State Variables** - Add more internal state for complex effects
2. **Multi-Input Nodes** - Process multiple inputs
3. **Parameter Modulation** - Accept modulation inputs
4. **Oversampling** - Reduce aliasing in nonlinear effects
5. **SIMD Optimization** - Use SIMD for faster processing

### Example: Multi-Input Mixer

```rust
#[aether_node]
pub struct Mixer {
    #[param(name = "Gain 1", min = 0.0, max = 2.0, default = 1.0)]
    gain1: f32,

    #[param(name = "Gain 2", min = 0.0, max = 2.0, default = 1.0)]
    gain2: f32,
}

impl DspProcess for Mixer {
    fn process(&mut self, inputs: &[Option<&[f32]>], output: &mut [f32], _sample_rate: f32) {
        let input1 = inputs.get(0).and_then(|&x| x).unwrap_or(&[]);
        let input2 = inputs.get(1).and_then(|&x| x).unwrap_or(&[]);

        for i in 0..output.len() {
            let s1 = if i < input1.len() { input1[i] * self.gain1 } else { 0.0 };
            let s2 = if i < input2.len() { input2[i] * self.gain2 } else { 0.0 };
            output[i] = s1 + s2;
        }
    }
}
```

---

## Resources

- [NDK Guide](../../docs/sdk/NDK_GUIDE.md) - Complete NDK documentation
- [AetherDSP Documentation](https://docs.rs/aetherdsp-core)
- [DSP Guide](https://www.dspguide.com/) - Learn DSP theory
- [Next Tutorial: Tuning Systems](tuning-systems.md)

---

**Congratulations!** You've created custom DSP nodes! 🎉
