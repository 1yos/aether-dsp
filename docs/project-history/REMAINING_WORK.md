# Remaining Work - Practical Implementation Guide

## Current Status: 85% Complete

Your DAW is **functional and usable** right now. The remaining 15% makes it **production-ready**.

---

## ✅ What's Already Done

- Full DAW UI (Song, Piano Roll, Mixer)
- Real-time audio engine
- Polyphonic synth (8 voices/track)
- **Effects chain (Compressor, Reverb, Delay, Filter)** ← Just completed
- MIDI editing
- Undo/redo
- Keyboard shortcuts

---

## 🎯 Critical Path to 100% (5-6 days)

### Day 1-2: Save/Load Project

**Why Critical:** Users need to save their work

**Implementation:**

```rust
// In app_state.rs - Add derives
#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub version: String,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub tracks: Vec<TrackData>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackData {
    pub name: String,
    pub track_type: TrackType,
    pub color: u32,
    pub volume: f32,
    pub pan: f32,
    pub clips: Vec<ClipData>,
    pub instrument: InstrumentPresetData,
    pub effects: Vec<EffectData>,
}

impl AppState {
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let project = ProjectFile {
            version: "1.0".to_string(),
            bpm: self.transport.bpm,
            time_signature: (self.transport.time_sig_num, self.transport.time_sig_den),
            tracks: self.tracks.iter().map(|t| TrackData {
                name: t.name.clone(),
                track_type: t.track_type.clone(),
                color: t.color,
                volume: t.volume,
                pan: t.pan,
                clips: t.clips.iter().map(|c| ClipData {
                    name: c.name.clone(),
                    start_beat: c.start_beat,
                    length_beats: c.length_beats,
                    notes: c.notes.iter().map(|n| NoteData {
                        pitch: n.pitch,
                        beat: n.beat,
                        duration: n.duration,
                        velocity: n.velocity,
                    }).collect(),
                }).collect(),
                instrument: InstrumentPresetData {
                    waveform: t.instrument.waveform,
                    attack: t.instrument.attack,
                    decay: t.instrument.decay,
                    sustain: t.instrument.sustain,
                    release: t.instrument.release,
                    cutoff: t.instrument.cutoff,
                    resonance: t.instrument.resonance,
                    gain: t.instrument.gain,
                },
                effects: t.effects.iter().map(|e| EffectData {
                    effect_type: format!("{:?}", e.effect_type),
                    enabled: e.enabled,
                    params: serde_json::to_value(&e.params).unwrap(),
                }).collect(),
            }).collect(),
        };

        let json = serde_json::to_string_pretty(&project)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let project: ProjectFile = serde_json::from_str(&json)?;

        // Clear current state
        self.tracks.clear();

        // Restore transport
        self.transport.bpm = project.bpm;
        self.transport.time_sig_num = project.time_signature.0;
        self.transport.time_sig_den = project.time_signature.1;

        // Restore tracks
        for td in project.tracks {
            let track_id = self.next_id();
            let mut clips = Vec::new();

            for cd in td.clips {
                let clip_id = self.next_id();
                let mut notes = Vec::new();

                for nd in cd.notes {
                    notes.push(MidiNote {
                        id: self.next_id(),
                        pitch: nd.pitch,
                        beat: nd.beat,
                        duration: nd.duration,
                        velocity: nd.velocity,
                    });
                }

                clips.push(Clip {
                    id: clip_id,
                    track_id,
                    name: cd.name,
                    start_beat: cd.start_beat,
                    length_beats: cd.length_beats,
                    color: td.color,
                    notes,
                });
            }

            self.tracks.push(Track {
                id: track_id,
                name: td.name,
                track_type: td.track_type,
                color: td.color,
                volume: td.volume,
                pan: td.pan,
                muted: false,
                solo: false,
                armed: false,
                height: 72.0,
                clips,
                instrument: InstrumentPreset {
                    waveform: td.instrument.waveform,
                    attack: td.instrument.attack,
                    decay: td.instrument.decay,
                    sustain: td.instrument.sustain,
                    release: td.instrument.release,
                    cutoff: td.instrument.cutoff,
                    resonance: td.instrument.resonance,
                    gain: td.instrument.gain,
                },
                effects: Vec::new(), // Rebuild effects after graph rebuild
            });
        }

        // Rebuild audio graph
        self.rebuild_audio_graph();

        Ok(())
    }

    fn rebuild_audio_graph(&mut self) {
        // Recreate master engine with new track count
        let sched_arc = self.scheduler.clone();
        if let Ok(mut sched) = sched_arc.try_lock() {
            self.master_engine = MasterEngine::build(&mut sched, self.tracks.len());

            // Restore instrument presets
            for (i, track) in self.tracks.iter().enumerate() {
                if let Some(ref mut engine) = self.master_engine {
                    if let Some(Some(ref mut te)) = engine.tracks.get_mut(i) {
                        te.update_preset(&mut sched, track.instrument.clone());
                    }
                }
            }
        }
    }
}
```

