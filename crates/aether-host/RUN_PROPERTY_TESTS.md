# Running Property-Based Tests for aether-host

## Property 7: Undo/Redo Round-Trip Restores Graph State

This test validates that applying a structural intent and then undoing it restores the graph to its exact prior state.

### Running the test

```bash
cd aether-dsp/crates/aether-host
cargo test prop_undo_round_trip -- --nocapture
```

### Test Details

- **Location**: `src/undo_stack.rs` in the `#[cfg(test)] mod property_7` module
- **Property**: Undo/redo round-trip restores graph state
- **Validates**: Requirements 3.3, 3.8, 3.10, 3.11, 3.12
- **Test Strategy**:
  - Generates random structural intents (AddNode, Connect, Disconnect)
  - Captures initial graph snapshot
  - Applies the intent and captures post-intent snapshot
  - Applies undo and captures post-undo snapshot
  - Asserts that post-undo snapshot matches initial snapshot

### Test Cases

1. **prop_undo_round_trip_add_node**: Tests undo/redo for AddNode intent
   - Generates random node types (Oscillator, Gain, StateVariableFilter, DelayLine, Mixer)
   - Verifies node count increases by 1 after add
   - Verifies node count returns to original after undo

2. **prop_undo_round_trip_connect**: Tests undo/redo for Connect intent
   - Creates two nodes (Oscillator and Gain)
   - Connects them with a random slot (0-3)
   - Verifies edge count increases by 1 after connect
   - Verifies edge count returns to original after undo

3. **prop_undo_round_trip_disconnect**: Tests undo/redo for Disconnect intent
   - Creates two connected nodes
   - Disconnects them
   - Verifies edge count decreases by 1 after disconnect
   - Verifies edge count returns to original after undo

### Test Parameters

- `node_type`: One of ["Oscillator", "Gain", "StateVariableFilter", "DelayLine", "Mixer"]
- `slot`: 0 to 3 (input slot index)
- Default iterations: 100 (proptest default)

### Known Issues

If you encounter linker errors related to paths with spaces (e.g., "Audio kernel"), this is a known issue with some Windows toolchains. Workarounds:

1. Move the project to a path without spaces
2. Use a different toolchain (e.g., MSVC instead of MinGW)
3. Update your MinGW/MSYS2 installation

### Implementation Notes

The test uses the actual `GraphManager` and `Scheduler` to ensure real-world behavior. It:

- Extracts snapshot data (nodes, edges, output_node_id) from Response::Snapshot
- Compares node counts, edge counts, and output node IDs
- Does not compare parameter values (as they may have default initialization differences)
- Focuses on structural graph state (topology) rather than parameter state

### Future Enhancements

Additional test cases could be added for:

- RemoveNode intent
- LoadPatch intent
- ClearGraph intent
- Multiple undo/redo cycles
- Redo after undo (Property 8)
