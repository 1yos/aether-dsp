# aether-midi

[![crates.io](https://img.shields.io/crates/v/aether-midi.svg)](https://crates.io/crates/aether-midi)
[![docs.rs](https://docs.rs/aether-midi/badge.svg)](https://docs.rs/aether-midi)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

MIDI engine for [AetherDSP](https://crates.io/crates/aether-core) — device routing, clock sync, and microtonal tuning.

## Features

- **MIDI device routing** — connect hardware controllers to graph nodes
- **MIDI clock** — sync BPM to external MIDI clock source
- **Tuning tables** — full microtonal support including Ethiopian scales (Tizita, Bati, Anchihoye), Arabic maqam, and custom frequency maps
- **Event system** — typed MIDI events (NoteOn, NoteOff, CC, PitchBend, Clock)

## Tuning systems

```rust
use aether_midi::tuning::TuningTable;

// Ethiopian Tizita scale
let tizita = TuningTable::ethiopian_tizita();

// Arabic Maqam Rast
let rast = TuningTable::arabic_maqam_rast();

// Custom — provide your own frequency ratios
let custom = TuningTable::from_ratios(&[1.0, 1.125, 1.25, 1.333, 1.5, 1.667, 1.875]);
```

## License

MIT — see [LICENSE](../../LICENSE)
