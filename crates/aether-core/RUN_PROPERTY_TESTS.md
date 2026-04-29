# Running Property-Based Tests for aether-core

## Property 1: Parallel ≡ Sequential Execution

This test validates that the parallel Rayon scheduler produces bit-identical output to the sequential scheduler.

### Running the test

```bash
cd aether-dsp/crates/aether-core
cargo test prop_parallel_equiv_sequential -- --nocapture
```

### Test Details

- **Location**: `src/scheduler.rs` in the `#[cfg(test)]` module
- **Property**: Parallel execution is output-equivalent to sequential execution
- **Validates**: Requirements 1.1, 1.4
- **Test Strategy**:
  - Generates random DAG patches (1-20 nodes, random edges)
  - Creates two identical schedulers with the same nodes and connections
  - Processes one audio block with both parallel and sequential implementations
  - Asserts bit-identical output buffers

### Test Parameters

- `num_nodes`: 1 to 20 nodes
- `edges`: 0 to 50 random edge connections (filtered to maintain DAG invariant)
- `seed`: Random seed for deterministic node gains
- Default iterations: 100 (proptest default)

### Known Issues

If you encounter linker errors related to paths with spaces (e.g., "Audio kernel"), this is a known issue with some Windows toolchains. Workarounds:

1. Move the project to a path without spaces
2. Use a different toolchain (e.g., MSVC instead of MinGW)
3. Update your MinGW/MSYS2 installation

### Implementation Notes

The test uses a `TestNode` that sums all inputs and multiplies by a deterministic gain. This provides:

- Deterministic behavior (no randomness in DSP processing)
- Non-trivial signal flow (multiple inputs can be summed)
- Verification that parallel dispatch correctly handles input/output buffer aliasing
