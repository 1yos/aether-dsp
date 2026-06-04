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
- **Tuning tables** — 17 microtonal tuning systems including all 7 Ethiopian qenet modes (Tizita major/minor, Bati minor/major, Ambassel, Anchihoye), Arabic maqam (Rast, Bayati, Hijaz), Indian raga (Yaman), Gamelan (Slendro, Slendro Stretched, Pelog), and Just Intonation (5-limit, 7-limit)
- **Event system** — typed MIDI events (NoteOn, NoteOff, CC, PitchBend, Clock)
- **JUCE Integration** — Use these tuning systems in C++ JUCE plugins via [`aetherdsp-juce-bridge`](../aether-juce-bridge)

## Tuning systems

```rust
use aether_midi::tuning::TuningTable;

// Ethiopian qenet (all 7 traditional modes)
let tizita = TuningTable::ethiopian_tizita();              // Tizita major
let tizita_minor = TuningTable::ethiopian_tizita_minor();  // Tizita minor
let bati = TuningTable::ethiopian_bati();                  // Bati minor (most common)
let bati_major = TuningTable::ethiopian_bati_major();      // Bati major
let ambassel = TuningTable::ethiopian_ambassel();          // Ambassel
let anchihoye = TuningTable::ethiopian_anchihoye();        // Anchihoye

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

// Standard Western
let tet = TuningTable::equal_temperament(440.0); // 12-TET

// Custom — provide your own frequencies
let custom = TuningTable::from_frequencies(vec![...], "My Scale", "Description");
```

## License

MIT — see [LICENSE](../../LICENSE)
