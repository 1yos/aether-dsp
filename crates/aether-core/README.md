# aether-core

[![crates.io](https://img.shields.io/crates/v/aether-core.svg)](https://crates.io/crates/aether-core)
[![docs.rs](https://docs.rs/aether-core/badge.svg)](https://docs.rs/aether-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Hard real-time modular DSP engine for Rust.

```
64-sample buffer · 48 kHz · ≤1.33 ms deadline · Zero allocations · Lock-free
```

## What's in this crate

| Module        | Description                                                    |
| ------------- | -------------------------------------------------------------- |
| `arena`       | Generational arena — O(1) insert/remove, no fragmentation      |
| `graph`       | DAG audio graph with topological sort (Kahn's algorithm)       |
| `scheduler`   | Lock-free RT scheduler — processes sorted node array each tick |
| `buffer_pool` | Pre-allocated audio buffer pool, zero RT allocation            |
| `param`       | Smoothed parameters with per-sample interpolation              |
| `command`     | SPSC command ring — control thread → RT thread mutations       |
| `node`        | `DspNode` trait — implement this to create a processing node   |
| `state`       | Serializable node state for save/restore                       |

## Real-time guarantees

| Rule                            | Enforcement                                |
| ------------------------------- | ------------------------------------------ |
| No heap allocation in RT thread | Pre-allocated arena + buffer pool          |
| No locks in RT thread           | SPSC ring buffer (`ringbuf`)               |
| No I/O in RT thread             | All I/O on control/tokio threads           |
| Bounded execution               | Flat topo-sorted array, ≤32 commands/tick  |
| No recursion                    | Iterative Kahn's sort, iterative execution |

## Quick start

```rust
use aether_core::{
    arena::NodeArena,
    graph::AudioGraph,
    scheduler::Scheduler,
    node::DspNode,
    param::ParamBlock,
    state::StateBlob,
    BUFFER_SIZE,
};

// Implement a node
struct Gain { amount: f32 }

impl DspNode for Gain {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; aether_core::MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        let input = inputs[0].map(|b| b.as_ref()).unwrap_or(&[0.0; BUFFER_SIZE]);
        for (o, i) in output.iter_mut().zip(input.iter()) {
            *o = i * self.amount;
        }
    }
    fn capture_state(&self) -> StateBlob { StateBlob::EMPTY }
    fn restore_state(&mut self, _: StateBlob) {}
    fn type_name(&self) -> &'static str { "Gain" }
}
```

## Benchmark results

| Benchmark                   | Result      |
| --------------------------- | ----------- |
| `param_fill_buffer_64`      | **51.7 ns** |
| Arena insert/remove ×1000   | < 5 µs      |
| Scheduler (1000 noop nodes) | < 100 µs    |

## License

MIT — see [LICENSE](../../LICENSE)
