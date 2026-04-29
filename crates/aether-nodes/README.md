# aether-nodes

[![crates.io](https://img.shields.io/crates/v/aether-nodes.svg)](https://crates.io/crates/aether-nodes)
[![docs.rs](https://docs.rs/aether-nodes/badge.svg)](https://docs.rs/aether-nodes)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Built-in DSP nodes for the [AetherDSP](https://crates.io/crates/aether-core) engine.

## Nodes

| Node             | Description                                               |
| ---------------- | --------------------------------------------------------- |
| `OscillatorNode` | Wavetable oscillator — sine, saw, square, triangle, noise |
| `FilterNode`     | State-variable filter (SVF) — LP, HP, BP, notch           |
| `EnvelopeNode`   | ADSR envelope generator with per-sample smoothing         |
| `DelayNode`      | Tempo-syncable delay line with feedback                   |
| `GainNode`       | Smoothed gain/volume control                              |
| `MixerNode`      | N-input summing mixer with per-channel gain               |
| `ScopeNode`      | Oscilloscope — writes to a shared ring buffer for UI      |
| `RecordNode`     | Captures audio to a WAV file via `hound`                  |

## Usage

```rust
use aether_nodes::oscillator::OscillatorNode;
use aether_nodes::filter::FilterNode;
use aether_core::node::DspNode;

let osc = OscillatorNode::default();   // 440 Hz sine
let filt = FilterNode::default();      // LP @ 1 kHz
```

All nodes implement `aether_core::node::DspNode` and are safe to use in the RT thread.

## License

MIT — see [LICENSE](../../LICENSE)
