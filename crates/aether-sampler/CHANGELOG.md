# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.2.1] - 2026-05-12

### Added
- Comprehensive CHANGELOG.md with full version history
- Improved documentation for better discoverability


## [0.2.0] - 2026-04-15

### Added

- Round-robin sample playback for natural variation
- Velocity layers for dynamic expression
- ADSR envelope per voice
- Polyphonic voice management (up to 16 voices)

### Changed

- Improved sample loading performance with ArcSwap
- Optimized voice allocation algorithm

## [0.1.0] - 2026-04-01

### Added

- Initial release of polyphonic sampler
- Lock-free instrument loading with ArcSwap
- MIDI-driven sample triggering
- Multi-sample support with zone mapping
- WAV file loading with hound
- Voice stealing for polyphony management
- Sample interpolation (linear)

### Features

- Zero-allocation RT path
- Lock-free instrument swapping
- MIDI velocity mapping
- Pitch shifting via playback rate

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
