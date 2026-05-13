# Migration Guide

This guide helps you upgrade between major versions of `aetherdsp-core`.

---

## Table of Contents

- [v0.1.3 → v0.1.4 (Upcoming)](#v013--v014-upcoming)
- [v0.1.2 → v0.1.3](#v012--v013)
- [v0.1.1 → v0.1.2](#v011--v012)
- [v0.1.0 → v0.1.1](#v010--v011)

---

## v0.1.3 → v0.1.4 (Upcoming)

### Summary

Added optional feature flags for `parallel` and `serde` dependencies. No breaking changes - default features match v0.1.3 behavior.

### New Features

#### Feature Flags

You can now opt-out of heavy dependencies:

```toml
# Before (v0.1.3) - all dependencies included
[dependencies]
aetherdsp-core = "0.1.3"

# After (v0.1.4) - same behavior (default features)
[dependencies]
aetherdsp-core = "0.1.4"

# After (v0.1.4) - minimal build (no Rayon, no serde)
[dependencies]
aetherdsp-core = { version = "0.1.4", default-features = false, features = ["std"] }
```

**Available features:**

- `std` - Standard library support (required)
- `parallel` - Parallel node execution via Rayon (default)
- `serde` - Graph snapshot serialization (default)

#### Performance Impact

**Disabling `parallel`:**

- Falls back to sequential node execution
- Slower for large graphs (100+ nodes)
- Faster compile times (~40% reduction)
- Smaller binary (~200KB reduction)

**Example - Sequential execution:**

```rust
// Cargo.toml
aetherdsp-core = { version = "0.1.4", default-features = false, features = ["std"] }

// Your code - no changes needed!
let mut sched = Scheduler::new(48_000.0);
// ... same API, sequential execution
```

**Disabling `serde`:**

- Removes `GraphSnapshot`, `NodeSnapshot`, `EdgeSnapshot` serialization
- Faster compile times (~10% reduction)
- Smaller binary (~50KB reduction)

**Example - No serialization:**

```rust
// Cargo.toml
aetherdsp-core = { version = "0.1.4", default-features = false, features = ["std", "parallel"] }

// GraphSnapshot is not available
// use aether_core::command::GraphSnapshot; // ❌ Compile error
```

### Migration Steps

**No migration needed!** Default features match v0.1.3 behavior.

**Optional optimization:**

If you don't need parallel execution or serialization:

```toml
[dependencies]
aetherdsp-core = { version = "0.1.4", default-features = false, features = ["std"] }
```

Benefits:

- 40% faster compile times
- 200KB smaller binary
- Same API, sequential execution

### Breaking Changes

**None.** This is a backward-compatible release.

---

## v0.1.2 → v0.1.3

### Summary

Documentation-only release. Added comprehensive inline API docs and 53 doc test examples. No code changes.

### New Features

- 35 APIs fully documented with examples
- 53 working doc tests
- Improved docs.rs appearance

### Migration Steps

**No migration needed.** Update your `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.3"
```

### Breaking Changes

**None.**

---

## v0.1.1 → v0.1.2

### Summary

Documentation improvements and working examples. No API changes.

### New Features

- CHANGELOG.md added
- Working examples in `examples/` directory
- Improved README

### Migration Steps

**No migration needed.** Update your `Cargo.toml`:

```toml
[dependencies]
aetherdsp-core = "0.1.2"
```

### Breaking Changes

**None.**

---

## v0.1.0 → v0.1.1

### Summary

Added parallel execution, property-based tests, and fixed critical bugs. Some API changes required.

### Breaking Changes

#### 1. Scheduler API Split

The `process()` method was split into two methods for better control.

**Before (v0.1.0):**

```rust
let mut sched = Scheduler::new(48_000.0);
let mut output = vec![0.0f32; 128];

// Single method for everything
sched.process(&mut output);
```

**After (v0.1.1):**

```rust
use ringbuf::{HeapRb, traits::Split};
use aether_core::command::Command;

let mut sched = Scheduler::new(48_000.0);
let (mut producer, mut consumer) = HeapRb::<Command>::new(1024).split();
let mut output = vec![0.0f32; 128];

// Option 1: With command ring (recommended for RT audio)
sched.process_block(&mut consumer, &mut output);

// Option 2: Without command ring (for Arc<Mutex<>> sharing)
sched.process_block_simple(&mut output);
```

**Reason:** Separated command draining for better control and flexibility.

**Migration:**

If you're using `Arc<Mutex<Scheduler>>`:

```rust
// Before
let sched = Arc::new(Mutex::new(Scheduler::new(48_000.0)));
sched.lock().unwrap().process(&mut output);

// After
let sched = Arc::new(Mutex::new(Scheduler::new(48_000.0)));
sched.lock().unwrap().process_block_simple(&mut output);
```

If you're using SPSC ring:

```rust
// Before
sched.process(&mut output);

// After
sched.process_block(&mut consumer, &mut output);
```

#### 2. Arena Generation Handling

Generations are now explicit in `NodeId`.

**Before (v0.1.0):**

```rust
// Direct slot access (unsafe!)
let record = &arena.slots[node_id.index];
```

**After (v0.1.1):**

```rust
// Safe generation-checked access
if let Some(record) = arena.get(node_id) {
    // Use record
}
```

**Reason:** Prevents use-after-free bugs from stale node references.

**Migration:**

Replace all direct slot access with `arena.get()`:

```rust
// ❌ Wrong (v0.1.0)
let record = &self.arena.slots[id.index];

// ✅ Correct (v0.1.1)
if let Some(record) = self.arena.get(id) {
    // Use record
}
```

#### 3. MAX_NODES Increased

`MAX_NODES` increased from 1,024 to 10,240.

**Before (v0.1.0):**

```rust
const MAX_NODES: usize = 1024;
```

**After (v0.1.1):**

```rust
const MAX_NODES: usize = 10240;
```

**Impact:** Larger graphs supported, slightly more memory usage (~2MB).

**Migration:** No code changes needed. Graphs with >1024 nodes now work.

### New Features

#### Parallel Execution

Nodes at the same BFS level now execute in parallel via Rayon.

```rust
// No code changes needed - automatic parallel execution!
let mut sched = Scheduler::new(48_000.0);

// Add 100 nodes
for _ in 0..100 {
    let id = sched.graph.add_node(Box::new(MyNode::new())).unwrap();
}

// Processes nodes in parallel automatically
sched.process_block_simple(&mut output);
```

**Performance:** 3-4× faster for large graphs on multi-core CPUs.

#### Property-Based Tests

Added proptest-based tests for correctness guarantees.

```rust
// Run property tests (requires proptest feature)
cargo test --features proptest
```

### Bug Fixes

#### 1. Generation Mismatch

**Issue:** Stale `NodeId` references could access wrong nodes.

**Fix:** All arena access now checks generation.

**Example:**

```rust
let id = graph.add_node(Box::new(Osc::new())).unwrap();
graph.remove_node(id);
let new_id = graph.add_node(Box::new(Filter::new())).unwrap();

// v0.1.0: Could access wrong node if new_id.index == id.index
// v0.1.1: Returns None (generation mismatch)
if let Some(record) = graph.arena.get(id) {
    // Safe - only succeeds if generation matches
}
```

#### 2. Buffer Pool Exhaustion

**Issue:** Rapid add/remove cycles could exhaust buffer pool.

**Fix:** Buffers now properly released on node removal.

**Example:**

```rust
// v0.1.0: Could fail after ~1000 iterations
// v0.1.1: Works indefinitely
for _ in 0..10_000 {
    let id = graph.add_node(Box::new(Osc::new())).unwrap();
    graph.remove_node(id);
}
```

### Migration Checklist

- [ ] Replace `sched.process()` with `sched.process_block()` or `sched.process_block_simple()`
- [ ] Replace direct arena slot access with `arena.get()`
- [ ] Test with graphs >1024 nodes (now supported)
- [ ] Run tests to verify no generation mismatches
- [ ] Benchmark parallel execution performance

### Testing Your Migration

```bash
# Run all tests
cargo test -p aetherdsp-core

# Run property tests
cargo test -p aetherdsp-core --features proptest

# Run benchmarks
cargo bench -p aetherdsp-core
```

---

## Need Help?

- **Documentation:** https://docs.rs/aetherdsp-core
- **Examples:** `crates/aether-core/examples/`
- **Issues:** https://github.com/1yos/aether-dsp/issues
- **Discussions:** https://github.com/1yos/aether-dsp/discussions

---

## Version History

| Version | Release Date | Type          | Summary                         |
| ------- | ------------ | ------------- | ------------------------------- |
| 0.1.4   | TBD          | Minor         | Feature flags (backward compat) |
| 0.1.3   | 2026-05-13   | Documentation | API docs + 53 examples          |
| 0.1.2   | 2026-05-12   | Documentation | CHANGELOG + examples            |
| 0.1.1   | 2026-05-12   | Minor         | Parallel execution + bug fixes  |
| 0.1.0   | 2026-04-01   | Major         | Initial release                 |
