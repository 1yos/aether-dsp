# aether-nodes

[![crates.io](https://img.shields.io/crates/v/aetherdsp-nodes.svg)](https://crates.io/crates/aetherdsp-nodes)
[![docs.rs](https://docs.rs/aetherdsp-nodes/badge.svg)](https://docs.rs/aetherdsp-nodes)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

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

## Examples

See the [examples](examples/) directory for complete working examples:

- `filter_sweep.rs` - Animate filter cutoff frequency
- `envelope_test.rs` - Trigger ADSR envelope
- `reverb_demo.rs` - Reverb with different room sizes

Run with:

```bash
cargo run --example filter_sweep -p aetherdsp-nodes
```

## License

MIT — see [LICENSE](../../LICENSE)
