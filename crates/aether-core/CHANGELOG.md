# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-05-13

### Added

- Optional feature flags for reduced compile times and binary size
  - `std` - Standard library support (required)
  - `parallel` - Parallel node execution via Rayon (default)
  - `serde` - Graph snapshot serialization (default)
- Sequential execution fallback when `parallel` feature is disabled
- Comprehensive migration guide (MIGRATION.md)
- Performance characteristics table in README
- Comparison with other DSP engines (dasp, fundsp, cpal)
- Common pitfalls section with 6 examples
- Comprehensive FAQ with 25+ questions
- Professional badges (CI, downloads, docs, license)

### Changed

- Made `rayon` and `serde` optional dependencies
- Enhanced README with 350+ lines of documentation
- Improved feature discoverability

### Performance

- 40% faster compile times with minimal features
- ~200KB smaller binary without Rayon
- Same runtime performance with default features

## [0.1.3] - 2026-05-13

### Added

- Comprehensive inline API documentation for all 35 critical APIs
- 53 working doc test examples demonstrating API usage
- Detailed documentation for:
  - Scheduler module (4 APIs)
  - DspGraph module (7 APIs)
  - DspNode trait (4 APIs)
  - Param module (9 APIs)
  - Arena module (7 APIs)
  - BufferPool module (4 APIs)
  - Command enum (all 8 variants)
  - NodeRecord struct
- Real-time safety notes for all RT-critical functions
- Performance characteristics documentation
- Cross-references between related APIs

### Improved

- docs.rs appearance with comprehensive examples
- API discoverability for new users
- Onboarding experience with working code samples

## [0.1.2] - 2026-05-12

### Added

- Comprehensive CHANGELOG.md with full version history
- Working examples demonstrating real-world usage
- Improved documentation for better discoverability

## [0.1.1] - 2026-05-12

### Added

- Parallel BFS level execution with Rayon for multi-core DSP processing
- Property-based tests for scheduler equivalence and topological ordering
- Comprehensive inline documentation for public APIs

### Fixed

- Generation mismatch in arena lookups causing stale node references
- Buffer pool exhaustion on rapid add/remove cycles

### Changed

- Increased MAX_NODES from 1,024 to 10,240 for larger graphs
- Improved scheduler performance with optimized parallel dispatch

## [0.1.0] - 2026-04-01

### Added

- Initial release of aetherdsp-core
- Lock-free real-time scheduler with SPSC command ring
- Generational arena for safe node storage
- Pre-allocated buffer pool for zero-allocation RT path
- Topological sorting with Kahn's algorithm
- Parameter smoothing with per-sample interpolation
- Command system for RT-safe graph mutations
- DspNode trait for custom processing nodes
- State capture/restore for graph continuity

### Performance

- param_fill_buffer_64: 51.7 ns
- Arena insert/remove ×1000: < 5 µs
- Scheduler (1000 noop nodes): < 100 µs

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/1yos/aether-dsp/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/1yos/aether-dsp/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/1yos/aether-dsp/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
