# Sampler Integration & Enhanced Synthesis Plan

## Current Status

### ✅ What Exists

- **aether-sampler crate**: Full sample playback engine with:
  - Multi-sample support (velocity layers, round-robin)
  - Pitch shifting and time stretching
  - ADSR envelope per sample
  - Polyphonic voice management
- **Sample Assets**:
  - `assets/instruments/drums-studio.json` - 12-piece drum kit (CC0 licensed)
  - `assets/presets/world/*.json` - 60 world instrument definitions
  - Sample manifest system for loading

- **Synthesis Engine**:
  - Polyphonic subtractive synth (8 voices per track)
  - Band-limited oscillators (sine, saw, square, triangle)
  - State-variable filter (LP/HP/BP)
  - ADSR envelope
  - Effects chain (EQ, Compressor, Reverb, Delay, Filter)

### ❌ What's Missing

- Sampler is NOT connected to UI track system
- No instrument browser/selector in UI
- No way to switch between synth and sampler per track
- Missing synthesis features: LFO, unison, pitch envelope
- No velocity → filter mapping

## Integration Architecture

### Option 1: Hybrid Track System (Recommended)

Each track can be either:

- **Synth Track**: Uses current polyphonic synth
- **Sampler Track**: Uses aether-sampler for sample playback

```rust
pub enum TrackInstrument {
    Synth(InstrumentPreset),
    Sampler {
        instrument_id: String,  // e.g., "drums-studio", "krar"
        sampler_engine: SamplerEngine,
    },
}

pub struct Track {
    // ... existing fields ...
    instrument_type: TrackInstrument,
}
```

### Option 2: Unified Engine

Merge sampler into synth engine as another oscillator type:

- Waveform 0-3: Synth (sine, saw, square, tri)
- Waveform 4+: Sample playback

**Pros**: Simpler architecture
**Cons**: Loses flexibility, harder to manage sample loading

## Implementation Steps

### Phase 1: Connect Sampler to Audio Graph (4-6 hours)

#### 1.1 Add Sampler to TrackEngine

```rust
// in instrument.rs
pub enum InstrumentType {
    Synth { preset: InstrumentPreset },
    Sampler { engine: SamplerEngine },
}

impl TrackEngine {
    pub fn build_synth(...) -> Option<Self> { /* existing code */ }

    pub fn build_sampler(
        sched: &mut Scheduler,
        instrument_def: InstrumentDefinition,
        master_mixer_id: NodeId,
        master_slot: usize,
    ) -> Option<Self> {
        // Create sampler node
        let sampler_id = sched.graph.add_node(Box::new(SamplerNode::new(instrument_def)))?;

        // Connect to master
        sched.graph.connect(sampler_id, master_mixer_id, master_slot);

        // Return TrackEngine with sampler
        Some(Self { /* ... */ })
    }
}
```

#### 1.2 Create SamplerNode DSP Node

```rust
// in aether-sampler/src/node.rs
pub struct SamplerNode {
    engine: SamplerEngine,
    voices: Vec<SamplerVoice>,
}

impl DspNode for SamplerNode {
    fn process(&mut self, ...) {
        // Process all active voices
        // Mix to output buffer
    }
}
```

#### 1.3 Load Instrument Definitions

```rust
// in app_state.rs
pub struct InstrumentLibrary {
    drums: HashMap<String, InstrumentDefinition>,
    world: HashMap<String, InstrumentDefinition>,
}

impl InstrumentLibrary {
    pub fn load_from_assets() -> Result<Self, Error> {
        // Load drums-studio.json
        // Load world presets
        // Parse and validate
    }
}
```

### Phase 2: UI Integration (3-4 hours)

#### 2.1 Instrument Browser Panel

```rust
// New UI panel for selecting instruments
pub struct InstrumentBrowser {
    categories: Vec<InstrumentCategory>,
    selected: Option<String>,
    search_query: String,
}

enum InstrumentCategory {
    Synth,
    Drums,
    World { region: String },
}
```

#### 2.2 Track Instrument Selector

Add to track header:

- Button to open instrument browser
- Display current instrument name
- Icon showing synth vs sampler

#### 2.3 Update Messages

```rust
pub enum Message {
    // ... existing ...

    // Instrument selection
    OpenInstrumentBrowser(u64),  // track_id
    SelectInstrument { track_id: u64, instrument_id: String },
    SearchInstruments(String),
}
```

### Phase 3: Enhanced Synthesis (4-5 hours)

#### 3.1 Add LFO Node

