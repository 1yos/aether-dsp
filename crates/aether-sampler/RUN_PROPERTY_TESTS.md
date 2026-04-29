# Running Property-Based Tests for aether-sampler

## Property 13: Sequential Round-Robin Full-Cycle

This test validates that Sequential mode cycles through all zones exactly once per N note-ons.

### Running the test

```bash
cd aether-dsp/crates/aether-sampler
cargo test prop_sequential_round_robin_full_cycle -- --nocapture
```

### Test Details

- **Location**: `src/instrument.rs` in the `#[cfg(test)]` module
- **Property**: Sequential round-robin full-cycle
- **Validates**: Requirements 6.4, 6.11
- **Test Strategy**:
  - Generates random N (1-16) zones
  - Creates a ZoneGroup with Sequential mode
  - Fires N note-ons and collects selected zone indices
  - Asserts each zone index appears exactly once

### Test Parameters

- `n`: 1 to 16 zones
- Default iterations: 100 (proptest default)

---

## Property 14: RandomNoRepeat Never Repeats Consecutively

This test validates that RandomNoRepeat mode never selects the same zone twice in a row.

### Running the test

```bash
cd aether-dsp/crates/aether-sampler
cargo test prop_random_no_repeat_no_consecutive_repeats -- --nocapture
```

### Test Details

- **Location**: `src/instrument.rs` in the `#[cfg(test)]` module
- **Property**: RandomNoRepeat never repeats consecutively
- **Validates**: Requirements 6.6, 6.12
- **Test Strategy**:
  - Generates random N (2-16) zones and random seed
  - Creates a RoundRobinState with the seed
  - Fires 100 note-ons via `rr_state.select` with RandomNoRepeat mode
  - Asserts no two adjacent selections are the same zone index

### Test Parameters

- `n`: 2 to 16 zones
- `seed`: Random u64 seed for RNG
- Default iterations: 100 (proptest default)

### Implementation Notes

The test uses `rr_state.select(0, n, &RoundRobinMode::RandomNoRepeat)` which:

- Takes group index 0 (arbitrary for this test)
- Takes group length n
- Uses RandomNoRepeat mode which guarantees no consecutive repeats

The test fires 100 note-ons to ensure the property holds across many selections, not just a few.

---

## Property 15: Backward-Compatible Instrument Loading

This test validates that legacy instrument JSON files load correctly and behave identically to the pre-upgrade implementation.

### Running the test

```bash
cd aether-dsp/crates/aether-sampler
cargo test prop_backward_compatible_instrument_loading -- --nocapture
```

### Test Details

- **Location**: `src/instrument.rs` in the `#[cfg(test)]` module
- **Property**: Backward-compatible instrument loading
- **Validates**: Requirements 6.7, 6.8
- **Test Strategy**:
  - Generates random legacy instrument JSON (flat zones array, no zone_groups)
  - Deserializes the JSON
  - Asserts each zone is wrapped in a ZoneGroup of size 1 with Sequential mode
  - Asserts `find_zone_rr` returns the same zone as legacy `find_zone` for any note/velocity

### Test Parameters

- `zone_count`: 1 to 10 zones
- `note_ranges`: Random note ranges for each zone
- `velocity_ranges`: Random velocity ranges for each zone
- Default iterations: 100 (proptest default)

---

## Known Issues

If you encounter linker errors related to paths with spaces (e.g., "Audio kernel"), this is a known issue with some Windows toolchains. Workarounds:

1. Move the project to a path without spaces
2. Use a different toolchain (e.g., MSVC instead of MinGW)
3. Update your MinGW/MSYS2 installation

## Running All Property Tests

To run all property-based tests in aether-sampler:

```bash
cd aether-dsp/crates/aether-sampler
cargo test prop_ -- --nocapture
```

This will run all tests whose names start with `prop_`.
