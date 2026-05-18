# Instrument Format Specification

This document describes the JSON format for defining sample-based instruments in AetherDSP.

## Overview

Instruments are defined using JSON files that specify:

- Metadata (name, author, description)
- Tuning system (frequency table)
- Sample zones (which samples play for which notes/velocities)
- Envelope parameters (ADSR)
- Voice management settings

## File Structure

```json
{
  "name": "Instrument Name",
  "origin": "Cultural origin (e.g., Western, Ethiopian, Indian)",
  "description": "Detailed description of the instrument",
  "author": "Author name or organization",
  "tuning": { ... },
  "zones": [ ... ],
  "attack": 0.005,
  "decay": 0.1,
  "sustain": 0.8,
  "release": 0.3,
  "max_voices": 16
}
```

## Fields

### Metadata

| Field         | Type   | Required | Description                                       |
| ------------- | ------ | -------- | ------------------------------------------------- |
| `name`        | string | Yes      | Display name of the instrument                    |
| `origin`      | string | Yes      | Cultural or geographical origin                   |
| `description` | string | Yes      | Detailed description including source attribution |
| `author`      | string | Yes      | Creator or organization name                      |

### Tuning System

The `tuning` object defines the frequency table for the instrument.

```json
"tuning": {
  "name": "12-TET",
  "description": "Standard equal temperament (A4 = 440 Hz)",
  "frequencies": [8.18, 8.66, 9.18, ...]
}
```

| Field         | Type            | Required | Description                                                  |
| ------------- | --------------- | -------- | ------------------------------------------------------------ |
| `name`        | string          | Yes      | Tuning system name (e.g., "12-TET", "Tizita")                |
| `description` | string          | Yes      | Human-readable description                                   |
| `frequencies` | array of floats | Yes      | Frequency values in Hz (typically 128 values for MIDI range) |

**Standard Tuning Systems:**

- **12-TET**: 12-tone equal temperament (Western standard)
- **Tizita**: Ethiopian pentatonic scale
- **Just Intonation**: Pure integer ratios
- **Custom**: User-defined frequency tables

### Sample Zones

The `zones` array defines which samples play for which notes and velocities.

```json
"zones": [
  {
    "id": "c4-soft",
    "file_path": "piano-basic/c4-soft.wav",
    "root_note": 60,
    "note_low": 60,
    "note_high": 60,
    "velocity_low": 0,
    "velocity_high": 63,
    "articulation": "Sustained",
    "volume_db": 0.0,
    "tune_cents": 0.0,
    "release_file": null
  }
]
```

| Field           | Type           | Required | Description                                        |
| --------------- | -------------- | -------- | -------------------------------------------------- |
| `id`            | string         | Yes      | Unique identifier for this zone                    |
| `file_path`     | string         | Yes      | Relative path to WAV file (from `assets/samples/`) |
| `root_note`     | integer        | Yes      | MIDI note number of the original sample (0-127)    |
| `note_low`      | integer        | Yes      | Lowest MIDI note that triggers this zone (0-127)   |
| `note_high`     | integer        | Yes      | Highest MIDI note that triggers this zone (0-127)  |
| `velocity_low`  | integer        | Yes      | Lowest velocity that triggers this zone (0-127)    |
| `velocity_high` | integer        | Yes      | Highest velocity that triggers this zone (0-127)   |
| `articulation`  | string         | Yes      | Articulation type (see below)                      |
| `volume_db`     | float          | Yes      | Volume adjustment in dB (-∞ to +12)                |
| `tune_cents`    | float          | Yes      | Pitch adjustment in cents (-100 to +100)           |
| `release_file`  | string or null | No       | Optional release sample path                       |

**Articulation Types:**

- `"OneShot"`: Plays sample once (drums, percussion)
- `"Sustained"`: Loops until note off (piano, strings)
- `"Legato"`: Smooth transition between notes
- `"Staccato"`: Short, detached notes

### Envelope Parameters

ADSR envelope applied to all zones.

| Field     | Type  | Required | Description                           |
| --------- | ----- | -------- | ------------------------------------- |
| `attack`  | float | Yes      | Attack time in seconds (0.0 to 10.0)  |
| `decay`   | float | Yes      | Decay time in seconds (0.0 to 10.0)   |
| `sustain` | float | Yes      | Sustain level (0.0 to 1.0)            |
| `release` | float | Yes      | Release time in seconds (0.0 to 10.0) |

**Typical Values:**

- **Piano**: `attack: 0.005, decay: 0.1, sustain: 0.8, release: 0.3`
- **Drums**: `attack: 0.001, decay: 0.0, sustain: 1.0, release: 0.05`
- **Strings**: `attack: 0.1, decay: 0.2, sustain: 0.9, release: 0.5`
- **Brass**: `attack: 0.05, decay: 0.1, sustain: 0.85, release: 0.2`

### Voice Management

| Field        | Type    | Required | Description                            |
| ------------ | ------- | -------- | -------------------------------------- |
| `max_voices` | integer | Yes      | Maximum simultaneous voices (1 to 128) |

**Recommended Values:**

