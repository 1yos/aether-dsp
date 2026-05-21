# AetherDSP

[![Rust](https://img.shields.io/badge/Rust-1.78+-orange)](https://www.rust-lang.org)
[![CI](https://github.com/1yos/aether-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/1yos/aether-dsp/actions)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/aetherdsp-core.svg)](https://crates.io/crates/aetherdsp-core)
[![Benchmark](https://img.shields.io/badge/param%20fill-51.7%20ns-yellow)](#benchmarks)

**A hard real-time modular DSP engine and world music production studio.**

```
64-sample buffer · 48 kHz · ≤1.33 ms deadline · Zero allocations · Lock-free
```

AetherDSP is two things at once: a production-grade audio engine library for Rust developers, and a standalone music studio that celebrates instruments from around the world — Ethiopian Krar, West African Kora, Arabic Oud, Indian Sitar, Javanese Gamelan, and 52 more.

---

## What's Inside

### The Engine (Rust crates)

| Crate                                                                 | Version | Description                                                                                              |
| --------------------------------------------------------------------- | ------- | -------------------------------------------------------------------------------------------------------- |
| [`aetherdsp-core`](https://crates.io/crates/aetherdsp-core)           | 0.1.4   | RT scheduler, generational arena, lock-free graph, buffer pool                                           |
| [`aetherdsp-nodes`](https://crates.io/crates/aetherdsp-nodes)         | 0.2.3   | 17 DSP nodes: oscillator, filters, reverb, LFO, granular, Karplus-Strong, compressor, waveshaper, chorus |
| [`aetherdsp-ndk`](https://crates.io/crates/aetherdsp-ndk)             | 0.1.4   | Node Development Kit — build custom nodes with `#[aether_node]`                                          |
| [`aetherdsp-ndk-macro`](https://crates.io/crates/aetherdsp-ndk-macro) | 0.1.4   | Proc-macro behind the NDK                                                                                |
| [`aetherdsp-midi`](https://crates.io/crates/aetherdsp-midi)           | 0.1.4   | MIDI engine with 13 tuning systems including Ethiopian, Arabic, Gamelan                                  |
| [`aetherdsp-sampler`](https://crates.io/crates/aetherdsp-sampler)     | 0.2.1   | Polyphonic sampler with ArcSwap lock-free instrument loading                                             |
| [`aetherdsp-timbre`](https://crates.io/crates/aetherdsp-timbre)       | 0.1.4   | FFT-based spectral timbre analysis and transfer                                                          |
| [`aetherdsp-manifest`](https://crates.io/crates/aetherdsp-manifest)   | 0.1.4   | Node package manifest format                                                                             |
| [`aetherdsp-registry`](https://crates.io/crates/aetherdsp-registry)   | 0.1.4   | Runtime node type registry                                                                               |
| `aether-samples`                                                      | 0.1.4   | On-demand sample pack download and management                                                            |
| `aether-ui`                                                           | 0.1.4   | Native GPU-accelerated DAW (Iced + wgpu) — not published                                                 |

### The Studio (Aether Studio v0.3)

A professional DAW built with **Iced** (GPU-accelerated native UI using wgpu):

- **Song View** — Multi-track timeline with clip-based arrangement. Drag, resize, split, and color-code clips. Snap to grid with configurable quantization.
- **Piano Roll** — MIDI note editor with velocity lanes, quantization, transposition, and scale highlighting. Full keyboard input support.
- **Mixer** — Per-track volume, pan, mute, solo, and arm controls. VU meters with peak hold. Insert effects chain (EQ, Compressor, Reverb, Delay).
- **Transport** — Play, stop, record, loop, metronome with downbeat accent. Dual time display (bars:beats + mm:ss.cs). BPM control with tap tempo.

**Project Management:** Save/load complete projects with all tracks, clips, MIDI notes, mixer settings, and effect chains. Atomic file writes prevent corruption.

**Audio Export:** Offline WAV rendering (48kHz, 16-bit stereo) with progress tracking. Export entire project or loop region.

**Metronome:** DSP-based click generator with downbeat accent (1200Hz) and beat clicks (800Hz). Tempo-synced to project BPM.

**Cross-Platform:** Runs on Windows (MSVC), macOS, and Linux. GPU-accelerated rendering via wgpu (Metal/Vulkan/DirectX 12).

---

## DSP Nodes

| Node                  | Description                                                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------ |
| `Oscillator`          | BLEP anti-aliased sawtooth/square/triangle/sine, tuning table support · SIMD-optimized sine path |
| `StateVariableFilter` | LP/HP/BP/Notch simultaneously (Cytomic SVF)                                                      |
| `MoogLadder`          | Huovilainen model, self-oscillation, audio-rate modulation                                       |
| `FormantFilter`       | Vowel shaping A/E/I/O/U morph — essential for wind instruments                                   |
| `AdsrEnvelope`        | Sample-accurate ADSR with gate                                                                   |
| `Lfo`                 | 5 waveforms: sine, triangle, square, S&H, random-smooth                                          |
| `Reverb`              | Freeverb (8 comb + 4 allpass filters)                                                            |
| `DelayLine`           | Feedback delay with tempo sync                                                                   |
| `KarplusStrong`       | Physically accurate plucked string synthesis                                                     |
| `Granular`            | Grain size, density, pitch scatter, position — world music textures                              |
| `Compressor`          | RMS-based dynamic range compression with soft-knee curve                                         |
| `Waveshaper`          | 5 distortion modes: tanh, hard-clip, fold-back, bit-crush, tube saturation                       |
| `Chorus`              | BBD-style modulated delay for thickening and widening                                            |
| `Gain`                | Smoothed gain control                                                                            |
| `Mixer`               | N-input summing mixer · SIMD FMA-optimized accumulation                                          |
| `SamplerNode`         | Polyphonic sampler, MIDI-driven, ArcSwap lock-free, round-robin zones                            |
| `TimbreTransferNode`  | FFT spectral envelope transfer                                                                   |

---

## Modulation System

Any node output can modulate any parameter in the graph. The Modulation Matrix UI lets you create and remove connections visually. Under the hood, `ModConnection` structs are stored in `GraphManager` and applied each block before parameter smoothing runs.

```
LFO output → Filter cutoff frequency
Envelope output → Oscillator amplitude
Compressor gain reduction → Reverb wet level
```

---

## Tuning Systems

AetherDSP treats tuning as a first-class feature. Every instrument loads with its correct tuning system by default.

| System                      | Description                                   |
| --------------------------- | --------------------------------------------- |
| 12-TET                      | Standard equal temperament                    |
| Ethiopian Tizita            | Pentatonic with characteristic flat intervals |
| Ethiopian Bati              | Minor pentatonic variant                      |
| Ethiopian Ambassel          | Pentatonic with raised 4th                    |
| Arabic Maqam Rast           | Quarter-tone flats on 3rd and 7th             |
| Arabic Maqam Bayati         | Half-flat on 2nd degree                       |
| Arabic Maqam Hijaz          | Augmented 2nd between 2nd and 3rd degrees     |
| Indian Raga Yaman           | Just intonation, raised 4th (Kalyan thaat)    |
| Gamelan Slendro             | 5-tone Javanese scale                         |
| Gamelan Slendro (Stretched) | 5-tone with stretched octave (~1210 cents)    |
| Gamelan Pelog               | 7-tone Javanese scale with unequal intervals  |
| Just Intonation (5-limit)   | Pure harmonic ratios (traditional)            |
| Just Intonation (7-limit)   | Pure ratios with septimal intervals (blues)   |

**Total: 13 tuning systems** covering Ethiopian, Arabic, Indian, and Javanese musical traditions.

For detailed information about tuning system implementation, precision, and pitch-bend interaction, see the [Tuning Systems Tutorial](docs/tutorials/tuning-systems.md).

---

## Real-Time Guarantees

| Rule                            | Enforcement                                            |
| ------------------------------- | ------------------------------------------------------ |
| No heap allocation in RT thread | Pre-allocated arena + buffer pool                      |
| No locks in RT thread           | ArcSwap for instrument loading, SPSC ring for commands |
| No I/O in RT thread             | All I/O on control/tokio threads                       |
| Bounded execution               | Flat topo-sorted array, parallel BFS levels via Rayon  |
| No recursion                    | Iterative Kahn's sort, iterative execution             |

---

## Benchmarks

| Benchmark                   | Result      |
| --------------------------- | ----------- |
| `param_fill_buffer_64`      | **51.7 ns** |
| Arena insert/remove ×1000   | < 5 µs      |
| Scheduler (1000 noop nodes) | < 100 µs    |

---

## Quick Start

### Tutorials

New to AetherDSP? Start with our step-by-step tutorials:

- **[Building Your First Synthesizer](docs/tutorials/first-synth.md)** - Learn the basics (30-45 min)
- **[Creating Custom DSP Nodes](docs/tutorials/custom-nodes.md)** - Build your own effects (20-30 min)
- **[Microtonal Music with Tuning Systems](docs/tutorials/tuning-systems.md)** - Explore world music scales (20-30 min)

See the complete [Tutorial Index](docs/tutorials/README.md) for more.

### Use the engine in your project

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aetherdsp-nodes = "0.2.3"
```

### Build a custom DSP node

```toml
[dependencies]
aetherdsp-ndk = "0.1.4"
```

```rust
use aether_ndk::prelude::*;

#[aether_node]
pub struct Tremolo {
    #[param(name = "Rate",  min = 0.1, max = 20.0, default = 4.0)]
    rate: f32,
    #[param(name = "Depth", min = 0.0, max = 1.0,  default = 0.5)]
    depth: f32,
    phase: f32,
}

impl DspProcess for Tremolo {
    fn process(&mut self, inputs: &NodeInputs, output: &mut NodeOutput,
               params: &mut ParamBlock, sample_rate: f32) {
        let input = inputs.get(0);
        for (i, out) in output.iter_mut().enumerate() {
            let lfo = 1.0 - params.get(1).current * 0.5
                * (1.0 - (self.phase * std::f32::consts::TAU).cos());
            *out = input[i] * lfo;
            self.phase = (self.phase + params.get(0).current / sample_rate).fract();
            params.tick_all();
        }
    }
}
```

---

## Running Aether Studio

### Prerequisites

| Tool          | Version | Notes                                                                   |
| ------------- | ------- | ----------------------------------------------------------------------- |
| Rust          | 1.78+   | `stable-x86_64-pc-windows-msvc` (Windows) or default stable (Mac/Linux) |
| Visual Studio | 2022+   | Windows only: Build Tools with C++ workload                             |
| Xcode         | Latest  | macOS only: Command Line Tools                                          |

**Windows setup (MSVC toolchain recommended):**

```powershell
# Install Rust
winget install Rustlang.Rustup
rustup default stable-x86_64-pc-windows-msvc

# Install Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools
# During installation, select "Desktop development with C++"
```

**macOS setup:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Xcode Command Line Tools
xcode-select --install
```

**Linux setup:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libasound2-dev
```

### Build and Run

```bash
# Build the DAW
cargo build --release -p aether-ui

# Run the DAW
cargo run --release -p aether-ui
# Or directly:
./target/release/aether-studio
```

### Known Issues

**Windows MinGW:** The GUI application cannot be built with MinGW due to linker command-line length limitations. Use the MSVC toolchain instead (see prerequisites above).

For detailed build instructions and troubleshooting, see [`docs/build/BUILD_GUIDE.md`](docs/build/BUILD_GUIDE.md).

---

## Project Structure

```
aether-dsp/
├── crates/
│   ├── aether-core/        # RT engine: arena, graph, scheduler, params
│   ├── aether-nodes/       # 17 DSP nodes including compressor, waveshaper, chorus
│   ├── aether-midi/        # MIDI engine + 13 tuning systems
│   ├── aether-sampler/     # Polyphonic sampler with ArcSwap
│   ├── aether-timbre/      # Spectral timbre analysis and transfer
│   ├── aether-samples/     # Sample pack download and management
│   ├── aether-ndk/         # Node Development Kit
│   ├── aether-ndk-macro/   # #[aether_node] proc-macro
│   ├── aether-manifest/    # Node package manifest format
│   ├── aether-registry/    # Runtime node registry
│   ├── aether-ui/          # Native GPU DAW (Iced + wgpu) — not published
│   ├── aether-host/        # CPAL audio host + WebSocket bridge (not published)
│   ├── aether-plugin/      # CLAP + VST3 plugin (not published)
│   └── aether-cli/         # Developer CLI (not published)
├── assets/
│   ├── instruments/        # Drum kit and instrument definitions
│   └── presets/            # World music presets (Krar, etc.)
├── scripts/                # Build and publish scripts
└── docs/                   # Architecture, SDK guides, design docs
```

---

## WebSocket Protocol (aether-host)

The `aether-host` crate provides a WebSocket bridge for external control. Connect to `ws://127.0.0.1:9001`.

```json
{ "type": "add_node", "node_type": "Oscillator" }
{ "type": "connect", "src_id": 0, "dst_id": 1, "slot": 0 }
{ "type": "update_param", "node_id": 0, "generation": 0, "param_index": 0, "value": 880.0, "ramp_ms": 20 }
{ "type": "inject_midi", "channel": 0, "note": 60, "velocity": 90, "is_note_on": true }
{ "type": "load_instrument", "node_id": 2, "generation": 0, "instrument_json": "..." }
{ "type": "set_modulation", "src_node_id": 3, "dst_node_id": 1, "param_index": 0, "amount": 0.5 }
{ "type": "get_snapshot" }
```

---

## CI

Every push runs on **Windows, macOS, and Linux**:

- `cargo check --workspace`
- `cargo test --lib` (core crates)
- `cargo clippy -- -D warnings`
- Benchmark regression check (Linux, main branch only)

---

## Roadmap

| Version | Milestone                                                                                       |
| ------- | ----------------------------------------------------------------------------------------------- |
| v0.1    | RT engine + WebSocket bridge + 9 crates on crates.io ✅                                         |
| v0.2    | 17 DSP nodes, modulation matrix, sample library, VST3/CLAP export ✅                            |
| v0.3    | Native GPU DAW (Iced): Song view, Piano roll, Mixer, Transport, Save/Load, Export, Metronome ✅ |
| v0.4    | SIMD optimization, world music instruments, expanded tuning systems                             |
| v1.0    | Stable public release                                                                           |

---

## Author

**Yoseph Abebe** — Creator and Lead Developer

Based in Addis Ababa, Ethiopia

GitHub: [@1yos](https://github.com/1yos)

---

## License

MIT — see [LICENSE](LICENSE)

Repository: [github.com/1yos/aether-dsp](https://github.com/1yos/aether-dsp)
