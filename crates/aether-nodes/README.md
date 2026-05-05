# aether-nodes

[![crates.io](https://img.shields.io/crates/v/aether-nodes.svg)](https://crates.io/crates/aether-nodes)
[![docs.rs](https://docs.rs/aether-nodes/badge.svg)](https://docs.rs/aether-nodes)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Built-in DSP nodes for the [AetherDSP](https://crates.io/crates/aetherdsp-core) engine.

## Nodes

| Node                  | Description                                                                |
| --------------------- | -------------------------------------------------------------------------- |
| `OscillatorNode`      | BLEP anti-aliased sawtooth/square/triangle/sine, tuning table support      |
| `StateVariableFilter` | LP/HP/BP/Notch simultaneously (Cytomic SVF)                                |
| `MoogLadder`          | Huovilainen model, self-oscillation, audio-rate modulation                 |
| `FormantFilter`       | Vowel shaping A/E/I/O/U morph — essential for wind instruments             |
| `AdsrEnvelope`        | Sample-accurate ADSR with gate                                             |
| `Lfo`                 | 5 waveforms: sine, triangle, square, S&H, random-smooth                    |
| `Reverb`              | Freeverb (8 comb + 4 allpass filters)                                      |
| `DelayLine`           | Feedback delay with tempo sync                                             |
| `KarplusStrong`       | Physically accurate plucked string synthesis                               |
| `Granular`            | Grain size, density, pitch scatter, position — world music textures        |
| `Compressor`          | RMS-based dynamic range compression with soft-knee curve                   |
| `Waveshaper`          | 5 distortion modes: tanh, hard-clip, fold-back, bit-crush, tube saturation |
| `Chorus`              | BBD-style modulated delay for thickening and stereo widening               |
| `GainNode`            | Smoothed gain/volume control                                               |
| `MixerNode`           | N-input summing mixer with per-channel gain · SIMD FMA-optimized           |
| `ScopeNode`           | Oscilloscope — writes to a shared ring buffer for UI                       |
| `RecordNode`          | Captures audio to a WAV file via `hound`                                   |

## Usage

```rust
use aether_nodes::oscillator::OscillatorNode;
use aether_nodes::filter::FilterNode;
use aether_nodes::compressor::CompressorNode;
use aether_nodes::chorus::ChorusNode;
use aether_core::node::DspNode;

let osc = OscillatorNode::default();    // 440 Hz sine
let filt = FilterNode::default();       // LP @ 1 kHz
let comp = CompressorNode::default();   // -12 dB threshold, 4:1 ratio
let chorus = ChorusNode::default();     // 0.5 Hz rate, 7 ms depth
```

All nodes implement `aether_core::node::DspNode` and are safe to use in the RT thread — no allocation, no locks, no I/O.

## RT Safety

Every node in this crate follows the AetherDSP real-time rules:

- No heap allocation during `process()`
- No mutex or lock usage
- No I/O or system calls
- Bounded execution time per block

## License

MIT — see [LICENSE](../../LICENSE)
