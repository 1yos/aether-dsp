opt# Aether CLI Reference v0.1

## Installation

```bash
cargo install --path crates/aether-cli
```

Or run directly from the workspace:

```bash
cargo run -p aether-cli -- <command>
```

---

## Commands

### `aether new <name>`

Scaffold a new AetherSDK project.

```bash
aether new my-synth
```

Creates:

```
my-synth/
├── aether.json          # Project manifest
├── Cargo.toml           # Rust crate
├── src/
│   ├── lib.rs
│   └── nodes/
│       └── mod.rs
├── presets/
└── .gitignore
```

---

### `aether node <name>`

Scaffold a new DSP node in `src/nodes/`.

```bash
aether node spectral-reverb
# Creates src/nodes/spectral_reverb.rs
```

Generated file:

```rust
use aether_ndk::prelude::*;

#[aether_node]
pub struct SpectralReverb {
    #[param(name = "Param1", min = 0.0, max = 1.0, default = 0.5)]
    param1: f32,
}

impl DspProcess for SpectralReverb {
    fn process(&mut self, inputs: &NodeInputs, output: &mut NodeOutput,
               params: &mut ParamBlock, _sample_rate: f32) {
        let input = inputs.get(0);
        let p = params.get(0).current;
        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * p;
            params.tick_all();
        }
    }
}
```

---

### `aether run`

Validate `aether.json` and launch the audio host.

```bash
aether run
```

Output:

```
✓ Manifest valid: 4 nodes, 3 connections
  Sample rate: 48000 Hz
  Block size:  64 samples

Starting audio host...
AetherDSP v0.1
Device: Default Output
Audio stream started.
```

---

### `aether build [--plugin <format>]`

Build the project. Optionally build as a plugin.

```bash
aether build                  # standard release build
aether build --plugin clap    # CLAP plugin
```

---

### `aether list`

List all registered node types with parameter ranges.

```bash
aether list
```

Output:

```
Registered node types (6):

  AdsrEnvelope  (5 params)
    • Attack   [0.0 – 10.0]  default: 0.01
    • Decay    [0.0 – 10.0]  default: 0.10
    • Sustain  [0.0 – 1.0]   default: 0.70
    • Release  [0.0 – 10.0]  default: 0.30
    • Gate     [0.0 – 1.0]   default: 0.00

  DelayLine  (3 params)
    • Time     [0.0 – 2.0]   default: 0.25
    ...
```

---

### `aether schema`

Print the full JSON schema for all registered nodes.

```bash
aether schema
aether schema > nodes.json
```

Output:

```json
[
  {
    "type_name": "Oscillator",
    "params": [
      { "name": "Frequency", "min": 20.0, "max": 20000.0, "default": 440.0 },
      { "name": "Amplitude", "min": 0.0,  "max": 1.0,     "default": 0.5 },
      { "name": "Waveform",  "min": 0.0,  "max": 3.0,     "default": 0.0 }
    ]
  },
  ...
]
```

---

### `aether validate`

Validate `aether.json` against the node registry.

```bash
aether validate
```

Output:

```
✓ aether.json is valid
  Project: my-synth v0.1.0
  Nodes:   4
  Edges:   3
```

Error example:

```
Error: Unknown node type: SpectralReverb
```

---

### `aether registry list`

List installed packages.

```bash
aether registry list
```

Output:

```
Installed packages (2):

  my-effects v0.1.0  — Custom effects pack
    • Tremolo
    • BitCrusher

  spatial-nodes v0.2.0  — Spatial audio nodes
    • Binaural
    • AmbisonicEncoder
```

---

### `aether registry install <path>`

Install a local package.

```bash
aether registry install ./my-effects
```

The package must contain a `package.json`:

```json
{
  "name": "my-effects",
  "version": "0.1.0",
  "description": "Custom effects",
  "author": "Your Name",
  "nodes": ["Tremolo", "BitCrusher"]
}
```

---

### `aether version`

Print version information.

```bash
aether version
# aether-cli v0.1.0 (AetherSDK)
```

---

## Exit Codes

| Code | Meaning                           |
| ---- | --------------------------------- |
| 0    | Success                           |
| 1    | Error (message printed to stderr) |

---

## Environment

The CLI respects:

- `AETHER_REGISTRY` — override registry path (default: `~/.aether/registry/`)
- `AETHER_LOG` — log level (`error`, `warn`, `info`, `debug`)
