# Running Property-Based Tests for aether-nodes

## Property 3: RecordNode Pass-Through

This test validates that RecordNode passes audio through unchanged while recording to the ring buffer.

### Running the test

```bash
cd aether-dsp/crates/aether-nodes
cargo test prop_record_node_pass_through -- --nocapture
```

### Test Details

- **Location**: `src/record.rs` in the `#[cfg(test)]` module
- **Property**: RecordNode pass-through
- **Validates**: Requirements 2.11
- **Test Strategy**:
  - Generates random `[f32; 64]` arrays with values in range [-1.0, 1.0]
  - Creates a RecordNode with a ring buffer
  - Processes the input through the node
  - Asserts output buffer equals input buffer exactly

### Test Parameters

- `input_samples`: 64 random f32 values in range [-1.0, 1.0]
- Default iterations: 100 (proptest default)

### Implementation Notes

The test verifies that RecordNode maintains unity gain pass-through regardless of:

- Input signal amplitude
- Ring buffer state (full or not)
- Sample values (positive, negative, zero)

This ensures RecordNode can be inserted inline in any signal chain without affecting the audio.

---

## Property 9: ScopeNode Pass-Through

This test validates that ScopeNode passes audio through unchanged while sending samples to the ring buffer for visualization.

### Running the test

```bash
cd aether-dsp/crates/aether-nodes
cargo test prop_scope_node_pass_through -- --nocapture
```

### Test Details

- **Location**: `src/scope.rs` in the `#[cfg(test)]` module
- **Property**: ScopeNode pass-through
- **Validates**: Requirements 4.4
- **Test Strategy**:
  - Generates random `[f32; 64]` arrays with values in range [-1.0, 1.0]
  - Creates a ScopeNode with a ring buffer
  - Processes the input through the node
  - Asserts output buffer equals input buffer exactly

### Test Parameters

- `input_samples`: 64 random f32 values in range [-1.0, 1.0]
- Default iterations: 100 (proptest default)

### Implementation Notes

The test verifies that ScopeNode maintains unity gain pass-through regardless of:

- Input signal amplitude
- Ring buffer state (full or not)
- Sample values (positive, negative, zero)

This ensures ScopeNode can be inserted inline in any signal chain without affecting the audio.

---

## Property 10: Scope Frame Serialization

This test validates that scope frames can be correctly serialized to binary format for WebSocket transmission and deserialized back.

### Running the test

```bash
cd aether-dsp/crates/aether-nodes
cargo test prop_scope_frame_serialization -- --nocapture
```

### Test Details

- **Location**: `src/scope.rs` in the `#[cfg(test)]` module
- **Property**: Scope frame serialization
- **Validates**: Requirements 4.7
- **Test Strategy**:
  - Generates random `[f32; 64]` arrays with values in range [-1.0, 1.0]
  - Serializes to 256 bytes (64 × `f32::to_le_bytes()`)
  - Deserializes back to `[f32; 64]` using `f32::from_le_bytes()`
  - Asserts byte length is exactly 256
  - Asserts round-trip equality (deserialized values match original)

### Test Parameters

- `samples`: 64 random f32 values in range [-1.0, 1.0]
- Default iterations: 100 (proptest default)

### Implementation Notes

The test verifies the binary serialization format used for WebSocket transmission:

- Each f32 is serialized as 4 bytes using little-endian IEEE 754 format
- Total frame size is exactly 256 bytes (64 samples × 4 bytes)
- Round-trip serialization/deserialization preserves exact bit patterns
- No data loss or corruption during the conversion process

This ensures the WebSocket binary protocol correctly transmits scope data from the audio thread to the UI.

---

## Known Issues

If you encounter linker errors related to paths with spaces (e.g., "Audio kernel"), this is a known issue with some Windows toolchains. Workarounds:

1. Move the project to a path without spaces
2. Use a different toolchain (e.g., MSVC instead of MinGW)
3. Update your MinGW/MSYS2 installation
