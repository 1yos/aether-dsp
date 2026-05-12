# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