**UI Integration:**

```rust
// In daw_app.rs - Add messages
Message::SaveProject => {
    // Use rfd crate for file dialog
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Aether Project", &["aether"])
        .save_file()
    {
        let path_str = path.to_string_lossy().to_string();
        self.state.lock().unwrap().save_to_file(&path_str).ok();
    }
}

Message::LoadProject => {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Aether Project", &["aether"])
        .pick_file()
    {
        let path_str = path.to_string_lossy().to_string();
        self.state.lock().unwrap().load_from_file(&path_str).ok();
    }
}
```

**Add to Cargo.toml:**

```toml
rfd = "0.12"  # File dialog
```

---

### Day 3: Export WAV

**Why Critical:** Users need to export their music

**Implementation:**

```rust
// In app_state.rs
impl AppState {
    pub fn export_wav(&self, path: &str, duration_beats: f64) -> Result<(), Box<dyn std::error::Error>> {
        use hound::{WavWriter, WavSpec, SampleFormat};

        let sample_rate = 48000;
        let duration_secs = (duration_beats / self.transport.bpm as f64) * 60.0;
        let total_samples = (duration_secs * sample_rate as f64) as usize;

        let spec = WavSpec {
            channels: 2,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;
        let mut output = Vec::with_capacity(total_samples * 2);

        // Offline rendering
        let sched_arc = self.scheduler.clone();
        if let Ok(mut sched) = sched_arc.try_lock() {
            let buffer_size = aether_core::BUFFER_SIZE;
            let num_buffers = total_samples / buffer_size;

            for _ in 0..num_buffers {
                sched.process();

                // Get output from master node
                if let Some(ref engine) = self.master_engine {
                    if let Some(master_record) = sched.graph.arena.get(engine.master_id) {
                        let buf = &master_record.outputs[0];
                        for i in 0..buffer_size {
                            let sample = buf[i].clamp(-1.0, 1.0);
                            output.push(sample);
                            output.push(sample); // Duplicate for stereo
                        }
                    }
                }
            }
        }

        // Write samples
        for sample in output {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }

        writer.finalize()?;
        Ok(())
    }
}
```

**UI Integration:**

```rust
Message::ExportWav => {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("WAV Audio", &["wav"])
        .save_file()
    {
        let path_str = path.to_string_lossy().to_string();
        let duration = 64.0; // Export 64 beats (16 bars at 4/4)
        self.state.lock().unwrap().export_wav(&path_str, duration).ok();
    }
}
```

**Add to Cargo.toml:**

```toml
hound = "3.5"  # WAV file writing
```

---

### Day 4: Metronome Sound

**Why Important:** Helps with timing while recording

**Implementation:**

```rust
// In instrument.rs
pub struct Metronome {
    pub click_osc_id: NodeId,
    pub click_env_id: NodeId,
    pub last_beat: f64,
}

impl Metronome {
    pub fn build(sched: &mut Scheduler, master_id: NodeId) -> Option<Self> {
        let osc_id = sched.graph.add_node(Box::new(Oscillator::new()))?;
        let env_id = sched.graph.add_node(Box::new(AdsrEnvelope::new()))?;

        sched.graph.connect(osc_id, env_id, 0);
        sched.graph.connect(env_id, master_id, 15); // Dedicated slot

        // Set click params
        set_param(sched, osc_id, 0, 1000.0); // freq
        set_param(sched, osc_id, 1, 0.2);    // amp
        set_param(sched, osc_id, 2, 0.0);    // sine wave

        set_param(sched, env_id, 0, 0.001);  // attack
        set_param(sched, env_id, 1, 0.03);   // decay
        set_param(sched, env_id, 2, 0.0);    // sustain
        set_param(sched, env_id, 3, 0.01);   // release
        set_param(sched, env_id, 4, 0.0);    // gate off

        Some(Self {
            click_osc_id: osc_id,
            click_env_id: env_id,
            last_beat: -1.0,
        })
    }

    pub fn tick(&mut self, sched: &mut Scheduler, current_beat: f64, time_sig: u8) {
        let beat_floor = current_beat.floor();
        if beat_floor != self.last_beat {
            self.last_beat = beat_floor;

            // Accent on downbeat
            let is_downbeat = (beat_floor as usize) % time_sig as usize == 0;
            let freq = if is_downbeat { 1200.0 } else { 800.0 };

            set_param(sched, self.click_osc_id, 0, freq);
            set_param(sched, self.click_env_id, 4, 1.0); // trigger
        }
    }
}

// In MasterEngine
pub struct MasterEngine {
    pub tracks: Vec<Option<TrackEngine>>,
    pub master_id: NodeId,
    pub metronome: Option<Metronome>,
}

impl MasterEngine {
    pub fn build(sched: &mut Scheduler, track_count: usize) -> Option<Self> {
        let master_id = sched.graph.add_node(Box::new(Mixer))?;
        sched.graph.set_output_node(master_id);

        // Build metronome
        let metronome = Metronome::build(sched, master_id);

        // ... rest of existing code ...

        Some(Self { tracks, master_id, metronome })
    }
}
```

