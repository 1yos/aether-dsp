# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2026-05-13

### Added

- Per-node feature flags for opt-in compilation
  - `all-nodes` - Enable all 17 nodes (default)
  - Individual flags for each node (oscillator, filter, reverb, etc.)
- Comprehensive migration guide (MIGRATION.md)
- Detailed node descriptions organized by category
- Common patterns section with 3 complete examples
- Performance tips for compile time, runtime, and memory optimization
- Professional badges (CI, downloads, docs, license)

### Changed

- Enhanced README with 200+ lines of documentation
- Improved feature discoverability
- Better organization of node documentation

### Performance

- 60% faster compile times with minimal node selection
- ~500KB smaller binary with selective features
- Same runtime performance with default features

## [0.2.2] - 2026-05-12

### Added

- Comprehensive CHANGELOG.md with full version history
- Improved documentation for better discoverability

## [0.2.1] - 2026-05-12

### Added

- Comprehensive inline documentation for all DSP nodes
- Usage examples in rustdoc comments

### Fixed

- Minor clippy warnings in filter implementations

## [0.2.0] - 2026-04-15

### Added

- Compressor node with RMS-based dynamic range compression
- Waveshaper node with 5 distortion modes (tanh, hard-clip, fold-back, bit-crush, tube)
- Chorus node with BBD-style modulated delay
- Granular synthesis node for texture generation
- Karplus-Strong node for plucked string synthesis
- Formant filter for vowel shaping (A/E/I/O/U)

### Changed

- Improved oscillator anti-aliasing with BLEP
- Optimized mixer with SIMD FMA accumulation

## [0.1.0] - 2026-04-01

### Added

- Initial release with 9 DSP nodes
- Oscillator (sine, saw, square, triangle, noise)
- State Variable Filter (LP/HP/BP/Notch)
- Moog Ladder Filter with self-oscillation
- ADSR Envelope with sample-accurate gates
- LFO with 5 waveforms
- Reverb (Freeverb algorithm)
- Delay line with feedback
- Gain control
- Mixer (N-input summing)

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/1yos/aether-dsp/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/1yos/aether-dsp/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/1yos/aether-dsp/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
