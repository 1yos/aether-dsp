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

- Comprehensive inline documentation for #[aether_node] macro
- Three working examples (tremolo, bitcrusher, registry_demo)
- Detailed usage guide in README

### Fixed

- Macro hygiene issues with parameter attribute parsing

## [0.1.0] - 2026-04-01

### Added

- Initial release of Node Development Kit
- `#[aether_node]` procedural macro for easy node creation
- Automatic Default impl generation from parameter defaults
- AetherNodeMeta trait for runtime introspection
- Parameter definition system with min/max/default values
- DspProcess trait for RT-safe processing
- Integration with aetherdsp-core and aetherdsp-nodes

### Features

- Zero-boilerplate node creation
- Compile-time parameter validation
- RT-safety enforcement through trait design
- State capture/restore support

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