**UI Integration:**

```rust
// In daw_app.rs Tick handler
if self.metronome_on && transport.is_playing {
    let sched_arc = s.scheduler.clone();
    if let Ok(mut sched) = sched_arc.try_lock() {
        if let Some(ref mut engine) = s.master_engine {
            if let Some(ref mut metro) = engine.metronome {
                metro.tick(&mut sched, transport.playhead_beat, transport.time_sig_num);
            }
        }
    }
}
```

---

### Day 5-6: UI Polish

**Quick Wins:**

1. **Time Display (Minutes:Seconds)**

```rust
fn format_time(beat: f64, bpm: f32) -> String {
    let seconds = (beat / bpm as f64) * 60.0;
    let mins = (seconds / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let ms = ((seconds % 1.0) * 100.0) as u32;
    format!("{:02}:{:02}.{:02}", mins, secs, ms)
}

// In transport bar
text(format!("{} | {}",
    format!("{:3}:{}", bar, beat_in_bar),
    format_time(transport.playhead_beat, transport.bpm)
))
```

2. **Track Color Picker**

```rust
fn color_picker_row(current: u32, on_select: impl Fn(u32) -> Message) -> Element<Message> {
    let colors = [
        0x4db8ffff_u32, 0xa78bfaff, 0x34d399ff, 0xf97316ff,
        0xf43f5eff, 0xfbbf24ff, 0x06b6d4ff, 0x8b5cf6ff,
    ];

    row(colors.iter().map(|&c| {
        button(iced::widget::horizontal_space())
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(color_from_u32(c))),
                border: if c == current {
                    iced::Border { color: Color::WHITE, width: 2.0, radius: 3.0.into() }
                } else {
                    iced::Border::default()
                },
                ..Default::default()
            })
            .on_press(on_select(c))
            .into()
    }).collect())
}
```

3. **Peak Hold on VU Meters**

```rust
// In DawApp
pub struct DawApp {
    // ... existing fields ...
    vu_peaks: Vec<(f32, Instant)>, // (peak_level, time)
}

// In update loop
for (i, level) in self.vu_levels.iter().enumerate() {
    if let Some((peak, time)) = self.vu_peaks.get_mut(i) {
        if *level > *peak {
            *peak = *level;
            *time = Instant::now();
        } else if time.elapsed().as_secs_f32() > 2.0 {
            *peak = *level; // Reset after 2 seconds
        }
    }
}

// In mixer view - draw peak line
frame.fill_rectangle(
    iced::Point::new(x, y_at_peak),
    iced::Size::new(meter_width, 1.0),
    Color::from_rgb(1.0, 0.0, 0.0) // Red peak line
);
```

---

## 📦 Dependencies to Add

Add to `crates/aether-ui/Cargo.toml`:

```toml
rfd = "0.12"      # File dialogs
hound = "3.5"     # WAV export
```

---

## 🚀 Implementation Order

1. **Day 1-2:** Save/Load (most critical)
2. **Day 3:** Export WAV (second most critical)
3. **Day 4:** Metronome (quick win)
4. **Day 5-6:** UI polish (nice-to-have)

---

## ✅ Testing Checklist

After each feature:

### Save/Load:

- [ ] Create project with tracks and notes
- [ ] Save to file
- [ ] Close app
- [ ] Reopen and load
- [ ] Verify all tracks, notes, effects restored

### Export:

- [ ] Create simple melody
- [ ] Export to WAV
- [ ] Open in audio player
- [ ] Verify sound quality

### Metronome:

- [ ] Enable metronome
- [ ] Press play
- [ ] Hear click on beats
- [ ] Hear accent on downbeat

---

## 🎯 Success Criteria

You'll have a **production-ready DAW** when:

- ✅ Users can save and reload their work
- ✅ Users can export finished tracks
- ✅ Metronome helps with timing
- ✅ UI is polished and professional

**Current: 85% → Target: 100%**

**Estimated Time: 5-6 focused days**

---

## 💡 Pro Tips

1. **Test incrementally** - Build and test after each feature
2. **Use version control** - Commit after each working feature
3. **Start with save/load** - Most critical for usability
4. **Keep it simple** - Don't over-engineer
5. **Ship it** - 90% done and shipped beats 100% perfect and never released

**You're so close! The hard work is done. Just need these finishing touches!**
