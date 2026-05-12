# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-12

### Added

- Comprehensive inline documentation for registry API
- Usage examples for node registration and lookup

### Fixed

- Thread safety in concurrent node registration

## [0.1.0] - 2026-04-01

### Added

- Initial release of runtime node type registry
- NodeRegistry for dynamic node type management
- Node factory pattern for runtime instantiation
- Type-safe node lookup by name
- Category-based node filtering
- Manifest integration for metadata

### Features

- Thread-safe registration with RwLock
- Dynamic node creation from string identifiers
- Metadata querying (parameters, ports, categories)
- Built-in node discovery

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
