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
| [`aetherdsp-core`](https://crates.io/crates/aetherdsp-core)           | 0.1.1   | RT scheduler, generational arena, lock-free graph, buffer pool                                           |
| [`aetherdsp-nodes`](https://crates.io/crates/aetherdsp-nodes)         | 0.2.0   | 15 DSP nodes: oscillator, filters, reverb, LFO, granular, Karplus-Strong, compressor, waveshaper, chorus |
| [`aetherdsp-ndk`](https://crates.io/crates/aetherdsp-ndk)             | 0.1.1   | Node Development Kit — build custom nodes with `#[aether_node]`                                          |
| [`aetherdsp-ndk-macro`](https://crates.io/crates/aetherdsp-ndk-macro) | 0.1.1   | Proc-macro behind the NDK                                                                                |
| [`aetherdsp-midi`](https://crates.io/crates/aetherdsp-midi)           | 0.1.1   | MIDI engine with 8 tuning systems including Ethiopian, Arabic, Gamelan                                   |
| [`aetherdsp-sampler`](https://crates.io/crates/aetherdsp-sampler)     | 0.2.0   | Polyphonic sampler with ArcSwap lock-free instrument loading                                             |
| [`aetherdsp-timbre`](https://crates.io/crates/aetherdsp-timbre)       | 0.1.1   | FFT-based spectral timbre analysis and transfer                                                          |
| [`aetherdsp-manifest`](https://crates.io/crates/aetherdsp-manifest)   | 0.1.1   | Node package manifest format                                                                             |
| [`aetherdsp-registry`](https://crates.io/crates/aetherdsp-registry)   | 0.1.1   | Runtime node type registry                                                                               |
| `aether-samples`                                                      | 0.1.1   | On-demand sample pack download and management                                                            |

### The Studio (Aether Studio v0.2)

A 4-mode music production interface built with React + Tauri:

- **Explore** — World map with 57 instruments from 8 regions. Click a region, browse its instruments, try them with your PC keyboard before adding to your project.
- **Create** — Modular node graph with a custom WebGL renderer. Connect oscillators, filters, reverb, LFO, granular synthesis, compressor, waveshaper, chorus, and more. Wire modulations between any node's output and any parameter via the Modulation Matrix.
- **Arrange** — Non-Western piano roll with 13 rhythmic systems (Teentaal, Maqsum, Gamelan Lancaran, and more) and 14 scale systems (Ethiopian Tizita/Bati/Ambassel, Arabic Maqam Rast/Bayati/Hijaz, Indian Raga Yaman/Bhairav, Gamelan Slendro/Pelog). Beat name headers, accent highlighting, and microtonal cent-deviation bars on every key.
- **Perform** — Clip launcher for live performance.

**Instrument presets:** 57 `.aether-instrument` files ship with the studio — one per instrument. Each loads a complete node graph with correct tuning, envelope, and effects for that instrument.

**Instrument Recorder:** Record your own instrument samples directly from a microphone. The recorder prompts note-by-note, auto-detects pitch via autocorrelation, and exports a `.aether-instrument` file ready to load into any SamplerNode.

**Patch sharing:** Share your node graph with anyone via a single URL. Patches are uploaded as GitHub Gists and auto-loaded from the `?patch=` URL parameter.

**Plugin export:** Builds as both a `.clap` and `.vst3` plugin for use in any compatible DAW.

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
| `ScopeNode`           | Oscilloscope output                                                                              |
| `RecordNode`          | Lock-free WAV recording via SPSC ring buffer                                                     |

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

| System              | Description                                   |
| ------------------- | --------------------------------------------- |
| 12-TET              | Standard equal temperament                    |
| Ethiopian Tizita    | Pentatonic with characteristic flat intervals |
| Ethiopian Bati      | Minor pentatonic variant                      |
| Ethiopian Ambassel  | Pentatonic with raised 4th                    |
| Arabic Maqam Rast   | Quarter-tone flats on 3rd and 7th             |
| Arabic Maqam Bayati | Half-flat on 2nd degree                       |
| Arabic Maqam Hijaz  | Augmented 2nd between 2nd and 3rd degrees     |
| Indian Raga Yaman   | Just intonation, raised 4th (Kalyan thaat)    |
| Indian Raga Bhairav | Flat 2nd and flat 6th                         |
| Gamelan Slendro     | 5-tone Javanese scale                         |
| Gamelan Pelog       | 7-tone Javanese scale with unequal intervals  |
| Just Intonation     | Pure harmonic ratios                          |
| Western Pentatonic  | 5-tone major pentatonic                       |
| Chromatic           | All 12 semitones                              |

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

### Use the engine in your project

```toml
[dependencies]
aetherdsp-core = "0.1"
aetherdsp-nodes = "0.2"
```

### Build a custom DSP node

```toml
[dependencies]
aetherdsp-ndk = "0.1"
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

### Development mode

```powershell
# Terminal 1 — audio engine (Windows: use C:\aether-dsp to avoid path spaces)
$env:PATH = "C:\msys64\mingw64\bin;" + $env:PATH
Set-Location C:\aether-dsp
cargo run -p aether-host --release

# Terminal 2 — UI
Set-Location C:\aether-dsp\ui
npm run dev
# Open http://localhost:5173
```

### Standalone app (Tauri)

```powershell
# Dev window (native desktop app with hot reload)
Set-Location C:\aether-dsp\ui
npm run tauri dev

# Production installer
Set-Location C:\aether-dsp
.\scripts\build_tauri.ps1
# Output: ui\src-tauri\target\release\bundle\
```

### Prerequisites

| Tool              | Version | Notes                                     |
| ----------------- | ------- | ----------------------------------------- |
| Rust              | 1.78+   | `stable-x86_64-pc-windows-gnu` on Windows |
| MSYS2 MinGW64 GCC | 13+     | Required linker on Windows                |
| Node.js           | 20+ LTS | For the UI                                |

**Windows setup:**

```powershell
winget install Rustlang.Rustup
rustup default stable-x86_64-pc-windows-gnu
winget install MSYS2.MSYS2
C:\msys64\usr\bin\bash.exe -lc "pacman -S --noconfirm mingw-w64-x86_64-gcc"
# Create junction to avoid spaces in path
New-Item -ItemType Junction -Path "C:\aether-dsp" -Target "D:\path\to\aether-dsp"
```

---

## Project Structure

```
aether-dsp/
├── crates/
│   ├── aether-core/        # RT engine: arena, graph, scheduler, params
│   ├── aether-nodes/       # 15 DSP nodes including compressor, waveshaper, chorus
│   ├── aether-midi/        # MIDI engine + 9 tuning systems
│   ├── aether-sampler/     # Polyphonic sampler with ArcSwap
│   ├── aether-timbre/      # Spectral timbre analysis and transfer
│   ├── aether-samples/     # Sample pack download and management
│   ├── aether-ndk/         # Node Development Kit
│   ├── aether-ndk-macro/   # #[aether_node] proc-macro
│   ├── aether-manifest/    # Node package manifest format
│   ├── aether-registry/    # Runtime node registry
│   ├── aether-host/        # CPAL audio host + WebSocket bridge (not published)
│   ├── aether-plugin/      # CLAP + VST3 plugin (not published)
│   └── aether-cli/         # Developer CLI (not published)
├── ui/                     # React + Tauri studio interface
│   ├── src/
│   │   ├── modes/          # Explore, Create, Arrange, Perform
│   │   ├── components/     # TopBar, KeyboardPlayer, ModulationMatrix, InstrumentRecorder
│   │   ├── catalog/        # 57-instrument world music catalog
│   │   ├── hooks/          # usePatchShare, useSampleLibrary, useInstrumentEngine
│   │   └── studio/         # WebGL node graph, engine store, WebSocket
│   └── src-tauri/          # Tauri standalone app wrapper
├── assets/
│   ├── instruments/        # Drum kit and instrument definitions
│   └── presets/            # World music presets (Krar, etc.)
├── scripts/                # Build and publish scripts
└── docs/                   # Architecture, SDK guides, design docs
```

---

## WebSocket Protocol

Connect to `ws://127.0.0.1:9001` (started by `aether-host`).

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
- Benchmark regression check
- TypeScript check + Vite build
- Tauri standalone app build (Linux, produces `.deb`, `.rpm`, `.AppImage`)

---

## Roadmap

| Version | Milestone                                                                                                                                   |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| v0.1    | RT engine + UI + WebSocket + 9 crates on crates.io                                                                                          |
| v0.2    | 15 DSP nodes, modulation matrix, WebGL canvas, non-Western piano roll, VST3/CLAP export, sample library, patch sharing, instrument recorder |
| v0.3    | SIMD optimization, AI timbre transfer, mobile support                                                                                       |
| v1.0    | Stable public release                                                                                                                       |

---

## License

MIT — see [LICENSE](LICENSE)

GitHub: [github.com/1yos/aether-dsp](https://github.com/1yos/aether-dsp)
