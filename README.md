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

| Crate                                                                 | Version | Description                                                              |
| --------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------ |
| [`aetherdsp-core`](https://crates.io/crates/aetherdsp-core)           | 0.1.1   | RT scheduler, generational arena, lock-free graph, buffer pool           |
| [`aetherdsp-nodes`](https://crates.io/crates/aetherdsp-nodes)         | 0.2.0   | 12 DSP nodes: oscillator, filters, reverb, LFO, granular, Karplus-Strong |
| [`aetherdsp-ndk`](https://crates.io/crates/aetherdsp-ndk)             | 0.1.1   | Node Development Kit — build custom nodes with `#[aether_node]`          |
| [`aetherdsp-ndk-macro`](https://crates.io/crates/aetherdsp-ndk-macro) | 0.1.1   | Proc-macro behind the NDK                                                |
| [`aetherdsp-midi`](https://crates.io/crates/aetherdsp-midi)           | 0.1.1   | MIDI engine with 8 tuning systems including Ethiopian, Arabic, Gamelan   |
| [`aetherdsp-sampler`](https://crates.io/crates/aetherdsp-sampler)     | 0.2.0   | Polyphonic sampler with ArcSwap lock-free instrument loading             |
| [`aetherdsp-timbre`](https://crates.io/crates/aetherdsp-timbre)       | 0.1.1   | FFT-based spectral timbre analysis and transfer                          |
| [`aetherdsp-manifest`](https://crates.io/crates/aetherdsp-manifest)   | 0.1.1   | Node package manifest format                                             |
| [`aetherdsp-registry`](https://crates.io/crates/aetherdsp-registry)   | 0.1.1   | Runtime node type registry                                               |

### The Studio (Aether Studio)

A 4-mode music production interface built with React + Tauri:

- **Explore** — World map with 57 instruments from 8 regions. Click a region, browse its instruments, try them with your PC keyboard before adding to your project.
- **Create** — Modular node graph. Connect oscillators, filters, reverb, LFO, granular synthesis, and more.
- **Arrange** — Timeline and piano roll with scale highlighting for world music tuning systems.
- **Perform** — Clip launcher for live performance.

---

## DSP Nodes

| Node                  | Description                                                           |
| --------------------- | --------------------------------------------------------------------- |
| `Oscillator`          | BLEP anti-aliased sawtooth/square/triangle/sine, tuning table support |
| `StateVariableFilter` | LP/HP/BP/Notch simultaneously (Cytomic SVF)                           |
| `MoogLadder`          | Huovilainen model, self-oscillation, audio-rate modulation            |
| `FormantFilter`       | Vowel shaping A/E/I/O/U morph — essential for wind instruments        |
| `AdsrEnvelope`        | Sample-accurate ADSR with gate                                        |
| `Lfo`                 | 5 waveforms: sine, triangle, square, S&H, random-smooth               |
| `Reverb`              | Freeverb (8 comb + 4 allpass filters)                                 |
| `DelayLine`           | Feedback delay with tempo sync                                        |
| `KarplusStrong`       | Physically accurate plucked string synthesis                          |
| `Granular`            | Grain size, density, pitch scatter, position — world music textures   |
| `Gain`                | Smoothed gain control                                                 |
| `Mixer`               | N-input summing mixer                                                 |
| `SamplerNode`         | Polyphonic sampler, MIDI-driven, ArcSwap lock-free                    |
| `TimbreTransferNode`  | FFT spectral envelope transfer                                        |
| `ScopeNode`           | Oscilloscope output                                                   |

---

## Tuning Systems

AetherDSP treats tuning as a first-class feature. Every instrument loads with its correct tuning system by default.

| System              | Description                                   |
| ------------------- | --------------------------------------------- |
| 12-TET              | Standard equal temperament                    |
| Ethiopian Tizita    | Pentatonic with characteristic flat intervals |
| Ethiopian Bati      | Minor pentatonic variant                      |
| Arabic Maqam Rast   | Quarter-tone flats on 3rd and 7th             |
| Arabic Maqam Bayati | Half-flat on 2nd degree                       |
| Indian Raga Yaman   | Just intonation, raised 4th (Kalyan thaat)    |
| Gamelan Slendro     | 5-tone Javanese scale                         |
| Gamelan Pelog       | 7-tone Javanese scale with unequal intervals  |
| Just Intonation     | Pure harmonic ratios                          |

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
| Node.js           | 18+ LTS | For the UI                                |

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
│   ├── aether-nodes/       # DSP nodes: 12 nodes including granular, Moog, reverb
│   ├── aether-midi/        # MIDI engine + 9 tuning systems
│   ├── aether-sampler/     # Polyphonic sampler with ArcSwap
│   ├── aether-timbre/      # Spectral timbre analysis and transfer
│   ├── aether-ndk/         # Node Development Kit
│   ├── aether-ndk-macro/   # #[aether_node] proc-macro
│   ├── aether-manifest/    # Node package manifest format
│   ├── aether-registry/    # Runtime node registry
│   ├── aether-host/        # CPAL audio host + WebSocket bridge (not published)
│   └── aether-cli/         # Developer CLI (not published)
├── ui/                     # React + Tauri studio interface
│   ├── src/
│   │   ├── modes/          # Explore, Create, Arrange, Perform
│   │   ├── components/     # TopBar, KeyboardPlayer, ModulationMatrix
│   │   ├── catalog/        # 57-instrument world music catalog
│   │   └── studio/         # Node graph, engine store, WebSocket
│   └── src-tauri/          # Tauri standalone app wrapper
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
- Tauri standalone app build (Linux)

---

## Roadmap

| Version | Milestone                                                                        |
| ------- | -------------------------------------------------------------------------------- |
| v0.1    | RT engine + UI + WebSocket + 9 crates on crates.io                               |
| v0.2    | 12 DSP nodes, world tuning systems, ArcSwap RT fix, Tauri app, modulation matrix |
| v0.3    | SIMD optimization, real instrument samples, CLAP plugin export                   |
| v0.4    | Collaborative sessions, AI timbre transfer, mobile support                       |
| v1.0    | Stable public release                                                            |

---

## License

MIT — see [LICENSE](LICENSE)

GitHub: [github.com/1yos/aether-dsp](https://github.com/1yos/aether-dsp)
