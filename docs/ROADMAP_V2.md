# AetherDSP — v0.2 Roadmap

## Overview

v0.2 focuses on three pillars: **WebSocket synchronization hardening**,
**advanced modulation**, and **SIMD performance**. Each milestone is scoped
to be independently shippable.

---

## Milestone 1 — WebSocket Sync (v0.2.0)

**Goal:** Full bidirectional graph control from the UI with sub-frame latency.

### Tasks

- [ ] **Snapshot push on mutation** — after every `AddNode`/`Connect`/`Disconnect`
      command, the control thread pushes a fresh `GraphSnapshot` to all connected
      WebSocket clients automatically (no polling).

- [ ] **Reconnect logic in UI** — `useWebSocket.ts` retries with exponential
      backoff (1s, 2s, 4s, max 30s) on disconnect.

- [ ] **Add/Remove node from UI** — Toolbar "+" buttons send `AddNode` commands
      over WebSocket; the Rust host allocates the node and responds with the new
      `NodeId`.

- [ ] **Edge connect/disconnect from UI** — React Flow `onConnect`/`onEdgesDelete`
      callbacks send `Connect`/`Disconnect` commands.

- [ ] **Preset save/load** — serialize `GraphSnapshot` to JSON, store in
      `localStorage`, restore on load.

### API additions

```rust
// New command variants
Command::AddNodeTyped { node_type: String, id: NodeId }
Command::SetOutputNode { id: NodeId }
```

```typescript
// New WS message types
{ "type": "add_node", "node_type": "Oscillator" }
{ "type": "connect", "src_id": 0, "dst_id": 1, "slot": 0 }
{ "type": "disconnect", "dst_id": 1, "slot": 0 }
{ "type": "remove_node", "node_id": 2, "generation": 0 }
```

---

## Milestone 2 — Advanced Modulation (v0.2.1)

**Goal:** LFO and modulation matrix for expressive sound design.

### New DSP Nodes

| Node         | Description                               | Params                            |
| ------------ | ----------------------------------------- | --------------------------------- |
| `LfoNode`    | Sine/triangle/square LFO                  | rate, depth, waveform             |
| `ModMatrix`  | Routes modulator outputs to target params | src, dst, amount                  |
| `SampleHold` | Sample-and-hold with trigger input        | —                                 |
| `Compressor` | RMS compressor with attack/release        | threshold, ratio, attack, release |

### Modulation Architecture

```
LfoNode → ModMatrix → target Param.target
```

`ModMatrix` writes directly into `ParamBlock` entries via the command ring,
maintaining RT safety. The modulation depth is itself a smoothed `Param`.

### UI additions

- Modulation cable rendering (dashed, colored differently from audio cables)
- LFO rate display with BPM sync option
- Modulation amount knob on each target parameter

---

## Milestone 3 — SIMD Optimization (v0.2.2)

**Goal:** 4–8× throughput improvement on oscillator, filter, and mixer hot paths.

### Strategy

Use `std::simd` (stable in Rust 1.78+) to process 4 samples per SIMD lane:

```rust
// Before: scalar loop
for i in 0..BUFFER_SIZE {
    output[i] = (self.phase * TAU).sin() * amp;
    self.phase = (self.phase + phase_inc).fract();
}

// After: SIMD batch
use std::simd::f32x4;
// Process 4 samples per iteration using vectorized sin approximation
```

### Targets

| Node                | Expected speedup |
| ------------------- | ---------------- |
| Oscillator (sine)   | 3–4×             |
| StateVariableFilter | 2–3×             |
| Mixer (N inputs)    | 4–8×             |
| Param::fill_buffer  | 4×               |

### Benchmark targets

| Benchmark                   | v0.1     | v0.2 target |
| --------------------------- | -------- | ----------- |
| `param_fill_buffer_64`      | 51.7 ns  | < 15 ns     |
| `scheduler/noop_nodes/1000` | < 100 µs | < 30 µs     |

---

## Milestone 4 — Parallel Execution (v0.2.3)

**Goal:** Utilize all CPU cores for independent DSP layers.

### Implementation

```rust
// Current: sequential
for id in &execution_order { process_node(id); }

// v0.2: parallel layers
for layer in &parallel_layers {
    layer.par_iter().for_each(|id| process_node(id));
}
```

`parallel_layers: Vec<Vec<NodeId>>` is computed during topological sort —
nodes with no data dependency between them are grouped into the same layer
and processed with `rayon::par_iter`.

### Safety

Each node writes to its own `output_buffer` (unique `BufferId`). Rayon's
work-stealing scheduler respects the layer ordering. No shared mutable state
between nodes in the same layer.

---

## Milestone 5 — CLAP Plugin (v0.2.4)

**Goal:** Ship `aether_plugin.clap` loadable in Bitwig, REAPER, and Ardour.

### Tasks

- [ ] Complete NIH-plug integration in `crates/aether-plugin`
- [ ] MIDI note → oscillator frequency mapping
- [ ] Per-voice polyphony (4-voice stack)
- [ ] DAW parameter automation (all 4 exposed params)
- [ ] CI build producing `.clap` artifact on GitHub Actions

### Build

```bash
cargo build -p aether-plugin --release
# Output: target/release/aether_plugin.dll (Windows)
#         target/release/libaether_plugin.so (Linux)
#         target/release/libaether_plugin.dylib (macOS)
```

---

## Timeline

| Week | Milestone                     |
| ---- | ----------------------------- |
| 1–2  | WebSocket sync (v0.2.0)       |
| 3–4  | Modulation nodes (v0.2.1)     |
| 5–6  | SIMD optimization (v0.2.2)    |
| 7    | Parallel execution (v0.2.3)   |
| 8    | CLAP plugin (v0.2.4)          |
| 9    | Integration testing + release |

---

## Success Criteria

- [ ] All WebSocket commands round-trip in < 5 ms
- [ ] LFO modulates filter cutoff without clicks
- [ ] SIMD `param_fill_buffer_64` < 15 ns
- [ ] 1000-node graph processes in < 30 µs
- [ ] `.clap` loads and produces audio in Bitwig Studio
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All Criterion benchmarks show improvement over v0.1 baseline
