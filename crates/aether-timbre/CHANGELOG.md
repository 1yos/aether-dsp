# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-12

### Added

- Comprehensive inline documentation for FFT-based analysis
- Usage examples for timbre transfer

### Fixed

- Phase alignment in overlap-add synthesis

## [0.1.0] - 2026-04-01

### Added

- Initial release of FFT-based spectral timbre analysis
- SpectralEnvelope extraction from audio samples
- TimbreProfile storage for multiple pitches/velocities
- TimbreTransfer node for real-time spectral envelope application
- InstrumentSynthesizer for generating synthetic samples
- Overlap-add synthesis for smooth spectral morphing

### Features

- FFT analysis with rustfft
- Spectral envelope smoothing
- Multi-pitch timbre profiling
- Real-time timbre transfer with low latency

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
