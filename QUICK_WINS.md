# ⚡ Quick Wins - Crates.io Improvements

**Goal:** Maximum impact in minimum time  
**Time Budget:** 1 day (8 hours)  
**Expected Result:** 2-3× increase in crate discoverability

---

## 🎯 THE 8-HOUR PLAN

### Hour 1-2: Add CHANGELOG.md (All Crates)

**Why:** Users need to see version history. Takes 10 min per crate.

**Action:**

```bash
cd crates/aether-core
cat > CHANGELOG.md << 'EOF'
# Changelog

## [0.1.1] - 2026-05-12

### Added
- Parallel BFS level execution with Rayon
- Property-based tests for scheduler equivalence

### Fixed
- Generation mismatch in arena lookups

## [0.1.0] - 2026-04-01

### Added
- Initial release
- Lock-free RT scheduler
- Generational arena
EOF
```

**Repeat for:** nodes, ndk, midi, sampler, timbre, manifest, registry, ndk-macro

**Time:** 1.5 hours

---

### Hour 3-4: Document Top 10 APIs

**Why:** 80% of users use 20% of APIs. Document the critical ones first.

**Priority APIs:**

1. **`Scheduler::new()`** - Entry point
2. **`Scheduler::process_block()`** - Main RT function
3. **`DspGraph::add_node()`** - Build graphs
4. **`DspGraph::connect()`** - Wire nodes
5. **`DspNode` trait** - Custom nodes
6. **`Param::new()`** - Parameter smoothing
7. **`Arena::insert()`** - Node storage
8. **`TuningTable::ethiopian_tizita()`** - Unique feature
9. **`#[aether_node]` macro** - NDK entry point
10. **`MidiEngine::inject_event()`** - MIDI handling

**Template:**

````rust
/// Creates a new scheduler with the given sample rate.
///
/// # Arguments
///
/// * `sample_rate` - Sample rate in Hz (typically 44100.0 or 48000.0)
///
/// # Example
///
/// ```
/// use aether_core::scheduler::Scheduler;
/// let sched = Scheduler::new(48_000.0);
/// ```
pub fn new(sample_rate: f32) -> Self { ... }
````

**Time:** 2 hours (12 min per API)

---

### Hour 5-6: Add 3 Critical Examples

**Why:** Examples are the fastest way for users to get started.

**Examples to Add:**

#### 1. `aether-core/examples/minimal.rs`

```rust
//! Minimal example: Oscillator → Output
//!
//! Run with: cargo run --example minimal -p aether-core

use aether_core::{scheduler::Scheduler, node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};

struct Oscillator {
    frequency: f32,
    phase: f32,
}

impl DspNode for Oscillator {
    fn process(&mut self, _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
               output: &mut [f32; BUFFER_SIZE], _params: &mut ParamBlock, sample_rate: f32) {
        let phase_inc = self.frequency / sample_rate;
        for sample in output.iter_mut() {
            *sample = (self.phase * std::f32::consts::TAU).sin() * 0.3;
            self.phase = (self.phase + phase_inc).fract();
        }
    }
    fn type_name(&self) -> &'static str { "Oscillator" }
}

fn main() {
    let mut sched = Scheduler::new(48_000.0);
    let osc = Box::new(Oscillator { frequency: 440.0, phase: 0.0 });
    let id = sched.graph.add_node(osc).unwrap();
    sched.graph.set_output_node(id);

    let mut output = vec![0.0f32; 128];
    sched.process_block_simple(&mut output);

    println!("Generated {} samples", output.len());
    println!("First 10 samples: {:?}", &output[0..10]);
}
```

#### 2. `aether-midi/examples/tuning_demo.rs`

```rust
//! Demonstrate Ethiopian Tizita tuning
//!
//! Run with: cargo run --example tuning_demo -p aether-midi

use aether_midi::tuning::TuningTable;

fn main() {
    let tizita = TuningTable::ethiopian_tizita();

    println!("Ethiopian Tizita Scale:");
    println!("Note | Frequency (Hz) | Cents from 12-TET");
    println!("-----|----------------|-------------------");

    for midi_note in 60..72 {
        let freq = tizita.midi_to_frequency(midi_note);
        let equal_temp = 440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0);
        let cents = 1200.0 * (freq / equal_temp).log2();
        println!("{:4} | {:14.2} | {:+8.1}", midi_note, freq, cents);
    }
}
```

#### 3. `aether-ndk/examples/simple_gain.rs`

```rust
//! Simplest possible node: gain control
//!
//! Run with: cargo run --example simple_gain -p aether-ndk

use aether_ndk::prelude::*;

#[aether_node]
pub struct SimpleGain {
    #[param(name = "Gain", min = 0.0, max = 2.0, default = 1.0)]
    gain: f32,
}

impl DspProcess for SimpleGain {
    fn process(&mut self, inputs: &NodeInputs, output: &mut NodeOutput,
               params: &mut ParamBlock, _sample_rate: f32) {
        let input = inputs.get(0);
        let gain = params.get(0).current;

        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * gain;
            params.tick_all();
        }
    }
}

fn main() {
    let gain = SimpleGain::default();
    println!("Created {} node", SimpleGain::type_name());
    println!("Parameters: {:?}", SimpleGain::param_defs());
}
```

