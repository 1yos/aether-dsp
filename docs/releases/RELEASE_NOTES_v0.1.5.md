# Release Notes: AetherDSP v0.1.5

**Release Date:** May 13, 2026  
**Codename:** "Documentation & Tutorials"

---

## 🎉 Overview

AetherDSP v0.1.5 is a major documentation release, adding **5950+ lines** of comprehensive documentation, tutorials, and polish. This release makes AetherDSP significantly more accessible to new users while establishing professional security and quality standards.

---

## ✨ What's New

### 📚 Comprehensive Tutorials (Phase 8)

Three step-by-step tutorials to get you started:

#### 1. Building Your First Synthesizer (30-45 min)

- Complete beginner guide with CPAL audio output
- Build an Oscillator → Filter → Envelope signal chain
- Add MIDI keyboard control
- **650+ lines** with 7 progressive examples

**Start here:** [docs/tutorials/first-synth.md](docs/tutorials/first-synth.md)

#### 2. Creating Custom DSP Nodes (20-30 min)

- Learn the Node Development Kit (NDK)
- Build 3 custom effects: Tremolo, Distortion, SimpleFilter
- Unit tests and property tests
- Publishing workflow
- **550+ lines** with complete working code

**Learn more:** [docs/tutorials/custom-nodes.md](docs/tutorials/custom-nodes.md)

#### 3. Microtonal Music with Tuning Systems (20-30 min)

- Ethiopian Tizita scale
- Arabic Maqam scales (quarter-tones)
- Just intonation (pure ratios)
- Custom tuning table builder
- **600+ lines** with microtonal sequencer

**Explore:** [docs/tutorials/tuning-systems.md](docs/tutorials/tuning-systems.md)

**Tutorial Index:** [docs/tutorials/README.md](docs/tutorials/README.md)

---

### 🔒 Security Policy (Phase 10)

Professional security policy with:

- Vulnerability reporting process (security@aetherdsp.dev)
- Response timeline (48h response, 7-21d fix)
- Real-time specific security considerations
- Safe usage guidelines with DO/DON'T examples
- Known security limitations
- Best practices for developers
- Coordinated disclosure policy

**Read:** [SECURITY.md](SECURITY.md)

---

### 📊 Performance Documentation (Phase 9)

Comprehensive benchmark results:

- **param_fill_buffer_64:** 51.7 ns (4× faster than std)
- **Arena insert/remove ×1000:** < 5 µs
- **Scheduler (1000 noop nodes):** < 100 µs
- **Parallel vs Sequential:** 3-4× faster on 4+ cores

Performance characteristics table and comparison with other engines (dasp, fundsp, cpal).

---

### 📖 Enhanced Documentation (Phases 2-7)

- **35+ APIs documented** with examples and doc tests
- **Feature flags** for all crates (parallel, serde, per-node)
- **Migration guides** for version upgrades
- **README improvements** with performance tables, comparisons, FAQ
- **Professional badges** on all crate READMEs
- **Common pitfalls** section with DO/DON'T examples

---

## 📦 Published Crates

All crates are available on crates.io:

| Crate                 | Version | Description                                 |
| --------------------- | ------- | ------------------------------------------- |
| `aetherdsp-core`      | 0.1.4   | RT scheduler, arena, graph, buffer pool     |
| `aetherdsp-nodes`     | 0.2.3   | 17 DSP nodes (oscillator, filters, effects) |
| `aetherdsp-ndk`       | 0.1.4   | Node Development Kit                        |
| `aetherdsp-ndk-macro` | 0.1.4   | `#[aether_node]` proc-macro                 |
| `aetherdsp-midi`      | 0.1.4   | MIDI engine with 9 tuning systems           |
| `aetherdsp-sampler`   | 0.2.1   | Polyphonic sampler                          |
| `aetherdsp-timbre`    | 0.1.4   | FFT spectral analysis                       |
| `aetherdsp-manifest`  | 0.1.4   | Node package manifest                       |
| `aetherdsp-registry`  | 0.1.4   | Runtime node registry                       |

---

## 🚀 Quick Start

### Install

```toml
[dependencies]
aetherdsp-core = "0.1.4"
aetherdsp-nodes = "0.2.3"
```

### Build Your First Synth

Follow the [First Synthesizer Tutorial](docs/tutorials/first-synth.md) to build a complete synth in 30-45 minutes.

