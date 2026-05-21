# aether-midi

[![crates.io](https://img.shields.io/crates/v/aether-midi.svg)](https://crates.io/crates/aether-midi)
[![docs.rs](https://docs.rs/aether-midi/badge.svg)](https://docs.rs/aether-midi)
[![CI](https://github.com/1yos/aether-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/1yos/aether-dsp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Downloads](https://img.shields.io/crates/d/aether-midi.svg)](https://crates.io/crates/aether-midi)

MIDI engine for [AetherDSP](https://crates.io/crates/aether-core) — device routing, clock sync, and microtonal tuning.

## Features

- **MIDI device routing** — connect hardware controllers to graph nodes
- **MIDI clock** — sync BPM to external MIDI clock source
- **Tuning tables** — 13 microtonal tuning systems including Ethiopian scales (Tizita, Bati, Ambassel), Arabic maqam (Rast, Bayati, Hijaz), Indian raga, Gamelan (Slendro, Pelog), and Just Intonation (5-limit, 7-limit)
- **Event system** — typed MIDI events (NoteOn, NoteOff, CC, PitchBend, Clock)

## Tuning systems

```rust
use aether_midi::tuning::TuningTable;

// Ethiopian scales
let tizita = TuningTable::ethiopian_tizita();
let bati = TuningTable::ethiopian_bati();
let ambassel = TuningTable::ethiopian_ambassel();

// Arabic maqam
let rast = TuningTable::arabic_maqam_rast();
let bayati = TuningTable::arabic_maqam_bayati();
let hijaz = TuningTable::arabic_maqam_hijaz();

// Indian raga
let yaman = TuningTable::indian_raga_yaman();

// Gamelan
let slendro = TuningTable::gamelan_slendro();
let slendro_stretched = TuningTable::gamelan_slendro_stretched(); // Ethnomusicologically accurate
let pelog = TuningTable::gamelan_pelog();

// Just Intonation
let ji_5 = TuningTable::just_intonation(); // 5-limit (traditional)
let ji_7 = TuningTable::just_intonation_7_limit(); // 7-limit (blues, barbershop)

// Custom — provide your own frequencies
let custom = TuningTable::from_frequencies(vec![...], "My Scale", "Description");
```

## License

MIT — see [LICENSE](../../LICENSE)
