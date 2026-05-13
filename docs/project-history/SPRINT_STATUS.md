# Aether DSP - Sprint Status & Completion Plan

## Current State Analysis

The Iced UI (`crates/aether-ui/src/daw_app.rs`) is **significantly more complete** than initially thought. Most Sprint 1 features are already implemented!

---

## ✅ ALREADY IMPLEMENTED (Sprint 1)

### Song View

- ✅ Track height resize by dragging
- ✅ Right-click context menu on clips (rename, duplicate, delete, split)
- ✅ Clip resize from right edge (drag to extend/shrink)
- ✅ Multi-select with rubber-band drag selection
- ✅ Track rename (double-click with inline text input)
- ✅ Track reorder (move up/down via context menu)
- ✅ Zoom in/out (Ctrl+scroll implemented in code)
- ✅ Horizontal scroll
- ✅ Loop region display
- ✅ Playhead click-to-seek on ruler
- ✅ Clip name display inside clip
- ✅ Snap indicator with toolbar buttons
- ✅ Time display in bars (ruler shows bar numbers)
- ✅ Clip mini-note preview (waveform for MIDI)

### Piano Roll

- ✅ Velocity lane at the bottom (VelocityCanvas)
- ✅ Quantize button (snap existing notes to grid)
- ✅ Scale highlighting (scale selector with 7 scales)
- ✅ Note length selector (default_note_len in state)
- ✅ Zoom with Ctrl+scroll
- ✅ Scroll with mouse
- ✅ Transpose selected notes (Shift+Up/Down)
- ✅ Transpose octave (Ctrl+Up/Down)
- ✅ Note color by scale
- ✅ Rubber-band selection
- ✅ Draw/Select/Erase tools
- ✅ Piano keyboard with playback preview

### Mixer

- ✅ Working faders (wired to DSP via SetTrackVolume)
- ✅ Pan knobs wired to DSP
- ✅ VU meters showing real audio levels (vu_levels array)
- ✅ Mute/Solo buttons
- ✅ Color-coded channel strips
- ✅ Volume percentage display

### Transport

- ✅ Play/Stop/Record toggle
- ✅ BPM control
- ✅ Loop toggle
- ✅ Metronome toggle (metronome_on flag)
- ✅ Playhead animation (Tick message updates transport)
- ✅ Time signature support

### Core Features

- ✅ Undo/Redo system (push_undo in state)
- ✅ Keyboard shortcuts (Space, Ctrl+Z, Ctrl+Y, Ctrl+C/V/X, Delete, etc.)
- ✅ MIDI preview (notes trigger sound on draw/move)
- ✅ Track mute/solo/arm
- ✅ Instrument panel per track
- ✅ Effects chain (add/remove/toggle effects)

---

## ⚠️ MISSING / INCOMPLETE (Sprint 1 Gaps)

### Song View

- ❌ Track color picker UI (color is set but no picker widget)
- ❌ Clip waveform display for audio clips (only MIDI preview exists)
- ❌ Track type icons (instrument/audio/bus) - not displayed
- ❌ Time display in minutes:seconds (only bars:beats)
- ❌ Metronome click sound (flag exists but no audio implementation)
- ❌ Loop region drag handles (display only, not editable)
- ❌ Middle mouse scroll

### Piano Roll

- ❌ Pitch bend lane
- ❌ Modulation lane (CC automation)
- ❌ Humanize function (add timing/velocity variation)
- ❌ Chord mode (draw chords from root)
- ❌ Select all notes in pitch row (click on piano key)
- ❌ Velocity edit mode (click+drag on note to set velocity) - partially done
- ❌ Ghost notes (show notes from other clips dimly)
- ❌ Legato/staccato tools

### Mixer

- ❌ Peak hold on meters
- ❌ Send/return routing (reverb send, etc.)
- ❌ Insert effects slots with UI (effects exist but no visual chain)
- ❌ Group buses
- ❌ Master bus (separate from tracks)
- ❌ Pre/post fader sends
- ❌ Sidechain routing
- ❌ A/B comparison

---

## 🔴 SPRINT 2 - Sound Design (Critical Missing Features)

### Built-in Instruments

The biggest gap is **actual DSP-wired instruments**. Current state:

- ✅ Instrument presets exist (`InstrumentPreset` struct)
- ✅ Track engine exists (`TrackEngine` in master engine)
- ✅ MIDI routing works (notes trigger on preview)
- ❌ **No actual synth DSP nodes** (Aether Poly, Aether Drums, etc.)
- ❌ No instrument UI panels (only placeholder button)
- ❌ No preset browser
- ❌ No sample loading for drums

**Required:**

1. **Aether Poly** - Polyphonic subtractive synth
   - 3 oscillators (sine/saw/square/triangle)
   - 2 filters (Moog ladder + SVF)
   - 3 envelopes (amp, filter, pitch)
   - 3 LFOs
   - Arpeggiator
   - 128-voice polyphony

2. **Aether Drums** - Sample-based drum machine
   - 16 pads
   - Load WAV/AIFF samples
   - Per-pad: pitch, volume, pan, envelope, filter
   - Built-in step sequencer

3. **Aether Sampler** - Full sampler
   - Load any audio file
   - Pitch to MIDI
   - Loop points
   - Granular mode

### Effects DSP Wiring

