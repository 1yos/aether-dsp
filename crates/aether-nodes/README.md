# aether-nodes

[![crates.io](https://img.shields.io/crates/v/aetherdsp-nodes.svg)](https://crates.io/crates/aetherdsp-nodes)
[![docs.rs](https://docs.rs/aetherdsp-nodes/badge.svg)](https://docs.rs/aetherdsp-nodes)
[![CI](https://github.com/1yos/aether-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/1yos/aether-dsp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Downloads](https://img.shields.io/crates/d/aetherdsp-nodes.svg)](https://crates.io/crates/aetherdsp-nodes)

Built-in DSP nodes for AetherDSP — oscillators, filters, effects, and more.

## Available Nodes

| Node             | Description                                  |
| ---------------- | -------------------------------------------- |
| `oscillator`     | Sine, saw, square, triangle waveforms        |
| `filter`         | Biquad filters (lowpass, highpass, bandpass) |
| `moog-ladder`    | Classic Moog ladder filter                   |
| `reverb`         | Schroeder reverb with configurable room size |
| `delay`          | Delay line with feedback                     |
| `compressor`     | Dynamic range compressor                     |
| `envelope`       | ADSR envelope generator                      |
| `lfo`            | Low-frequency oscillator                     |
| `gain`           | Simple gain/volume control                   |
| `mixer`          | Multi-channel mixer                          |
| `formant`        | Formant filter for vocal synthesis           |
| `granular`       | Granular synthesis engine                    |
| `karplus-strong` | Physical modeling string synthesis           |
| `waveshaper`     | Waveshaping distortion                       |
| `chorus`         | Chorus effect                                |
| `record`         | Record audio to buffer                       |
| `scope`          | Oscilloscope for visualization               |

## Feature Flags

All nodes are enabled by default. You can opt-in to specific nodes to reduce compile times and binary size:

```toml
[dependencies]
aetherdsp-nodes = { version = "0.2", default-features = false, features = ["oscillator", "filter"] }
```

| Feature          | Default | Description              |
| ---------------- | ------- | ------------------------ |
| `all-nodes`      | ✅      | Enable all nodes         |
| `oscillator`     | ✅      | Oscillator node          |
| `filter`         | ✅      | Biquad filter node       |
| `moog-ladder`    | ✅      | Moog ladder filter       |
| `reverb`         | ✅      | Reverb effect            |
| `delay`          | ✅      | Delay line               |
| `compressor`     | ✅      | Compressor               |
| `envelope`       | ✅      | ADSR envelope            |
| `lfo`            | ✅      | LFO                      |
| `gain`           | ✅      | Gain control             |
| `mixer`          | ✅      | Mixer                    |
| `formant`        | ✅      | Formant filter           |
| `granular`       | ✅      | Granular synthesis       |
| `karplus-strong` | ✅      | Karplus-Strong synthesis |
| `waveshaper`     | ✅      | Waveshaper               |
| `chorus`         | ✅      | Chorus effect            |
| `record`         | ✅      | Record node              |
| `scope`          | ✅      | Oscilloscope             |

**Examples:**

```toml
# Minimal synth (oscillator + filter + envelope)
aetherdsp-nodes = { version = "0.2", default-features = false, features = ["oscillator", "filter", "envelope"] }

# Effects only
aetherdsp-nodes = { version = "0.2", default-features = false, features = ["reverb", "delay", "chorus"] }

# All nodes (default)
aetherdsp-nodes = "0.2"
```

## Quick Start

```rust
use aether_core::scheduler::Scheduler;
use aether_nodes::oscillator::Oscillator;

let mut sched = Scheduler::new(48_000.0);
let osc = Box::new(Oscillator::new(440.0));
let id = sched.graph.add_node(osc).unwrap();
sched.graph.set_output_node(id);

// Process audio
let mut output = vec![0.0f32; 128];
sched.process_block_simple(&mut output);
```

## Node Details

### Oscillators

**`oscillator`** - Anti-aliased waveforms

- Sine, saw, square, triangle
- BLEP anti-aliasing for saw/square
- Frequency modulation support
- Phase reset

**`karplus-strong`** - Physical modeling

- Plucked string synthesis
- Damping control
- Stretch tuning
- Realistic string behavior

### Filters

**`filter`** - Biquad filters

- Lowpass, highpass, bandpass, notch
- Resonance control
- Audio-rate modulation
- Stable at high resonance

**`moog-ladder`** - Classic Moog filter

- 4-pole lowpass
- Self-oscillation
- Huovilainen model
- Warm analog sound

**`formant`** - Vowel filter

- A/E/I/O/U vowel shapes
- Morphing between vowels
- Essential for wind instruments
- Vocal synthesis

### Effects

**`reverb`** - Schroeder reverb

- 8 comb + 4 allpass filters
- Room size control
- Damping control
- Wet/dry mix

**`delay`** - Delay line

- Feedback control
- Tempo sync
- Modulation support
- Ping-pong mode

**`chorus`** - BBD-style chorus

- Modulated delay
- Stereo widening
- Depth/rate control
- Vintage sound

**`compressor`** - Dynamics processor

- RMS-based detection
- Soft-knee curve
- Attack/release
- Makeup gain

**`waveshaper`** - Distortion

- 5 modes: tanh, hard-clip, fold-back, bit-crush, tube
- Drive control
- Tone shaping
- Harmonic generation

### Modulation

**`envelope`** - ADSR envelope

- Attack, decay, sustain, release
- Gate triggering
- Sample-accurate
- Exponential curves

**`lfo`** - Low-frequency oscillator

- 5 waveforms: sine, triangle, square, S&H, random-smooth
- Tempo sync
- Phase control
- Unipolar/bipolar

### Utility

**`gain`** - Volume control

- Smoothed gain changes
- dB or linear
- No clicks/pops

**`mixer`** - Multi-channel mixer

- N-input summing
- Per-channel gain
- SIMD optimized
- FMA instructions

**`record`** - Audio capture

- Record to buffer
- WAV file export
- Circular buffer
- Trigger modes

**`scope`** - Oscilloscope

- Waveform visualization
- Ring buffer output
- Trigger modes
- UI integration

### Synthesis

**`granular`** - Granular synthesis

- Grain size/density control
- Pitch scatter
- Position control
- World music textures

## Common Patterns

### Basic Synthesizer

```rust
use aether_core::scheduler::Scheduler;
use aether_nodes::{oscillator::Oscillator, filter::Filter, envelope::Envelope};

let mut sched = Scheduler::new(48_000.0);

// Oscillator → Filter → Envelope → Output
let osc = sched.graph.add_node(Box::new(Oscillator::new(440.0))).unwrap();
let filt = sched.graph.add_node(Box::new(Filter::lowpass(1000.0, 0.7))).unwrap();
let env = sched.graph.add_node(Box::new(Envelope::new())).unwrap();

sched.graph.connect(osc, filt, 0);
sched.graph.connect(filt, env, 0);
sched.graph.set_output_node(env);
```

### Effects Chain

```rust
use aether_nodes::{reverb::Reverb, delay::Delay, chorus::Chorus};

// Input → Chorus → Delay → Reverb → Output
let chorus = sched.graph.add_node(Box::new(Chorus::new())).unwrap();
let delay = sched.graph.add_node(Box::new(Delay::new(0.5))).unwrap();
let reverb = sched.graph.add_node(Box::new(Reverb::new(0.8))).unwrap();

sched.graph.connect(input, chorus, 0);
sched.graph.connect(chorus, delay, 0);
sched.graph.connect(delay, reverb, 0);
sched.graph.set_output_node(reverb);
```

### Modulation Routing

```rust
use aether_nodes::{lfo::Lfo, oscillator::Oscillator};

// LFO → Oscillator frequency modulation
let lfo = sched.graph.add_node(Box::new(Lfo::new(5.0))).unwrap();
let osc = sched.graph.add_node(Box::new(Oscillator::new(440.0))).unwrap();

// Connect LFO to oscillator's frequency parameter
sched.graph.connect(lfo, osc, 1); // Slot 1 = frequency modulation
sched.graph.set_output_node(osc);
```

## Performance Tips

### Compile Time Optimization

Use feature flags to reduce compile times:

```toml
# Minimal synth (60% faster compile)
aetherdsp-nodes = {
    version = "0.2",
    default-features = false,
    features = ["oscillator", "filter", "envelope"]
}
```

### Runtime Optimization

1. **Reuse nodes** - Don't create/destroy nodes frequently
2. **Batch commands** - Send multiple commands at once
3. **Minimize connections** - Fewer connections = faster topological sort
4. **Use parallel execution** - Enable `parallel` feature in aetherdsp-core
5. **Profile first** - Use `cargo bench` to identify bottlenecks

### Memory Optimization

1. **Disable unused features** - Smaller binary
2. **Adjust MAX_NODES** - Reduce if you don't need 10,240 nodes
3. **Adjust BUFFER_SIZE** - Smaller buffers = less memory
4. **Use release builds** - `cargo build --release`

## Examples

See the [examples](examples/) directory for complete working examples:

- `filter_sweep.rs` - Animate filter cutoff frequency
- `envelope_test.rs` - Trigger ADSR envelope
- `reverb_demo.rs` - Reverb with different room sizes

Run with:

```bash
cargo run --example filter_sweep -p aetherdsp-nodes
```

## Resources

- **Documentation:** https://docs.rs/aetherdsp-nodes
- **Migration Guide:** [MIGRATION.md](MIGRATION.md)
- **Core Engine:** [aetherdsp-core](../aether-core)
- **Issues:** https://github.com/1yos/aether-dsp/issues

## License

MIT — see [LICENSE](../../LICENSE)