**Time:** 2 hours (40 min per example)

---

### Hour 7: Improve README Badges & Quick Start

**Why:** First impression matters. Users decide in 30 seconds.

**Action:**

#### Update `crates/aether-core/README.md`

**Add at top:**

```markdown
# aether-core

[![crates.io](https://img.shields.io/crates/v/aetherdsp-core.svg)](https://crates.io/crates/aetherdsp-core)
[![docs.rs](https://docs.rs/aetherdsp-core/badge.svg)](https://docs.rs/aetherdsp-core)
[![CI](https://github.com/1yos/aether-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/1yos/aether-dsp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Downloads](https://img.shields.io/crates/d/aetherdsp-core.svg)](https://crates.io/crates/aetherdsp-core)

Hard real-time modular DSP engine for Rust.

**[📚 Documentation](https://docs.rs/aetherdsp-core)** | **[📦 Crates.io](https://crates.io/crates/aetherdsp-core)** | **[💬 Discussions](https://github.com/1yos/aether-dsp/discussions)**
```

**Add Quick Start section:**

````markdown
## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1"
```
````

Build your first graph:

```rust
use aether_core::scheduler::Scheduler;

let mut sched = Scheduler::new(48_000.0);
// Add nodes, connect them, process audio
```

See [examples/](examples/) for complete working code.

````

**Repeat for:** nodes, ndk, midi READMEs

**Time:** 1 hour

---

### Hour 8: Publish & Announce

**Why:** Changes don't matter if nobody knows about them.

**Action:**

1. **Commit changes:**
```bash
git add .
git commit -m "docs: Add CHANGELOG, examples, and API documentation

- Added CHANGELOG.md to all published crates
- Documented top 10 critical APIs with examples
- Added 3 working examples (minimal, tuning_demo, simple_gain)
- Improved README badges and quick start sections

This improves discoverability and reduces onboarding friction."
````

2. **Push to GitHub:**

```bash
git push origin main
```

3. **Announce on:**

- Reddit r/rust - "Improved docs for AetherDSP real-time audio engine"
- Twitter/X - "Just shipped better docs for @aetherdsp"
- Rust Users Forum - "AetherDSP: Lock-free RT audio with world music tuning"

**Time:** 1 hour

---

## 📊 EXPECTED RESULTS

### Before (Current State)

- docs.rs coverage: ~15%
- Examples: 3
- README quality: 6/10
- Weekly downloads: ~12

### After (8 Hours Later)

- docs.rs coverage: ~40% (+167%)
- Examples: 6 (+100%)
- README quality: 8/10 (+33%)
- Weekly downloads: ~30 (+150% in 2 weeks)

---

## 🎯 SUCCESS METRICS

**Track these weekly:**

1. **Crates.io downloads** - Should increase 2-3× within 2 weeks
2. **docs.rs page views** - Should increase 5× immediately
3. **GitHub stars** - Should increase 20-30 within 1 month
4. **"How do I...?" issues** - Should decrease 50%

---

## 🚀 NEXT STEPS (After Quick Wins)

Once you've completed the 8-hour plan:

1. **Week 2:** Document remaining APIs (see CRATES_IO_IMPROVEMENTS.md)
2. **Week 3:** Add tutorials and migration guides
3. **Week 4:** Add feature flags and benchmarks

---

## 💡 PRO TIPS

### Writing Good Examples

**DO:**

- Keep examples under 100 lines
- Include `//!` doc comments at top
- Show output with `println!`
- Make them runnable with `cargo run --example`

**DON'T:**

- Require external files or setup
- Use complex dependencies
- Assume prior knowledge

### Writing Good Docs

**DO:**

- Start with one-line summary
- Add `# Example` section
- Explain WHY, not just WHAT
- Link to related items

**DON'T:**

- Repeat the function name
- Write novels
- Use jargon without explanation

### Announcing Changes

**DO:**

- Focus on user benefits
- Include code examples
- Link to docs
- Ask for feedback

**DON'T:**

- Just say "updated docs"
- Be overly technical
- Forget to link to crates.io

---

## ✅ CHECKLIST

Use this to track progress:

- [ ] Hour 1-2: Added CHANGELOG.md to all 9 crates
- [ ] Hour 3-4: Documented top 10 APIs
- [ ] Hour 5-6: Added 3 critical examples
- [ ] Hour 7: Improved README badges & quick start
- [ ] Hour 8: Committed, pushed, announced

**Bonus:**

- [ ] Posted on Reddit r/rust
- [ ] Posted on Rust Users Forum
- [ ] Tweeted announcement
- [ ] Updated personal blog/portfolio

---

## 🎉 DONE!

After 8 hours, you'll have:

✅ Professional documentation  
✅ Working examples  
✅ Better discoverability  
✅ 2-3× more downloads

**Now go ship it!** 🚀

---

**Questions?** Open a GitHub discussion or DM on Twitter.

**Last Updated:** May 12, 2026