- ✅ Effects stored in track state
- ❌ **No actual DSP processing** (EQ, Compressor, Reverb, Delay, Filter)
- ❌ No effect UI panels
- ❌ No bypass toggle (flag exists but not wired)
- ❌ No preset save/load

**Required:**

- EQ (8-band parametric with spectrum analyzer)
- Compressor (VCA/FET/optical modes, sidechain, GR meter)
- Limiter (true peak, lookahead)
- Reverb (plate/hall/room/spring + convolution)
- Delay (tempo-synced, ping-pong, tape)
- Chorus/Flanger/Phaser
- Distortion/Saturator
- Bitcrusher
- Transient shaper
- Stereo imager

---

## 🟡 SPRINT 3 - Recording & Files

### Audio Recording

- ❌ Arm track + record from mic/interface
- ❌ Waveform display after recording
- ❌ Punch in/out
- ❌ Overdub mode
- ❌ Take management
- ❌ Audio clip editing (trim, split, fade, normalize, reverse)

### Project Management

- ❌ Save/load project (full session state to file)
- ❌ Project file format (JSON-based)
- ❌ Auto-save
- ❌ Recent projects
- ❌ Export: WAV/FLAC/MP3 (stems + master mix)
- ❌ Import: WAV/AIFF/FLAC/MP3/MIDI

---

## 🟢 SPRINT 4 - Polish & Power Features

### Automation

- ❌ Automation clips in Song view
- ❌ Draw automation curves
- ❌ Automation modes (Read, Write, Touch, Latch)
- ❌ Curve types (linear, exponential, S-curve, step)
- ❌ Record automation in real time

### MIDI

- ❌ Hardware MIDI input (keyboard/controller)
- ❌ MIDI output to external gear
- ❌ MIDI clock sync (master/slave)
- ❌ MPE support
- ❌ MIDI learn (right-click knob → assign to CC)
- ❌ MIDI monitor

### Browser / Asset Management

- ❌ File browser (navigate to samples/presets)
- ❌ Drag samples from browser to track
- ❌ Preset browser for instruments and effects
- ❌ Search/filter
- ❌ Favorites
- ❌ Recent files

### Mastering View

- ❌ Dedicated mastering chain
- ❌ LUFS meter (integrated/short-term/momentary)
- ❌ Streaming platform targets (Spotify -14, Apple -16)
- ❌ A/B reference track
- ❌ Multiband compressor
- ❌ True peak limiter
- ❌ Dithering for 16/24-bit export

### Performance / Stability

- ❌ Stress testing (no audio glitches under load)
- ❌ Crash recovery (save project before panic)
- ❌ CPU meter
- ❌ Latency compensation
- ❌ ASIO support on Windows (currently WASAPI via CPAL)

---

## 🎯 PRIORITY COMPLETION ORDER

### Phase 1: Complete Sprint 1 Polish (1-2 days)

1. Add track color picker widget
2. Add time display in minutes:seconds
3. Implement metronome click sound
4. Add loop region drag handles
5. Add pitch bend lane to piano roll
6. Add modulation lane
7. Implement humanize function
8. Add chord mode
9. Add peak hold to VU meters
10. Add master bus to mixer

### Phase 2: Sprint 2 - Core Sound (1 week)

1. **Aether Poly synth** - Full DSP implementation
   - Oscillator nodes (sine/saw/square/triangle)
   - Filter nodes (Moog ladder + SVF)
   - Envelope generators (ADSR)
   - LFO nodes
   - Voice allocator (128 voices)
   - UI panel with all controls

2. **Aether Drums** - Sample playback engine
   - Sample loader (WAV/AIFF)
   - 16-pad architecture
   - Per-pad DSP chain
   - Step sequencer UI

3. **Effects DSP** - Wire all effects to audio graph
   - EQ node (8-band parametric)
   - Compressor node (VCA/FET/optical)
   - Reverb node (algorithmic + convolution)
   - Delay node (tempo-synced)
   - Filter node (LP/HP/BP/Notch)
   - UI panels for each effect

### Phase 3: Sprint 3 - Recording & Files (3-4 days)

1. Audio recording from interface
2. Waveform display
3. Save/load project (JSON format)
4. Export WAV/FLAC (stems + master)
5. Import audio files

### Phase 4: Sprint 4 - Automation & Polish (1 week)

1. Automation clips
2. Automation recording
3. Hardware MIDI input
4. Browser panel
5. Preset management
6. Mastering view
7. LUFS metering
8. Performance optimization

---

## 📊 COMPLETION ESTIMATE

- **Sprint 1 Polish**: 90% complete → 2 days to 100%
- **Sprint 2 (Sound)**: 20% complete → 1 week to 100%
- **Sprint 3 (Files)**: 5% complete → 4 days to 100%
- **Sprint 4 (Polish)**: 0% complete → 1 week to 100%

**Total time to production-ready**: ~3-4 weeks of focused development

---

## 🚀 IMMEDIATE NEXT STEPS

1. **Read the existing DSP node implementations** to understand the architecture
2. **Implement Aether Poly synth** as the first complete instrument
3. **Wire effects to the audio graph** (EQ, Compressor, Reverb, Delay)
4. **Add instrument UI panels** with knobs/sliders
5. **Test end-to-end**: Draw notes → Hear synth → Apply effects → Export audio

The foundation is **solid**. The UI is **90% complete**. The missing piece is **DSP wiring** for instruments and effects.