```rust
// in aether-nodes/src/lfo.rs
pub struct LFO {
    phase: f32,
    waveform: u8,  // 0=sine, 1=tri, 2=square, 3=random
    rate: f32,     // Hz
    depth: f32,    // 0..1
}

// Params: rate, depth, waveform, retrigger
```

#### 3.2 Add Pitch Envelope

```rust
// in aether-nodes/src/pitch_envelope.rs
pub struct PitchEnvelope {
    start_freq: f32,
    end_freq: f32,
    time: f32,
    curve: f32,  // 0=linear, 1=exponential
}
```

#### 3.3 Add Unison

```rust
// in instrument.rs
pub struct UnisonVoice {
    voices: Vec<Voice>,  // 3-7 voices
    detune: f32,         // cents
    spread: f32,         // stereo spread
}
```

#### 3.4 Add Velocity Sensitivity

```rust
// in TrackEngine::note_on
let vel_factor = velocity as f32 / 127.0;
let cutoff = preset.cutoff * (0.5 + 0.5 * vel_factor);  // Velocity → filter
let gain = preset.gain * vel_factor;  // Velocity → volume
```

## File Structure

```
crates/
├── aether-sampler/
│   ├── src/
│   │   ├── lib.rs           # ✅ Exists
│   │   ├── engine.rs        # ✅ Exists
│   │   ├── voice.rs         # ✅ Exists
│   │   └── node.rs          # ❌ Need to create (DSP node wrapper)
│   └── Cargo.toml
│
├── aether-nodes/
│   ├── src/
│   │   ├── lfo.rs           # ❌ Need to create
│   │   ├── pitch_envelope.rs # ❌ Need to create
│   │   └── ...
│   └── Cargo.toml
│
├── aether-ui/
│   ├── src/
│   │   ├── instrument.rs    # ✅ Exists - needs extension
│   │   ├── instrument_browser.rs # ❌ Need to create
│   │   └── ...
│   └── Cargo.toml
│
└── assets/
    ├── instruments/
    │   └── drums-studio.json # ✅ Exists
    ├── presets/
    │   └── world/            # ✅ Exists (60 instruments)
    └── samples/              # ❌ Need actual WAV files
        └── drums-studio/
            ├── kick/
            ├── snare/
            ├── hihat/
            ├── toms/
            └── cymbals/
```

## Sample Files Status

### Drums (drums-studio)

**Status**: ❌ **WAV files NOT included in repository**

The `drums-studio.json` references sample files like:

- `drums-studio/kick/kick-v1.wav`
- `drums-studio/snare/snare-v1.wav`
- etc.

**Action Required**:

1. Download VSCO 2 Community Edition samples (CC0 license)
2. Extract drum samples
3. Place in `assets/samples/drums-studio/` directory
4. OR: Find alternative CC0 drum samples
5. OR: Generate synthetic drum samples

**Source**: https://github.com/sgossner/VSCO-2-CE

### World Instruments

**Status**: ✅ **Algorithmic synthesis** (no samples needed)

These use Karplus-Strong and formant synthesis, defined in JSON presets.

## Estimated Time

| Task                         | Time         | Priority |
| ---------------------------- | ------------ | -------- |
| Improve synth presets        | ✅ Done      | Critical |
| Add velocity → filter        | 1 hour       | High     |
| Connect sampler to graph     | 4 hours      | High     |
| Create instrument browser UI | 3 hours      | Medium   |
| Add LFO node                 | 2 hours      | Medium   |
| Add pitch envelope           | 2 hours      | Medium   |
| Add unison                   | 3 hours      | Low      |
| Download/setup samples       | 2 hours      | Medium   |
| Testing & polish             | 3 hours      | High     |
| **TOTAL**                    | **20 hours** |          |

## Recommended Next Steps

### Immediate (This Session)

1. ✅ Improve synth presets (DONE)
2. Add velocity sensitivity to filter
3. Test improved sounds
4. Commit changes

### Short Term (Next Session)

1. Download drum samples
2. Connect sampler to audio graph
3. Add basic instrument selector
4. Test drum playback

### Long Term (Future Sessions)

1. Add LFO and pitch envelope
2. Add unison for thick sounds
3. Build full instrument browser
4. Add all 60 world instruments

## Questions

1. **Do you want to download drum samples now?**
   - Requires downloading VSCO 2 CE (~500MB)
   - Or find alternative CC0 samples

2. **Priority: Better synth OR more instruments?**
   - Better synth: Add LFO, unison, pitch envelope
   - More instruments: Connect sampler, add browser

3. **Should we continue in this session?**
   - We can add velocity sensitivity (quick)
   - Or save major work for next session

Let me know your preference and I'll proceed accordingly!
