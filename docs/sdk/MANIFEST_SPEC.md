# Aether Manifest Specification v0.1

## Overview

An Aether manifest (`aether.json`) is a JSON file that describes a complete
audio application: its nodes, connections, parameters, and build targets.
It is the portable, declarative format for AetherSDK projects.

---

## Schema

```json
{
  "name": "string",
  "version": "string",
  "engine": "string",
  "sample_rate": "integer",
  "block_size": "integer",
  "nodes": [ NodeDef ],
  "connections": [ ConnectionDef ],
  "output_node": "string",
  "plugin_targets": [ "string" ],
  "metadata": { }
}
```

### Top-level fields

| Field            | Type    | Required | Default        | Description                  |
| ---------------- | ------- | -------- | -------------- | ---------------------------- |
| `name`           | string  | ✓        | —              | Project name                 |
| `version`        | string  | ✓        | —              | Semver version               |
| `engine`         | string  | —        | `"aether-dsp"` | Engine identifier            |
| `sample_rate`    | integer | —        | `48000`        | Audio sample rate in Hz      |
| `block_size`     | integer | —        | `64`           | Buffer size in samples       |
| `nodes`          | array   | ✓        | —              | Node instances               |
| `connections`    | array   | —        | `[]`           | Audio routing                |
| `output_node`    | string  | ✓        | —              | ID of the final output node  |
| `plugin_targets` | array   | —        | `[]`           | Plugin formats to build      |
| `metadata`       | object  | —        | `{}`           | Arbitrary key-value metadata |

---

### NodeDef

```json
{
  "id": "osc1",
  "type": "Oscillator",
  "params": {
    "Frequency": 440.0,
    "Amplitude": 0.7
  }
}
```

| Field    | Type   | Required | Description                              |
| -------- | ------ | -------- | ---------------------------------------- |
| `id`     | string | ✓        | Unique identifier within this project    |
| `type`   | string | ✓        | Registered node type name                |
| `params` | object | —        | Initial parameter values by display name |

Parameter values not specified default to the node's registered default.

---

### ConnectionDef

```json
{
  "from": "osc1",
  "to": "filt1",
  "slot": 0
}
```

| Field  | Type    | Required | Description                                  |
| ------ | ------- | -------- | -------------------------------------------- |
| `from` | string  | ✓        | Source node ID                               |
| `to`   | string  | ✓        | Destination node ID                          |
| `slot` | integer | —        | Input slot index on destination (default: 0) |

---

## Complete Example

```json
{
  "name": "acid-bass",
  "version": "0.1.0",
  "engine": "aether-dsp",
  "sample_rate": 48000,
  "block_size": 64,
  "nodes": [
    {
      "id": "osc",
      "type": "Oscillator",
      "params": { "Frequency": 110.0, "Amplitude": 0.8, "Waveform": 1.0 }
    },
    {
      "id": "filt",
      "type": "StateVariableFilter",
      "params": { "Cutoff": 800.0, "Resonance": 8.0, "Mode": 0.0 }
    },
    {
      "id": "env",
      "type": "AdsrEnvelope",
      "params": {
        "Attack": 0.005,
        "Decay": 0.2,
        "Sustain": 0.3,
        "Release": 0.1,
        "Gate": 1.0
      }
    },
    {
      "id": "out",
      "type": "Gain",
      "params": { "Gain": 0.9 }
    }
  ],
  "connections": [
    { "from": "osc", "to": "filt", "slot": 0 },
    { "from": "filt", "to": "env", "slot": 0 },
    { "from": "env", "to": "out", "slot": 0 }
  ],
  "output_node": "out",
  "plugin_targets": ["clap"],
  "metadata": {
    "author": "Your Name",
    "description": "303-style acid bass patch",
    "tags": ["bass", "acid", "synth"]
  }
}
```

---

## Validation Rules

The manifest is validated by `aether validate` and `Manifest::validate()`:

1. All `id` values must be unique.
2. Every `type` must be registered in the node registry.
3. Every `from` and `to` in connections must reference a valid `id`.
4. `output_node` must reference a valid `id`.

---

## Plugin Targets

| Value    | Format     | Output                             |
| -------- | ---------- | ---------------------------------- |
| `"clap"` | CLAP 1.x   | `aether_plugin.clap`               |
| `"vst3"` | VST3       | `aether_plugin.vst3` _(v0.3)_      |
| `"au"`   | Audio Unit | `aether_plugin.component` _(v0.4)_ |

---

## CLI Integration

```bash
# Validate
aether validate

# Run with manifest
aether run

# Build plugin
aether build --plugin clap

# Print schema for all registered nodes
aether schema
```

---

## Programmatic Use

```rust
use aether_manifest::Manifest;
use aether_ndk::registry::builtin_registry;

let manifest = Manifest::from_file(Path::new("aether.json"))?;
let registry = builtin_registry();
manifest.validate(&registry)?;
let graph = manifest.build_graph(&registry, 48_000.0)?;
```

---

## Versioning

The manifest format follows semantic versioning. Breaking changes increment
the major version. The `engine` field identifies the runtime required.

| Manifest Version | Engine         | Status  |
| ---------------- | -------------- | ------- |
| 0.1              | aether-dsp 0.1 | Current |
