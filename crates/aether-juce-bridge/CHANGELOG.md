# Changelog

All notable changes to the AetherDSP JUCE Bridge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8] - 2026-06-15

### Fixed

- Documentation link on crates.io now correctly points to `docs.rs/aetherdsp-juce-bridge` instead of `docs.rs/aether-core`

## [0.1.7] - 2026-06-15

### Added

- `aether_tuning_ethiopian_tizita_minor()` - Tizita minor variant
- `aether_tuning_ethiopian_bati_major()` - Bati major variant
- `aether_tuning_ethiopian_anchihoye()` - Anchihoye mode
- `extern "C"` blocks in header for proper C++ compatibility
- Complete Ethiopian qenet system (all 7 traditional modes)

### Changed

- Increased total tuning systems from 13 to 17
- Updated documentation to reflect all 7 Ethiopian qenet modes
- Updated `aether_tuning_count()` to return 17
- Regenerated C header with all new functions and C++ compatibility
- Updated dependency to `aetherdsp-midi` 0.1.7

### Fixed

- C++ name mangling issues - added `extern "C"` guards to header
- JUCE 7.x compatibility verified with test plugin

**Total tuning systems: 17** (7 Ethiopian, 3 Arabic, 1 Indian, 3 Gamelan, 3 Western)

## [0.1.6] - 2026-06-04

### Added

- Initial release of AetherDSP JUCE Bridge
- C FFI API for integrating AetherDSP world music tuning systems with JUCE plugins
- Support for 13 world music tuning systems:
  - Ethiopian: Tizita major, Bati minor, Ambassel (3 systems)
  - Arabic: Rast, Bayati, Hijaz (3 systems)
  - Indian: Yaman (1 system)
  - Gamelan: Slendro, Slendro Stretched, Pelog (3 systems)
  - Western: Just Intonation (5-limit), Just Intonation (7-limit), 12-TET (3 systems)
- `aether_tuning_get_frequency()` - Get frequency for a single MIDI note
- `aether_tuning_get_all_frequencies()` - Get all 128 frequencies at once
- `aether_version()` - Get AetherDSP version string
- `aether_tuning_count()` - Get count of available tuning systems
- Automatic C header generation via cbindgen
- Comprehensive unit tests (5 tests, all passing)
- Documentation and examples

### Technical Details

- Zero-cost C FFI with no runtime overhead
- Memory-safe thanks to Rust implementation
- Thread-safe tuning table creation and queries
- Default concert A = 440 Hz for all tuning systems
- Support for static linking (`.a`/`.lib`) and dynamic linking (`.dylib`/`.so`/`.dll`)

[0.1.6]: https://github.com/1yos/aether-dsp/releases/tag/aetherdsp-juce-bridge-v0.1.6