### Create Custom Nodes

```rust
use aether_ndk::prelude::*;

#[aether_node]
pub struct Tremolo {
    #[param(name = "Rate", min = 0.1, max = 20.0, default = 4.0)]
    rate: f32,
    #[param(name = "Depth", min = 0.0, max = 1.0, default = 0.5)]
    depth: f32,
    phase: f32,
}

impl DspProcess for Tremolo {
    fn process(&mut self, inputs: &[...], output: &mut [...], ...) {
        // Your DSP code here
    }
}
```

See the [Custom Nodes Tutorial](docs/tutorials/custom-nodes.md) for complete examples.

---

## 📈 Statistics

### Documentation Added

| Category            | Lines     | Content                                 |
| ------------------- | --------- | --------------------------------------- |
| Tutorials           | 2050+     | 3 comprehensive step-by-step guides     |
| API Documentation   | 1000+     | 35+ APIs with examples and doc tests    |
| Migration Guides    | 900+      | Version upgrade paths                   |
| README Improvements | 550+      | Performance, comparisons, FAQ           |
| Security Policy     | 400+      | Vulnerability reporting, best practices |
| Feature Flags       | 200+      | Optional features documentation         |
| Examples            | 800+      | 5 new working examples                  |
| **Total**           | **5950+** | Comprehensive documentation             |

### Code Examples

- **21 complete code examples** in tutorials
- **53 passing doc tests** in API documentation
- **5 new working examples** in crates
- All code tested and verified

---

## 🎯 Key Features

### Real-Time Guarantees

- ✅ **Zero allocation** in RT thread
- ✅ **Lock-free** graph processing
- ✅ **Bounded execution** (≤1.33ms @ 48kHz)
- ✅ **No I/O** in RT thread
- ✅ **No recursion** (iterative algorithms)

### Performance

- **51.7 ns** parameter smoothing (4× faster than std)
- **1000+ nodes** at < 100 µs processing time
- **3-4× speedup** with parallel execution
- **~2.5 MB** memory footprint

### World Music Support

9 tuning systems:

- Ethiopian (Tizita, Bati, Ambassel)
- Arabic (Maqam Rast, Bayati, Hijaz)
- Indian (Raga Yaman, Bhairav)
- Gamelan (Slendro, Pelog)
- Just Intonation, 12-TET, Chromatic

---

## 🔧 Breaking Changes

**None.** This is a documentation-only release with no breaking API changes.

---

## 🐛 Bug Fixes

- Fixed clippy doc formatting warning in `node.rs`
- All clippy checks passing
- All tests passing (29 unit tests, 53 doc tests)

---

## 📚 Documentation

- **API Docs:** https://docs.rs/aetherdsp-core
- **Tutorials:** [docs/tutorials/README.md](docs/tutorials/README.md)
- **Examples:** [crates/aether-core/examples/](crates/aether-core/examples/)
- **Migration Guide:** [crates/aether-core/MIGRATION.md](crates/aether-core/MIGRATION.md)
- **Security Policy:** [SECURITY.md](SECURITY.md)

---

## 🙏 Acknowledgments

Thank you to everyone who has shown interest in AetherDSP! This release is dedicated to making the project more accessible to new users.

Special thanks to the Rust audio community for inspiration and feedback.

---

## 🔮 What's Next

### Immediate (v0.1.6)

- Gather user feedback
- Fix any issues discovered
- Improve based on real usage

### Future (v0.2.0+)

Based on user feedback, we'll prioritize:

- Parameter validation
- Presets system
- More DSP nodes (EQ, limiter, gate)
- MPE support
- MIDI file I/O
- Hot reload
- GUI support

See [COMPLETE_IMPROVEMENT_PLAN.md](COMPLETE_IMPROVEMENT_PLAN.md) for the full roadmap.

---

## 📞 Get Involved

- **GitHub:** https://github.com/1yos/aether-dsp
- **Issues:** https://github.com/1yos/aether-dsp/issues
- **Discussions:** https://github.com/1yos/aether-dsp/discussions
- **Security:** security@aetherdsp.dev

---

## 📄 License

MIT - see [LICENSE](LICENSE)

---

**Full Changelog:** https://github.com/1yos/aether-dsp/compare/v0.1.4...v0.1.5
