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

- Comprehensive inline documentation for macro internals
- Better error messages for invalid attribute syntax

### Fixed

- Macro hygiene with fully qualified paths
- Parameter attribute parsing edge cases

## [0.1.0] - 2026-04-01

### Added

- Initial release of #[aether_node] procedural macro
- Automatic Default impl generation
- AetherNodeMeta trait implementation
- Parameter attribute parsing (#[param(...)])
- PARAM_COUNT constant generation
- Attribute stripping for clean output

### Features

- Compile-time parameter validation
- Type-safe parameter definitions
- Minimal boilerplate for node creation
- Integration with syn and quote

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
