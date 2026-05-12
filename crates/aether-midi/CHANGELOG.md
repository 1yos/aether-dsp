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

- Comprehensive inline documentation for tuning systems
- Examples demonstrating Ethiopian, Arabic, and Indian scales
- Detailed README with tuning table usage

### Fixed

- Tuning table interpolation for non-standard scales

## [0.1.0] - 2026-04-01

### Added

- Initial release with MIDI engine and tuning system support
- 14 tuning systems including:
  - 12-TET (standard equal temperament)
  - Ethiopian scales (Tizita, Bati, Ambassel, Anchihoye)
  - Arabic maqam (Rast, Bayati, Hijaz, Saba)
  - Indian raga (Yaman, Bhairav, Todi)
  - Gamelan (Slendro, Pelog)
  - Just Intonation, Western Pentatonic, Chromatic
- MIDI device routing with midir integration
- MIDI clock synchronization
- Typed MIDI events (NoteOn, NoteOff, CC, PitchBend, Clock)
- MidiEngine for event processing and routing

### Features

- Microtonal support with custom frequency tables
- Real-time tuning table switching
- MIDI-to-frequency conversion with tuning awareness
- Device enumeration and connection management

[Unreleased]: https://github.com/1yos/aether-dsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/1yos/aether-dsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/1yos/aether-dsp/releases/tag/v0.1.0
