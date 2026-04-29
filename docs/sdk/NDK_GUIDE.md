# Aether NDK — Node Development Kit Guide v0.1

## Overview

The Aether NDK lets you build custom DSP nodes in Rust with minimal boilerplate.
A node is a struct annotated with `#[aether_node]` that implements `DspProcess`.
The macro handles everything else: parameter registration, `Default`, metadata,
and engine integration.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
aether-ndk = { path = "../aether-dsp/crates/aether-ndk" }
```

Write a node:

```rust
use aether_ndk::prelude::*;

#[aether_node]
pub struct Tremolo {
    #[param(name = "Rate",  min = 0.1, max = 20.0, default = 4.0)]
    rate: f32,
    #[param(name = "Depth", min = 0.0, max = 1.0,  default = 0.5)]
    depth: f32,
    // Internal state — not a param (no #[param] attribute)
    phase: f32,
}

impl DspProcess for Tremolo {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        sample_rate: f32,
    ) {
        let input = inputs.get(0);  // slot 0, returns silence if unconnected
        for (i, out) in output.iter_mut().enumerate() {
            let rate  = params.get(0).current;
            let depth = params.get(1).current;
            let lfo = 1.0 - depth * 0.5 * (1.0 - (self.phase * std::f32::consts::TAU).cos());
            *out = input[i] * lfo;
            self.phase = (self.phase + rate / sample_rate).fract();
            params.tick_all();  // advance all param smoothers by one sample
        }
    }
}
```

Register it and add to the graph:

```rust
use aether_ndk::{register_node, registry::builtin_registry, into_node};

let mut registry = builtin_registry();
register_node!(registry, Tremolo);

// Or directly:
let node = into_node(Tremolo::default());
graph.add_node(node);
```

---

## The `#[aether_node]` Macro

### What it generates

Given:

```rust
#[aether_node]
pub struct MyNode {
    #[param(name = "Cutoff", min = 20.0, max = 20000.0, default = 1000.0)]
    cutoff: f32,
    ic1eq: f32,  // internal state
}
```

The macro generates:

```rust
// 1. Default impl using param defaults and Default::default() for state
impl Default for MyNode {
    fn default() -> Self {
        Self { cutoff: 1000.0, ic1eq: Default::default() }
    }
}

// 2. Static param count
impl MyNode {
    pub const PARAM_COUNT: usize = 1;
    pub fn param_defs() -> &'static [ParamDef] { ... }
}

// 3. Metadata trait
impl AetherNodeMeta for MyNode {
    fn type_name() -> &'static str { "MyNode" }
    fn param_defs() -> &'static [ParamDef] { ... }
}
```

### `#[param]` attributes

| Key       | Type   | Required | Description                           |
| --------- | ------ | -------- | ------------------------------------- |
| `name`    | `&str` | No       | Display name (defaults to field name) |
| `min`     | `f32`  | No       | Minimum value (default: 0.0)          |
| `max`     | `f32`  | No       | Maximum value (default: 1.0)          |
| `default` | `f32`  | No       | Initial value (default: 0.0)          |

---

## The `DspProcess` Trait

```rust
pub trait DspProcess: Send {
    fn process(
        &mut self,
        inputs: &NodeInputs,
        output: &mut NodeOutput,
        params: &mut ParamBlock,
        sample_rate: f32,
    );

    // Optional: implement for state continuity across graph mutations
    fn capture_state(&self) -> StateBlob { StateBlob::EMPTY }
    fn restore_state(&mut self, _state: StateBlob) {}
}
```

### `NodeInputs`

```rust
let audio = inputs.get(0);   // &[f32; 64] — slot 0, silence if unconnected
let cv    = inputs.get(1);   // slot 1
```

### `ParamBlock`

```rust
let freq = params.get(0).current;   // current smoothed value
params.tick_all();                   // call once per sample in the loop
```

### State continuity

Implement `capture_state` / `restore_state` to preserve DSP state when a node
is replaced during a live graph mutation (prevents clicks):

```rust
fn capture_state(&self) -> StateBlob {
    StateBlob::from_value(&self.phase)
}
fn restore_state(&mut self, state: StateBlob) {
    self.phase = state.to_value();
}
```

---

## Real-Time Safety Rules

Your `process` implementation **must** follow these rules:

| Rule                                        | Why                              |
| ------------------------------------------- | -------------------------------- |
| No heap allocation (`Box`, `Vec`, `String`) | Causes non-deterministic latency |
| No locks (`Mutex`, `RwLock`)                | Risk of priority inversion       |
| No I/O (`println!`, file access)            | Unbounded execution time         |
| No unbounded loops                          | Deadline violation               |
| No recursion                                | Stack overflow risk              |

The Rust compiler enforces most of these via the `Send` bound and borrow checker.

---

## Node Registry

```rust
use aether_ndk::{register_node, registry::builtin_registry};

let mut registry = builtin_registry();  // pre-loaded with built-in nodes
register_node!(registry, Tremolo);      // add your node

// Instantiate by name (used by manifest system and CLI)
let node = registry.create("Tremolo").unwrap();

// List all registered types
for name in registry.list() {
    println!("{name}");
}
```

---

## Built-in Nodes

| Type Name             | Params                                | Description              |
| --------------------- | ------------------------------------- | ------------------------ |
| `Oscillator`          | Frequency, Amplitude, Waveform        | Sine/saw/square/triangle |
| `StateVariableFilter` | Cutoff, Resonance, Mode               | LP/HP/BP SVF             |
| `AdsrEnvelope`        | Attack, Decay, Sustain, Release, Gate | ADSR                     |
| `DelayLine`           | Time, Feedback, Wet                   | Feedback delay           |
| `Gain`                | Gain                                  | Scalar multiply          |
| `Mixer`               | —                                     | N-input sum              |

---

## Examples

```bash
cargo run --example tremolo      -p aether-ndk
cargo run --example bitcrusher   -p aether-ndk
cargo run --example registry_demo -p aether-ndk
```

---

## Packaging Your Node

Create a `package.json` in your crate root:

```json
{
  "name": "my-effects",
  "version": "0.1.0",
  "description": "Custom effects for AetherSDK",
  "author": "Your Name",
  "nodes": ["Tremolo", "BitCrusher"]
}
```

Install locally:

```bash
aether registry install ./my-effects
```
