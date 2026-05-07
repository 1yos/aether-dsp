# Instrument Sound Quality Fixes

## Current Issues

### 1. Kick Drum - CRITICAL ❌

**Problem**: Uses fixed-pitch sine wave, sounds like a bass tone instead of a kick drum.

**Solution**: Add pitch envelope to oscillator

- Start frequency: 150Hz (C3)
- End frequency: 50Hz (G1)
- Envelope time: 80ms
- Curve: Exponential decay

**Alternative**: Use sampled kick from drums-studio kit (already available)

### 2. Pad Sound - Thin ⚠️

**Problem**: Single oscillator sounds thin and lifeless.

**Solutions**:

- Add unison (3-7 voices with slight detuning)
- Add chorus effect
- Increase filter resonance slightly
- Add subtle vibrato (LFO → pitch)

### 3. No Velocity Sensitivity ⚠️

**Problem**: All notes sound the same brightness regardless of velocity.

**Solution**: Map velocity → filter cutoff

- Low velocity (1-64): Darker sound (cutoff \* 0.5)
- High velocity (65-127): Brighter sound (cutoff \* 1.0)

### 4. Missing Modulation ⚠️

**Problem**: No LFO for vibrato, tremolo, or filter sweeps.

**Solution**: Add LFO node with:

- Waveforms: Sine, Triangle, Square, Random
- Rate: 0.1Hz - 20Hz
- Depth: 0-100%
- Destinations: Pitch, Filter, Amplitude

## Implementation Priority

### PHASE 1: Quick Wins (1-2 hours)

1. **Fix Kick Preset** - Add pitch envelope parameters
2. **Improve Pad Preset** - Better ADSR and filter settings
3. **Add Velocity → Filter** - Simple mapping in voice allocation

### PHASE 2: Enhanced Synthesis (3-4 hours)

4. **Add LFO Node** - New DSP node for modulation
5. **Add Unison** - Multiple detuned voices per note
6. **Add Pitch Envelope** - Dedicated envelope for frequency

### PHASE 3: Sampler Integration (6-8 hours)

7. **Connect Sampler** - Wire aether-sampler to UI
8. **Load Drum Kit** - Use existing drums-studio samples
9. **Load World Instruments** - Use existing presets
10. **Instrument Browser** - UI for selecting instruments

## Recommended Approach

**Option A: Quick Fix (Recommended for now)**

- Fix kick drum with better synthesis parameters
- Improve other presets
- Add velocity sensitivity
- **Time**: 1-2 hours
- **Result**: Decent sounding instruments

**Option B: Full Implementation**

- Add all missing features (LFO, unison, pitch envelope)
- Connect sampler system
- Add instrument browser
- **Time**: 10-15 hours
- **Result**: Professional-quality DAW

## Improved Preset Parameters

### Kick (Synthesized with Pitch Envelope)

```rust
// Oscillator
waveform: 0.0 (sine)
start_freq: 150.0 Hz
end_freq: 50.0 Hz
pitch_env_time: 0.08 seconds

// Amplitude Envelope
attack: 0.001
decay: 0.15
sustain: 0.0
release: 0.05

// Filter
cutoff: 120 Hz (very low)
resonance: 1.5 (boost sub)
mode: LP

gain: 1.0
```

### Bass (Already Good)

```rust
waveform: 1.0 (saw)
attack: 0.005
decay: 0.3
sustain: 0.6
release: 0.15
cutoff: 800 Hz
resonance: 1.2
gain: 0.75
```

### Lead (Add Brightness)

```rust
waveform: 2.0 (square)
attack: 0.01
decay: 0.08
sustain: 0.75
release: 0.2
cutoff: 4000 Hz (brighter)
resonance: 2.0 (more character)
gain: 0.7
```

### Pad (Improved)

```rust
waveform: 3.0 (triangle)
attack: 0.4 (slower swell)
decay: 0.6
sustain: 0.85 (higher sustain)
release: 1.2 (longer tail)
cutoff: 2500 Hz (slightly brighter)
resonance: 1.2 (more resonance)
gain: 0.6
```

## Sample-Based Instruments Available

### Drums (drums-studio kit)

- Kick, Snare, Snare Rim
- Hi-hats: Closed, Open, Pedal
- Toms: High, Mid, Floor
- Cymbals: Crash, Ride, Ride Bell

**Source**: VSCO 2 Community Edition (CC0 - Public Domain)
**Quality**: Professional studio recordings
**Status**: ✅ Samples exist, ❌ Not connected to UI

### World Instruments (60 instruments)

**Status**: ✅ Presets defined, ❌ Not connected to UI
**Synthesis**: Algorithmic (Karplus-Strong, formant filters)

## Next Steps

1. **Review this document** - Confirm approach
2. **Choose option** - Quick fix (A) or Full implementation (B)
3. **Implement fixes** - Start with highest priority items
4. **Test audio** - Verify sounds are correct
5. **Commit and push** - Update repository

## Questions to Answer

1. Do you want quick fixes now, or full implementation?
2. Should we use sampled kick drum or synthesized with pitch envelope?
3. Do you want the sampler system connected in this session?
4. Priority: Better synth sounds OR more instruments?
