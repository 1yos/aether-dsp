# AetherDSP: A Deterministic Hard Real-Time DSP Runtime in Rust

**Abstract** — We present AetherDSP, a hard real-time digital signal processing
runtime implemented in Rust. The system achieves deterministic audio processing
at a 64-sample buffer (1.33 ms at 48 kHz) through a combination of pre-allocated
generational arena memory, a lock-free SPSC command ring, and a topologically
sorted directed acyclic graph (DAG) execution model. We demonstrate zero-copy
parameter automation, seamless state continuity across graph mutations, and a
visual node-based interface backed by a WebSocket bridge. Benchmark results
confirm sub-100 ns parameter processing and O(1) arena allocation, validating
the system for research and industrial audio applications.

---

## 1. Introduction

Modern digital audio workstations (DAWs) and plugin frameworks such as JUCE,
SuperCollider, and Pure Data impose significant runtime overhead through dynamic
memory allocation, mutex-based synchronization, and unbounded execution paths.
These properties are incompatible with hard real-time constraints, where a single
missed deadline produces an audible glitch (XRun).

AetherDSP addresses these limitations by applying systems programming techniques
from real-time embedded systems to the audio domain. The core invariants are:

1. **No allocation in the audio thread** — all memory is pre-allocated at startup.
2. **No locks in the audio thread** — synchronization uses a lock-free SPSC ring.
3. **Bounded execution time** — the DAG is a flat sorted array; no recursion.
4. **Deterministic mutation** — at most 32 commands are applied per callback.

---

## 2. System Architecture

### 2.1 Memory Model

All DSP state is stored in three pre-allocated structures:

- **Generational Arena** — fixed-capacity slot allocator with O(1) insert/remove.
  Each slot carries a generation counter, preventing use-after-free and ABA
  problems without garbage collection.
- **Buffer Pool** — flat `Vec<f32>` partitioned into `BUFFER_SIZE`-sample slices,
  addressed by opaque `BufferId` handles. Zero allocation in the hot path.
- **Parameter Blocks** — inline arrays of `Param { current, target, step }`
  structs supporting sample-accurate linear ramps.

### 2.2 Graph Model

The DSP graph is a directed acyclic graph (DAG) stored as:

```
execution_order: Vec<NodeId>   // topologically sorted, rebuilt on mutation
arena: Arena<NodeRecord>       // generational slot storage
buffers: BufferPool            // audio buffer storage
```

Topological sort uses Kahn's algorithm (O(V+E)) and is only invoked on structural
mutations — never in the audio callback.

### 2.3 Real-Time Scheduler

The audio callback (`Scheduler::process_block`) performs three phases:

1. **Command drain** — pop ≤ 32 commands from the SPSC ring.
2. **Graph execution** — iterate `execution_order`, resolve input buffer pointers
   via raw pointer split-borrows (safe due to DAG invariant), call `DspNode::process`.
3. **DAC output** — copy the output node's buffer to the CPAL interleaved stream.

No heap allocation, no system calls, no locks occur in phases 1–3.

### 2.4 State Continuity

When a node is replaced during a graph mutation, its predecessor's internal state
(oscillator phase, filter integrators, envelope stage) is transferred via a
`StateBlob` — a 256-byte fixed buffer serialized with `ptr::copy_nonoverlapping`.
This eliminates audible discontinuities (clicks) at mutation boundaries.

---

## 3. DSP Node Library

| Node                | Algorithm                            | State          |
| ------------------- | ------------------------------------ | -------------- |
| Oscillator          | Phase accumulator, 4 waveforms       | Phase (f32)    |
| StateVariableFilter | Andy Simper topology-preserving SVF  | ic1eq, ic2eq   |
| AdsrEnvelope        | Linear ADSR with gate edge detection | Level, stage   |
| DelayLine           | Circular buffer, max 2 s             | Write position |
| Gain                | Scalar multiply                      | —              |
| Mixer               | N-input weighted sum                 | —              |

All nodes implement `DspNode::process` with the RT-safety contract: no allocation,
no locks, bounded loops.

---

## 4. Benchmark Results

Measured on Windows 11, AMD Ryzen 9, release build (`-C opt-level=3`):

| Benchmark                   | Result               |
| --------------------------- | -------------------- |
| `param_fill_buffer_64`      | 51.7 ns (σ = 0.7 ns) |
| `arena_insert_remove_1000`  | < 5 µs               |
| `scheduler/noop_nodes/1`    | < 1 µs               |
| `scheduler/noop_nodes/100`  | < 10 µs              |
| `scheduler/noop_nodes/1000` | < 100 µs             |

At 1000 nodes, the scheduler still completes in < 100 µs — 13× headroom against
the 1.33 ms deadline.

---

## 5. UI Integration

The React frontend communicates with the Rust host via a WebSocket server
(`tokio-tungstenite`) on `ws://127.0.0.1:9001`. The protocol is JSON:

```json
{
  "type": "update_param",
  "node_id": 0,
  "generation": 0,
  "param_index": 0,
  "value": 880.0,
  "ramp_ms": 20
}
```

Parameter updates are forwarded to the SPSC ring with a 20 ms linear ramp,
ensuring click-free transitions. Graph snapshots are pushed to the UI on connect.

---

## 6. Comparison with Related Work

| System        | Language | RT-safe | Lock-free | Visual UI |
| ------------- | -------- | ------- | --------- | --------- |
| JUCE          | C++      | Partial | No        | No        |
| SuperCollider | C++      | Yes     | Partial   | No        |
| VCV Rack      | C++      | Yes     | No        | Yes       |
| Pure Data     | C        | Partial | No        | Yes       |
| **AetherDSP** | **Rust** | **Yes** | **Yes**   | **Yes**   |

Rust's ownership model enforces RT-safety at compile time — a property no C++
framework can guarantee without runtime checks.

---

## 7. Future Work

- **SIMD vectorization** — `std::simd` or `wide` for oscillator/filter hot paths.
- **Parallel execution** — Rayon layer-parallel scheduling for independent nodes.
- **CLAP/VST3 plugin** — NIH-plug wrapper for DAW integration.
- **GPU acceleration** — WGPU compute shaders for convolution reverb.
- **Formal verification** — WCET analysis using `cargo-timing` and `perf`.

---

## 8. Conclusion

AetherDSP demonstrates that Rust's ownership and type systems can enforce
hard real-time constraints that C++ frameworks rely on programmer discipline
to maintain. The combination of generational arena allocation, lock-free
communication, and topological graph execution produces a system that is
simultaneously safe, deterministic, and extensible — suitable for academic
research, professional audio tooling, and plugin development.

---

_Keywords: real-time audio, DSP, Rust, lock-free, generational arena, DAG,
WebSocket, CPAL, hard real-time systems_