- **Monophonic**: `max_voices: 1`
- **Polyphonic (basic)**: `max_voices: 8-16`
- **Polyphonic (rich)**: `max_voices: 32-64`
- **Drums**: `max_voices: 32-64`

## Sample File Requirements

### Audio Format

- **Format**: WAV (PCM)
- **Bit Depth**: 16-bit or 24-bit
- **Sample Rate**: 44.1 kHz or 48 kHz
- **Channels**: Mono or Stereo

### File Organization

```
assets/
  samples/
    instrument-name/
      category/
        sample-name.wav
```

**Example:**

```
assets/
  samples/
    piano-basic/
      c3-soft.wav
      c3-loud.wav
      c4-soft.wav
      c4-loud.wav
```

### Licensing

All samples must be:

- **CC0** (Public Domain)
- **CC-BY** (Attribution required)
- **Original work** (with appropriate license)

**Attribution Format:**

```json
"description": "... Source: [Library Name] ([License]). Author: [Name]."
```

## Zone Mapping Strategies

### 1. Single Sample per Note (Drums)

```json
{
  "id": "kick",
  "root_note": 36,
  "note_low": 36,
  "note_high": 36,
  "velocity_low": 0,
  "velocity_high": 127
}
```

### 2. Velocity Layers (Piano)

```json
[
  {
    "id": "c4-soft",
    "root_note": 60,
    "note_low": 60,
    "note_high": 60,
    "velocity_low": 0,
    "velocity_high": 63
  },
  {
    "id": "c4-loud",
    "root_note": 60,
    "note_low": 60,
    "note_high": 60,
    "velocity_low": 64,
    "velocity_high": 127
  }
]
```

### 3. Key Ranges (Sampled every 3rd note)

```json
[
  {
    "id": "c3",
    "root_note": 48,
    "note_low": 47,
    "note_high": 49
  },
  {
    "id": "d#3",
    "root_note": 51,
    "note_low": 50,
    "note_high": 52
  }
]
```

### 4. Round Robin (Multiple samples per note)

```json
[
  {
    "id": "c4-rr1",
    "root_note": 60,
    "note_low": 60,
    "note_high": 60,
    "round_robin_group": 1
  },
  {
    "id": "c4-rr2",
    "root_note": 60,
    "note_low": 60,
    "note_high": 60,
    "round_robin_group": 1
  }
]
```

## Validation

Use the schema validator to check instrument definitions:

```rust
use aetherdsp_core::schema::SchemaValidator;

let validator = SchemaValidator::new();
let json = std::fs::read_to_string("assets/instruments/piano-basic.json")?;

match validator.validate_instrument(&json) {
    Ok(_) => println!("Valid instrument"),
    Err(errors) => {
        for error in errors {
            eprintln!("Error: {}", error);
        }
    }
}
```

## Examples

### Minimal Instrument (Single Sample)

```json
{
  "name": "Test Tone",
  "origin": "Synthetic",
  "description": "440 Hz sine wave for testing",
  "author": "AetherDSP",
  "tuning": {
    "name": "12-TET",
    "description": "Standard tuning",
    "frequencies": [...]
  },
  "zones": [
    {
      "id": "a4",
      "file_path": "test/sine-440.wav",
      "root_note": 69,
      "note_low": 0,
      "note_high": 127,
      "velocity_low": 0,
      "velocity_high": 127,
      "articulation": "Sustained",
      "volume_db": 0.0,
      "tune_cents": 0.0,
      "release_file": null
    }
  ],
  "attack": 0.01,
  "decay": 0.0,
  "sustain": 1.0,
  "release": 0.01,
  "max_voices": 1
}
```

### Full Instrument (Piano with Velocity Layers)

See `assets/instruments/piano-basic.json` for a complete example.

### Drum Kit

See `assets/instruments/drums-studio.json` for a complete example.

## Best Practices

### Performance

1. **Limit sample size**: Keep samples under 5 MB each
2. **Use appropriate sample rates**: 44.1 kHz is sufficient for most instruments
3. **Optimize zone count**: More zones = more memory usage
4. **Set reasonable max_voices**: Higher values increase CPU usage

### Quality

1. **Normalize samples**: Ensure consistent volume levels
2. **Remove DC offset**: Prevent clicks and pops
3. **Trim silence**: Remove leading/trailing silence
4. **Loop points**: For sustained instruments, set proper loop points

### Organization

1. **Consistent naming**: Use descriptive, lowercase names with hyphens
2. **Group by category**: Organize samples into logical folders
3. **Document sources**: Always attribute sample sources
4. **Version control**: Track changes to instrument definitions

## Tools

### Sample Preparation

- **Audacity**: Free audio editor (normalize, trim, export)
- **ffmpeg**: Command-line audio conversion
- **sox**: Command-line audio processing

### Validation

```bash
# Validate instrument JSON
cargo run --bin aether-cli validate-instrument assets/instruments/piano-basic.json

# List all zones
cargo run --bin aether-cli list-zones assets/instruments/piano-basic.json

# Test instrument playback
cargo run --bin aether-cli test-instrument assets/instruments/piano-basic.json
```

## License

This specification is released under CC0 (Public Domain).

Sample libraries must include their own license information in the `description` field.
