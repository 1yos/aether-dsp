//! # aether-core
//!
//! Hard real-time modular DSP engine — lock-free graph scheduler,
//! generational arena, and zero-allocation buffer pool.
//!
//! ```
//! 64-sample buffer · 48 kHz · ≤1.33 ms deadline · Zero allocations · Lock-free
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Control thread                    RT audio thread
//! ──────────────                    ───────────────
//! AudioGraph (add/remove nodes)     Scheduler::process_block()
//!     │                                 │
//!     │  CommandRing (SPSC)             ├─ drain CommandRing
//!     └────────────────────────────────►│
//!                                       ├─ iterate sorted NodeArena
//!                                       ├─ borrow buffers from BufferPool
//!                                       └─ call DspNode::process() per node
//! ```
//!
//! ## Real-time guarantees
//!
//! | Rule | Enforcement |
//! |---|---|
//! | No heap allocation | Pre-allocated arena + buffer pool |
//! | No locks | SPSC ring buffer (`ringbuf`) |
//! | No I/O | All I/O on control/tokio threads |
//! | Bounded execution | Flat topo-sorted array, ≤32 commands/tick |
//!
//! ## Quick start
//!
//! See [`node::DspNode`] to implement a processing node, [`graph::AudioGraph`]
//! to build a patch, and [`scheduler::Scheduler`] to drive the RT loop.

pub mod arena;
pub mod buffer_pool;
pub mod command;
pub mod graph;
pub mod node;
pub mod param;
pub mod scheduler;
pub mod state;

/// Audio buffer size in samples. Hard real-time constraint: 64 samples @ 48kHz = 1.33ms deadline.
pub const BUFFER_SIZE: usize = 64;

/// Maximum number of inputs per node. Kept small to fit in cache lines.
pub const MAX_INPUTS: usize = 8;

/// Maximum number of nodes in the arena. Pre-allocated at startup.
pub const MAX_NODES: usize = 10_240;

/// Maximum number of audio buffers in the pool.
pub const MAX_BUFFERS: usize = MAX_NODES * 2;

/// Maximum commands processed per audio callback. Bounds mutation cost.
pub const MAX_COMMANDS_PER_TICK: usize = 32;

/// Command ring buffer capacity.
pub const COMMAND_RING_CAPACITY: usize = 1024;
