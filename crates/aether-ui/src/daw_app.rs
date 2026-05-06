// daw_app
//! DawApp — the iced Application struct.
//! Full interactive DAW: Song view, Piano Roll, Mixer, transport.
use iced::{
    widget::{button, canvas, column, container, row, scrollable, slider, text},
    Alignment, Color, Element, Length, Task, Theme, time,
};
use std::time::{Duration, Instant};
use crate::app_state::{
    AppState, ActiveView, Clip, MidiNote, PianoRollTool, SongTool,
};
use crate::instrument::MidiEvent;

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    // Transport
    SetView(ActiveView),
    Play, Stop, ToggleRecord, SetBpm(f32), Tick(Instant),
    // Song view
    AddTrack,
    SongCanvasClick { beat: f64, track_idx: usize },
    SongCanvasDrag  { beat: f64, track_idx: usize },
    SongCanvasRelease,
    OpenPianoRoll   { clip_id: u64, track_id: u64 },
    DeleteClip(u64),
    SetSongTool(SongTool),
    SetSongSnap(f64),
    // Piano Roll
    PianoRollClick  { beat: f64, pitch: u8 },
    PianoRollDrag   { beat: f64, pitch: u8 },
    PianoRollRelease,
    DeleteNote(u64),
    SetPianoTool(PianoRollTool),
    SetPianoSnap(f64),
    PianoKeyPress(u8),
    PianoKeyRelease(u8),
    ClosePianoRoll,
    // Mixer
    SetTrackVolume { track_idx: usize, volume: f32 },
    SetTrackMute   { track_idx: usize },
    SetTrackSolo   { track_idx: usize },
    // Keyboard shortcuts
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    DeleteSelected,
    SelectAll,
    TapTempo,
    // Clip operations
    ResizeClip { clip_id: u64, new_length: f64 },
    RenameTrack { track_id: u64, name: String },
    SetTrackInstrumentParam { track_idx: usize, param: InstrumentParam },
    OpenInstrumentPanel(u64),
    CloseInstrumentPanel,
    AddEffect { track_idx: usize, effect_type: crate::app_state::EffectType },
    RemoveEffect { track_idx: usize, effect_id: u64 },
    ToggleEffect { track_idx: usize, effect_id: u64 },
    SetEffectParam { track_idx: usize, effect_id: u64, param: EffectParamMsg },
    // BPM
    TapTempoClick,
    SetBpmDirect(f32),
    // Loop
    ToggleLoop,
    SetLoopStart(f64),
    SetLoopEnd(f64),
}

#[derive(Debug, Clone)]
pub enum InstrumentParam {
    Waveform(f32),
    Attack(f32),
    Decay(f32),
    Sustain(f32),
    Release(f32),
    Cutoff(f32),
    Resonance(f32),
    Gain(f32),
}

#[derive(Debug, Clone)]
pub enum EffectParamMsg {
    ReverbRoom(f32),
    ReverbDamp(f32),
    ReverbWet(f32),
    DelayTime(f32),
    DelayFeedback(f32),
    DelayWet(f32),
    CompThreshold(f32),
    CompRatio(f32),
    FilterCutoff(f32),
    FilterResonance(f32),
}

// ── App struct ────────────────────────────────────────────────────────────────

pub struct DawApp {
    pub state: AppState,
    last_tick: Instant,
    pr_drag_note_id: Option<u64>,
    pr_drag_start_beat: f64,
    pr_drag_start_pitch: u8,
    song_drag_clip_id: Option<u64>,
    song_drag_offset: f64,
}

impl DawApp {
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        let app = Self {
            state,
            last_tick: Instant::now(),
            pr_drag_note_id: None,
            pr_drag_start_beat: 0.0,
            pr_drag_start_pitch: 60,
            song_drag_clip_id: None,
            song_drag_offset: 0.0,
        };
        (app, Task::none())
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        use iced::keyboard::{self, key::Named};
        let tick = time::every(Duration::from_millis(16)).map(Message::Tick);
        let keys = keyboard::on_key_press(|key, modifiers| {
            match key.as_ref() {
                keyboard::Key::Named(Named::Space) => Some(Message::Play),
                keyboard::Key::Character("z") if modifiers.control() => {
                    if modifiers.shift() { Some(Message::Redo) } else { Some(Message::Undo) }
                }
                keyboard::Key::Character("y") if modifiers.control() => Some(Message::Redo),
                keyboard::Key::Character("c") if modifiers.control() => Some(Message::Copy),
                keyboard::Key::Character("v") if modifiers.control() => Some(Message::Paste),
                keyboard::Key::Character("d") if modifiers.control() => Some(Message::Duplicate),
                keyboard::Key::Character("a") if modifiers.control() => Some(Message::SelectAll),
                keyboard::Key::Named(Named::Delete) | keyboard::Key::Named(Named::Backspace) => {
                    Some(Message::DeleteSelected)
                }
                keyboard::Key::Character("1") => Some(Message::SetSongTool(SongTool::Draw)),
                keyboard::Key::Character("2") => Some(Message::SetSongTool(SongTool::Select)),
                keyboard::Key::Character("3") => Some(Message::SetSongTool(SongTool::Erase)),
                _ => None,
            }
        });
        iced::Subscription::batch([tick, keys])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                let delta = now.duration_since(self.last_tick).as_secs_f64();
                self.last_tick = now;
                self.state.lock().unwrap().tick_transport(delta);
            }
            Message::SetView(v) => { self.state.lock().unwrap().active_view = v; }
            Message::Play => { self.state.lock().unwrap().transport.is_playing = true; }
            Message::Stop => {
                let mut s = self.state.lock().unwrap();
                s.transport.is_playing = false;
                s.transport.playhead_beat = 0.0;
                for i in 0..s.tracks.len() {
                    s.midi_queue.push(crate::app_state::PendingMidi {
                        track_idx: i, event: MidiEvent::AllNotesOff,
                    });
                }
                s.flush_midi();
            }
            Message::ToggleRecord => {
                let mut s = self.state.lock().unwrap();
                s.transport.is_recording = !s.transport.is_recording;
            }
            Message::SetBpm(bpm) => {
                self.state.lock().unwrap().transport.bpm = bpm.clamp(20.0, 300.0);
            }
            Message::AddTrack => {
                let mut s = self.state.lock().unwrap();
                let id  = s.next_id();
                let idx = s.tracks.len();
                let colors = [0x4db8ffff_u32,0xa78bfaff,0x34d399ff,0xf97316ff,
                              0xf43f5eff,0xfbbf24ff,0x06b6d4ff,0x8b5cf6ff];
                s.tracks.push(crate::app_state::Track {
                    id, name: format!("Track {}", idx + 1),
                    track_type: crate::app_state::TrackType::Instrument,
                    color: colors[idx % colors.len()],
                    muted: false, solo: false, armed: false,
                    volume: 0.8, pan: 0.0, height: 72.0,
                    clips: Vec::new(), effects: Vec::new(),
                    instrument: crate::instrument::InstrumentPreset::default_instrument(),
                });
                let sched_arc = s.scheduler.clone();
                if let Ok(mut sched) = sched_arc.try_lock() {
                    if let Some(ref mut engine) = s.master_engine {
                        engine.ensure_track(&mut sched, idx);
                    }
                };
            }

            // ── Song view interactions ────────────────────────────────────
            Message::SongCanvasClick { beat, track_idx } => {
                let mut s = self.state.lock().unwrap();
                let snap = s.song_view.snap_beats;
                let snapped = (beat / snap).floor() * snap;
                let tool = s.song_view.tool.clone();
                match tool {
                    SongTool::Draw => {
                        if track_idx < s.tracks.len() {
                            let id = s.next_id();
                            let track_id = s.tracks[track_idx].id;
                            let color = s.tracks[track_idx].color;
                            s.tracks[track_idx].clips.push(Clip {
                                id, track_id,
                                name: format!("Clip {}", id),
                                start_beat: snapped,
                                length_beats: snap * 4.0,
                                color, notes: Vec::new(),
                            });
                            self.song_drag_clip_id = Some(id);
                            self.song_drag_offset = 0.0;
                        }
                    }
                    SongTool::Select => {
                        if track_idx < s.tracks.len() {
                            let found = s.tracks[track_idx].clips.iter()
                                .find(|c| beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                                .map(|c| (c.id, c.start_beat));
                            if let Some((cid, cstart)) = found {
                                self.song_drag_clip_id = Some(cid);
                                self.song_drag_offset = beat - cstart;
                                s.selected_clip_id = Some(cid);
                            }
                        }
                    }
                    SongTool::Erase => {
                        if track_idx < s.tracks.len() {
                            s.tracks[track_idx].clips.retain(|c| {
                                !(beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                            });
                        }
                    }
                }
            }
            Message::SongCanvasDrag { beat, track_idx: _ } => {
                if let Some(clip_id) = self.song_drag_clip_id {
                    let mut s = self.state.lock().unwrap();
                    let snap = s.song_view.snap_beats;
                    let new_start = ((beat - self.song_drag_offset) / snap).floor() * snap;
                    if let Some(clip) = s.find_clip_mut(clip_id) {
                        clip.start_beat = new_start.max(0.0);
                    }
                }
            }
            Message::SongCanvasRelease => { self.song_drag_clip_id = None; }
            Message::OpenPianoRoll { clip_id, track_id } => {
                let mut s = self.state.lock().unwrap();
                s.piano_roll.open_clip_id = Some(clip_id);
                s.piano_roll.open_track_id = Some(track_id);
                s.active_view = ActiveView::PianoRoll;
            }
            Message::DeleteClip(id) => {
                let mut s = self.state.lock().unwrap();
                for track in &mut s.tracks { track.clips.retain(|c| c.id != id); }
            }
            Message::SetSongTool(t) => { self.state.lock().unwrap().song_view.tool = t; }
            Message::SetSongSnap(sn) => { self.state.lock().unwrap().song_view.snap_beats = sn; }

            // ── Piano Roll interactions ───────────────────────────────────
            Message::PianoRollClick { beat, pitch } => {
                let mut s = self.state.lock().unwrap();
                let snap = s.piano_roll.snap_beats;
                let len  = s.piano_roll.default_note_len;
                let tool = s.piano_roll.tool.clone();
                let snapped = (beat / snap).floor() * snap;
                match tool {
                    PianoRollTool::Draw => {
                        if let Some(clip_id) = s.piano_roll.open_clip_id {
                            let note_id = s.next_id();
                            if let Some(clip) = s.find_clip_mut(clip_id) {
                                clip.notes.push(MidiNote {
                                    id: note_id, pitch,
                                    beat: snapped, duration: len, velocity: 100,
                                });
                            }
                            self.pr_drag_note_id = Some(note_id);
                            self.pr_drag_start_beat = snapped;
                            self.pr_drag_start_pitch = pitch;
                        }
                        if let Some(track_id) = s.piano_roll.open_track_id {
                            let track_idx = s.tracks.iter().position(|t| t.id == track_id).unwrap_or(0);
                            s.midi_queue.push(crate::app_state::PendingMidi {
                                track_idx, event: MidiEvent::NoteOn { pitch, velocity: 100 },
                            });
                            s.flush_midi();
                        }
                    }
                    PianoRollTool::Select => {
                        if let Some(clip_id) = s.piano_roll.open_clip_id {
                            let found = s.find_clip(clip_id).and_then(|clip| {
                                clip.notes.iter().find(|n| {
                                    n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration
                                }).map(|n| n.id)
                            });
                            if let Some(nid) = found {
                                s.piano_roll.selected_note_ids = vec![nid];
                                self.pr_drag_note_id = Some(nid);
                                self.pr_drag_start_beat = beat;
                                self.pr_drag_start_pitch = pitch;
                            }
                        }
                    }
                    PianoRollTool::Erase => {
                        if let Some(clip_id) = s.piano_roll.open_clip_id {
                            if let Some(clip) = s.find_clip_mut(clip_id) {
                                clip.notes.retain(|n| {
                                    !(n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration)
                                });
                            }
                        }
                    }
                }
            }
            Message::PianoRollDrag { beat, pitch } => {
                if let Some(note_id) = self.pr_drag_note_id {
                    let mut s = self.state.lock().unwrap();
                    let snap = s.piano_roll.snap_beats;
                    let tool = s.piano_roll.tool.clone();
                    if let Some(clip_id) = s.piano_roll.open_clip_id {
                        match tool {
                            PianoRollTool::Draw => {
                                let start = self.pr_drag_start_beat;
                                let new_end = ((beat / snap).ceil() * snap).max(start + snap);
                                if let Some(clip) = s.find_clip_mut(clip_id) {
                                    if let Some(note) = clip.notes.iter_mut().find(|n| n.id == note_id) {
                                        note.duration = (new_end - start).max(snap);
                                    }
                                }
                            }
                            PianoRollTool::Select => {
                                let db = beat - self.pr_drag_start_beat;
                                let dp = pitch as i16 - self.pr_drag_start_pitch as i16;
                                if let Some(clip) = s.find_clip_mut(clip_id) {
                                    if let Some(note) = clip.notes.iter_mut().find(|n| n.id == note_id) {
                                        note.beat  = ((note.beat + db).max(0.0) / snap).floor() * snap;
                                        note.pitch = (note.pitch as i16 + dp).clamp(0, 127) as u8;
                                    }
                                }
                                self.pr_drag_start_beat  = beat;
                                self.pr_drag_start_pitch = pitch;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Message::PianoRollRelease => {
                if let Some(note_id) = self.pr_drag_note_id {
                    let mut s = self.state.lock().unwrap();
                    if let Some(track_id) = s.piano_roll.open_track_id {
                        let track_idx = s.tracks.iter().position(|t| t.id == track_id).unwrap_or(0);
                        let pitch = s.piano_roll.open_clip_id
                            .and_then(|cid| s.find_clip(cid))
                            .and_then(|clip| clip.notes.iter().find(|n| n.id == note_id))
                            .map(|n| n.pitch)
                            .unwrap_or(self.pr_drag_start_pitch);
                        s.midi_queue.push(crate::app_state::PendingMidi {
                            track_idx, event: MidiEvent::NoteOff { pitch },
                        });
                        s.flush_midi();
                    }
                }
                self.pr_drag_note_id = None;
            }
            Message::DeleteNote(id) => {
                let mut s = self.state.lock().unwrap();
                if let Some(clip_id) = s.piano_roll.open_clip_id {
                    if let Some(clip) = s.find_clip_mut(clip_id) {
                        clip.notes.retain(|n| n.id != id);
                    }
                }
            }
            Message::SetPianoTool(t) => { self.state.lock().unwrap().piano_roll.tool = t; }
            Message::SetPianoSnap(sn) => { self.state.lock().unwrap().piano_roll.snap_beats = sn; }
            Message::PianoKeyPress(pitch) => {
                let mut s = self.state.lock().unwrap();
                if let Some(track_id) = s.piano_roll.open_track_id {
                    let track_idx = s.tracks.iter().position(|t| t.id == track_id).unwrap_or(0);
                    s.midi_queue.push(crate::app_state::PendingMidi {
                        track_idx, event: MidiEvent::NoteOn { pitch, velocity: 100 },
                    });
                    s.flush_midi();
                }
            }
            Message::PianoKeyRelease(pitch) => {
                let mut s = self.state.lock().unwrap();
                if let Some(track_id) = s.piano_roll.open_track_id {
                    let track_idx = s.tracks.iter().position(|t| t.id == track_id).unwrap_or(0);
                    s.midi_queue.push(crate::app_state::PendingMidi {
                        track_idx, event: MidiEvent::NoteOff { pitch },
                    });
                    s.flush_midi();
                }
            }
            Message::ClosePianoRoll => {
                let mut s = self.state.lock().unwrap();
                s.piano_roll.open_clip_id = None;
                s.active_view = ActiveView::Song;
            }

            // ── Mixer ─────────────────────────────────────────────────────
            Message::SetTrackVolume { track_idx, volume } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.tracks[track_idx].volume = volume;
                    let sched_arc = s.scheduler.clone();
                    if let Ok(mut sched) = sched_arc.try_lock() {
                        if let Some(ref mut engine) = s.master_engine {
                            if let Some(slot) = engine.tracks.get_mut(track_idx) {
                                if let Some(ref mut te) = slot {
                                    te.set_volume(&mut sched, volume);
                                }
                            }
                        }
                    };
                }
            }
            Message::SetTrackMute { track_idx } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.tracks[track_idx].muted = !s.tracks[track_idx].muted;
                    let muted = s.tracks[track_idx].muted;
                    if muted {
                        s.midi_queue.push(crate::app_state::PendingMidi {
                            track_idx, event: MidiEvent::AllNotesOff,
                        });
                        s.flush_midi();
                    }
                }
            }
            Message::SetTrackSolo { track_idx } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    let was = s.tracks[track_idx].solo;
                    for t in &mut s.tracks { t.solo = false; }
                    if !was { s.tracks[track_idx].solo = true; }
                }
            }

            // ── Keyboard shortcuts ────────────────────────────────────────
            Message::Undo => { self.state.lock().unwrap().undo(); }
            Message::Redo => { self.state.lock().unwrap().redo(); }
            Message::Copy => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                match view {
                    ActiveView::Song      => s.copy_selected_clips(),
                    ActiveView::PianoRoll => s.copy_selected_notes(),
                    _ => {}
                }
            }
            Message::Paste => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                match view {
                    ActiveView::Song      => s.paste_clips(),
                    ActiveView::PianoRoll => s.paste_notes(),
                    _ => {}
                }
            }
            Message::Duplicate => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                if view == ActiveView::Song {
                    s.duplicate_selected_clips();
                }
            }
            Message::DeleteSelected => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                match view {
                    ActiveView::Song      => s.delete_selected_clips(),
                    ActiveView::PianoRoll => s.delete_selected_notes(),
                    _ => {}
                }
            }
            Message::SelectAll => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                match view {
                    ActiveView::Song      => s.select_all_clips(),
                    ActiveView::PianoRoll => s.select_all_notes(),
                    _ => {}
                }
            }
            Message::TapTempo | Message::TapTempoClick => {
                let mut s = self.state.lock().unwrap();
                if let Some(bpm) = s.tap_tempo.tap() {
                    s.transport.bpm = bpm.clamp(20.0, 300.0);
                }
            }

            // ── Clip operations ───────────────────────────────────────────
            Message::ResizeClip { clip_id, new_length } => {
                let mut s = self.state.lock().unwrap();
                if let Some(clip) = s.find_clip_mut(clip_id) {
                    clip.length_beats = new_length.max(0.25);
                }
            }
            Message::RenameTrack { track_id, name } => {
                let mut s = self.state.lock().unwrap();
                if let Some(track) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                    track.name = name;
                }
            }

            // ── Instrument panel ──────────────────────────────────────────
            Message::OpenInstrumentPanel(track_id) => {
                self.state.lock().unwrap().instrument_panel_track = Some(track_id);
            }
            Message::CloseInstrumentPanel => {
                self.state.lock().unwrap().instrument_panel_track = None;
            }
            Message::SetTrackInstrumentParam { track_idx, param } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    match param {
                        InstrumentParam::Waveform(v)  => s.tracks[track_idx].instrument.waveform  = v,
                        InstrumentParam::Attack(v)    => s.tracks[track_idx].instrument.attack    = v,
                        InstrumentParam::Decay(v)     => s.tracks[track_idx].instrument.decay     = v,
                        InstrumentParam::Sustain(v)   => s.tracks[track_idx].instrument.sustain   = v,
                        InstrumentParam::Release(v)   => s.tracks[track_idx].instrument.release   = v,
                        InstrumentParam::Cutoff(v)    => s.tracks[track_idx].instrument.cutoff    = v,
                        InstrumentParam::Resonance(v) => s.tracks[track_idx].instrument.resonance = v,
                        InstrumentParam::Gain(v)      => s.tracks[track_idx].instrument.gain      = v,
                    }
                    // Apply to live engine
                    let preset = s.tracks[track_idx].instrument.clone();
                    let sched_arc = s.scheduler.clone();
                    if let Ok(mut sched) = sched_arc.try_lock() {
                        if let Some(ref mut engine) = s.master_engine {
                            if let Some(slot) = engine.tracks.get_mut(track_idx) {
                                if let Some(ref mut te) = slot {
                                    te.update_preset(&mut sched, preset);
                                }
                            }
                        }
                    };
                }
            }

            // ── Effects ───────────────────────────────────────────────────
            Message::AddEffect { track_idx, effect_type } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.push_undo("Add Effect");
                    let id = s.next_id();
                    s.tracks[track_idx].effects.push(crate::app_state::TrackEffect {
                        id,
                        effect_type,
                        enabled: true,
                        params: crate::app_state::EffectParams::default(),
                    });
                }
            }
            Message::RemoveEffect { track_idx, effect_id } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.push_undo("Remove Effect");
                    s.tracks[track_idx].effects.retain(|e| e.id != effect_id);
                }
            }
            Message::ToggleEffect { track_idx, effect_id } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    if let Some(fx) = s.tracks[track_idx].effects.iter_mut().find(|e| e.id == effect_id) {
                        fx.enabled = !fx.enabled;
                    }
                }
            }
            Message::SetEffectParam { track_idx, effect_id, param } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    if let Some(fx) = s.tracks[track_idx].effects.iter_mut().find(|e| e.id == effect_id) {
                        match param {
                            EffectParamMsg::ReverbRoom(v)      => fx.params.reverb_room = v,
                            EffectParamMsg::ReverbDamp(v)      => fx.params.reverb_damp = v,
                            EffectParamMsg::ReverbWet(v)       => fx.params.reverb_wet = v,
                            EffectParamMsg::DelayTime(v)       => fx.params.delay_time = v,
                            EffectParamMsg::DelayFeedback(v)   => fx.params.delay_feedback = v,
                            EffectParamMsg::DelayWet(v)        => fx.params.delay_wet = v,
                            EffectParamMsg::CompThreshold(v)   => fx.params.comp_threshold = v,
                            EffectParamMsg::CompRatio(v)       => fx.params.comp_ratio = v,
                            EffectParamMsg::FilterCutoff(v)    => fx.params.filter_cutoff = v,
                            EffectParamMsg::FilterResonance(v) => fx.params.filter_resonance = v,
                        }
                    }
                }
            }

            // ── BPM / Loop ────────────────────────────────────────────────
            Message::SetBpmDirect(bpm) => {
                self.state.lock().unwrap().transport.bpm = bpm.clamp(20.0, 300.0);
            }
            Message::ToggleLoop => {
                let mut s = self.state.lock().unwrap();
                s.transport.loop_enabled = !s.transport.loop_enabled;
            }
            Message::SetLoopStart(b) => { self.state.lock().unwrap().transport.loop_start = b; }
            Message::SetLoopEnd(b)   => { self.state.lock().unwrap().transport.loop_end = b; }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let s = self.state.lock().unwrap();
        let active    = s.active_view;
        let transport = s.transport.clone();
        let tracks    = s.tracks.clone();
        let engine_ok = s.engine_status == crate::app_state::EngineStatus::Running;
        let song_st   = s.song_view.clone();
        let pr_st     = s.piano_roll.clone();
        let open_clip = pr_st.open_clip_id.and_then(|id| s.find_clip(id)).cloned();
        let instr_panel_track = s.instrument_panel_track;
        let selected_track_idx = s.selected_track_id
            .and_then(|id| s.tracks.iter().position(|t| t.id == id));
        drop(s);

        let top_bar = build_top_bar(active, &transport, engine_ok);
        let workspace: Element<Message> = match active {
            ActiveView::Song      => song_view_el(tracks.clone(), transport, song_st),
            ActiveView::PianoRoll => piano_roll_el(open_clip, pr_st),
            ActiveView::Mixer     => mixer_view_el(tracks.clone()),
            _ => placeholder_view("Coming soon"),
        };

        // Instrument panel overlay (shown when a track instrument is open)
        let instr_panel: Option<Element<Message>> = instr_panel_track.and_then(|tid| {
            let track_idx = tracks.iter().position(|t| t.id == tid)?;
            Some(instrument_panel_el(&tracks[track_idx], track_idx))
        });

        // Effects chain bar (shown for selected track in Song view)
        let fx_bar: Option<Element<Message>> = if active == ActiveView::Song {
            selected_track_idx.map(|idx| effects_bar_el(&tracks[idx], idx))
        } else {
            None
        };

        let mut main_col = column![top_bar, workspace];
        if let Some(fx) = fx_bar {
            main_col = main_col.push(fx);
        }

        if let Some(panel) = instr_panel {
            // Overlay the instrument panel at the bottom
            let base: Element<Message> = main_col.into();
            column![base, panel].into()
        } else {
            main_col.into()
        }
    }
}

// ── Top bar ───────────────────────────────────────────────────────────────────

fn build_top_bar(
    active: ActiveView,
    transport: &crate::app_state::Transport,
    engine_ok: bool,
) -> Element<'static, Message> {
    let views = [
        ("Song",       ActiveView::Song),
        ("Piano Roll", ActiveView::PianoRoll),
        ("Mixer",      ActiveView::Mixer),
        ("Patcher",    ActiveView::Patcher),
        ("Perform",    ActiveView::Perform),
    ];

    let tabs = views.iter().fold(row![].spacing(2), |r, (label, view)| {
        let is_active = active == *view;
        let msg = Message::SetView(*view);
        let lbl = label.to_string();
        r.push(
            button(text(lbl).size(12))
                .on_press(msg)
                .style(move |_, _| tab_style(is_active))
        )
    });

    let is_playing   = transport.is_playing;
    let is_recording = transport.is_recording;
    let bpm          = transport.bpm;
    let beat         = transport.playhead_beat;
    let loop_enabled = transport.loop_enabled;

    container(
        row![
            text("Aether Studio").size(14).color(Color::from_rgb(0.3, 0.72, 1.0)),
            tabs,
            button(text("■").size(12)).on_press(Message::Stop)
                .style(|_,_| btn_style(false, false)),
            button(text(if is_playing { "▶ Playing" } else { "▶ Play" }).size(12))
                .on_press(Message::Play)
                .style(move |_,_| btn_style(is_playing, false)),
            button(text("● Rec").size(12)).on_press(Message::ToggleRecord)
                .style(move |_,_| btn_style(is_recording, true)),
            text(format!("BPM {:.0}", bpm)).size(12).color(Color::WHITE),
            text(format!("{:.2}", beat)).size(11).color(Color::from_rgb(0.28,0.4,0.52)),
            button(text("TAP").size(10)).on_press(Message::TapTempoClick)
                .style(|_,_| btn_style(false, false)),
            button(text("⟳ Loop").size(10)).on_press(Message::ToggleLoop)
                .style(move |_,_| tab_style(loop_enabled)),
            iced::widget::horizontal_space(),
            text(if engine_ok { "● Live" } else { "● Offline" }).size(11)
                .color(if engine_ok {
                    Color::from_rgb(0.0, 0.9, 0.63)
                } else {
                    Color::from_rgb(0.94, 0.33, 0.31)
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(8),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.04, 0.07, 0.13))),
        border: iced::Border {
            color: Color::from_rgb(0.06, 0.12, 0.18),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Song view ─────────────────────────────────────────────────────────────────

fn song_view_el(
    tracks: Vec<crate::app_state::Track>,
    transport: crate::app_state::Transport,
    sv: crate::app_state::SongViewState,
) -> Element<'static, Message> {
    let snap_options: Vec<(&str, f64)> = vec![
        ("1 bar", 1.0), ("1/2", 0.5), ("1/4", 0.25), ("1/8", 0.125),
    ];
    let snap_row = snap_options.iter().fold(row![].spacing(4), |r, (label, val)| {
        let active = (sv.snap_beats - val).abs() < 0.001;
        let v = *val;
        r.push(
            button(text(*label).size(10))
                .on_press(Message::SetSongSnap(v))
                .style(move |_,_| tab_style(active))
        )
    });

    let tool_row = {
        let draw_active   = sv.tool == SongTool::Draw;
        let select_active = sv.tool == SongTool::Select;
        let erase_active  = sv.tool == SongTool::Erase;
        row![
            button(text("✏ Draw").size(10)).on_press(Message::SetSongTool(SongTool::Draw))
                .style(move |_,_| tab_style(draw_active)),
            button(text("↖ Select").size(10)).on_press(Message::SetSongTool(SongTool::Select))
                .style(move |_,_| tab_style(select_active)),
            button(text("✕ Erase").size(10)).on_press(Message::SetSongTool(SongTool::Erase))
                .style(move |_,_| tab_style(erase_active)),
        ].spacing(4)
    };

    let toolbar = container(
        row![
            button(text("+ Track").size(11)).on_press(Message::AddTrack)
                .style(|_,_| btn_style(false, false)),
            tool_row,
            snap_row,
        ].spacing(8).padding(6),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03, 0.06, 0.1))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    });

    // Track headers
    let headers: Vec<Element<Message>> = tracks.iter().enumerate().map(|(i, t)| {
        let c    = color_from_u32(t.color);
        let name = t.name.clone();
        let vol  = (t.volume * 100.0) as u32;
        let muted = t.muted;
        let solo  = t.solo;
        let tid   = t.id;
        container(
            row![
                container(iced::widget::vertical_space())
                    .width(3).height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(c)),
                        ..Default::default()
                    }),
                column![
                    text(name).size(11).color(Color::WHITE),
                    text(format!("{}%", vol)).size(9).color(Color::from_rgb(0.28,0.4,0.52)),
                    row![
                        button(text("M").size(9))
                            .on_press(Message::SetTrackMute { track_idx: i })
                            .style(move |_,_| btn_style(muted, true)),
                        button(text("S").size(9))
                            .on_press(Message::SetTrackSolo { track_idx: i })
                            .style(move |_,_| btn_style(solo, false)),
                        button(text("🎹").size(9))
                            .on_press(Message::OpenInstrumentPanel(tid))
                            .style(|_,_| btn_style(false, false)),
                    ].spacing(3),
                ].spacing(2).padding([4, 8]),
            ].height(Length::Fixed(72.0)),
        )
        .width(Length::Fixed(180.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    let header_col: Element<Message> = if headers.is_empty() {
        container(text("No tracks — click + Track").size(11).color(Color::from_rgb(0.28,0.4,0.52)))
            .width(Length::Fixed(180.0)).height(Length::Fill).into()
    } else {
        scrollable(column(headers).spacing(1)).height(Length::Fill).into()
    };

    let timeline: Element<Message> = canvas::Canvas::new(TimelineCanvas {
        tracks,
        playhead_beat: transport.playhead_beat,
        bpm: transport.bpm,
        time_sig: transport.time_sig_num,
        zoom_x: sv.zoom_x,
        snap_beats: sv.snap_beats,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let main = row![header_col, timeline].height(Length::Fill);
    column![toolbar, main].height(Length::Fill).into()
}

// ── Timeline canvas ───────────────────────────────────────────────────────────

struct TimelineCanvas {
    tracks: Vec<crate::app_state::Track>,
    playhead_beat: f64,
    bpm: f32,
    time_sig: u8,
    zoom_x: f32,
    snap_beats: f64,
}

// ── Timeline canvas interaction state ────────────────────────────────────────

#[derive(Default, Clone)]
struct TimelineState {
    dragging: bool,
    last_beat: f64,
    last_track: usize,
}

impl canvas::Program<Message> for TimelineCanvas {
    type State = TimelineState;

    fn update(
        &self,
        state: &mut TimelineState,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let ruler_h = 28.0f32;
        let bw = self.zoom_x;

        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => return (canvas::event::Status::Ignored, None),
        };

        // Convert pixel position to beat + track index
        let beat = (pos.x / bw) as f64;
        let track_idx = if pos.y > ruler_h {
            let mut ty = ruler_h;
            let mut idx = 0;
            for (i, track) in self.tracks.iter().enumerate() {
                if pos.y >= ty && pos.y < ty + track.height {
                    idx = i;
                    break;
                }
                ty += track.height;
                idx = i;
            }
            idx
        } else {
            0
        };

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                state.dragging = true;
                state.last_beat = beat;
                state.last_track = track_idx;
                (canvas::event::Status::Captured,
                 Some(Message::SongCanvasClick { beat, track_idx }))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if state.dragging => {
                state.last_beat = beat;
                (canvas::event::Status::Captured,
                 Some(Message::SongCanvasDrag { beat, track_idx }))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                state.dragging = false;
                (canvas::event::Status::Captured, Some(Message::SongCanvasRelease))
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self, _: &TimelineState, renderer: &iced::Renderer, _: &Theme,
        bounds: iced::Rectangle, _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let bw       = self.zoom_x;
        let ruler_h  = 28.0f32;
        let total    = 256usize;

        // Background
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.04,0.08,0.13));
        // Ruler
        frame.fill_rectangle(iced::Point::ORIGIN, iced::Size::new(bounds.width, ruler_h),
            Color::from_rgb(0.03,0.06,0.1));

        // Beat/bar lines + numbers
        for i in 0..total {
            let is_bar = i % self.time_sig as usize == 0;
            let x = i as f32 * bw;
            if x > bounds.width { break; }
            let lh = if is_bar { ruler_h } else { ruler_h * 0.4 };
            frame.fill_rectangle(
                iced::Point::new(x, ruler_h - lh),
                iced::Size::new(1.0, lh),
                if is_bar { Color::from_rgb(0.15,0.28,0.45) } else { Color::from_rgb(0.06,0.1,0.16) },
            );
            if is_bar {
                frame.fill_text(canvas::Text {
                    content: (i / self.time_sig as usize + 1).to_string(),
                    position: iced::Point::new(x + 3.0, 4.0),
                    color: Color::from_rgb(0.3, 0.72, 1.0),
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });
            }
        }

        // Tracks
        let mut ty = ruler_h;
        for track in &self.tracks {
            let th = track.height;
            let tc = color_from_u32(track.color);

            // Track row background
            frame.fill_rectangle(iced::Point::new(0.0, ty), iced::Size::new(bounds.width, th),
                Color::from_rgb(0.04,0.08,0.13));

            // Grid lines
            for i in 0..total {
                let x = i as f32 * bw;
                if x > bounds.width { break; }
                let is_bar = i % self.time_sig as usize == 0;
                frame.fill_rectangle(iced::Point::new(x, ty), iced::Size::new(1.0, th),
                    if is_bar { Color::from_rgb(0.07,0.14,0.22) } else { Color::from_rgb(0.04,0.08,0.13) });
            }

            // Clips
            for clip in &track.clips {
                let cx = clip.start_beat as f32 * bw + 1.0;
                let cw = (clip.length_beats as f32 * bw - 2.0).max(8.0);
                // Fill
                frame.fill_rectangle(
                    iced::Point::new(cx, ty + 4.0),
                    iced::Size::new(cw, th - 8.0),
                    Color { a: 0.18, ..tc },
                );
                // Border
                let stroke = canvas::Stroke::default()
                    .with_color(Color { a: 0.7, ..tc })
                    .with_width(1.0);
                frame.stroke(
                    &canvas::Path::rectangle(iced::Point::new(cx, ty + 4.0), iced::Size::new(cw, th - 8.0)),
                    stroke,
                );
                // Clip name
                frame.fill_text(canvas::Text {
                    content: clip.name.clone(),
                    position: iced::Point::new(cx + 4.0, ty + 8.0),
                    color: Color { a: 0.9, ..tc },
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });
                // Mini note preview inside clip
                if !clip.notes.is_empty() {
                    let note_area_h = th - 16.0;
                    for note in &clip.notes {
                        let nx = cx + note.beat as f32 * bw;
                        let nw = (note.duration as f32 * bw).max(2.0);
                        let ny = ty + 8.0 + (1.0 - note.pitch as f32 / 127.0) * note_area_h;
                        frame.fill_rectangle(
                            iced::Point::new(nx, ny),
                            iced::Size::new(nw, 2.0),
                            Color { a: 0.8, ..tc },
                        );
                    }
                }
            }

            // Track separator
            frame.fill_rectangle(
                iced::Point::new(0.0, ty + th - 1.0),
                iced::Size::new(bounds.width, 1.0),
                Color::from_rgb(0.06, 0.12, 0.18),
            );
            ty += th;
        }

        // Playhead
        let phx = self.playhead_beat as f32 * bw;
        frame.fill_rectangle(
            iced::Point::new(phx, 0.0),
            iced::Size::new(2.0, bounds.height),
            Color::from_rgba(0.3, 0.72, 1.0, 0.9),
        );
        let tri = canvas::Path::new(|p| {
            p.move_to(iced::Point::new(phx - 5.0, 0.0));
            p.line_to(iced::Point::new(phx + 5.0, 0.0));
            p.line_to(iced::Point::new(phx, 10.0));
            p.close();
        });
        frame.fill(&tri, Color::from_rgb(0.3, 0.72, 1.0));

        vec![frame.into_geometry()]
    }
}

// ── Piano Roll ────────────────────────────────────────────────────────────────

fn piano_roll_el(
    clip: Option<Clip>,
    pr: crate::app_state::PianoRollState,
) -> Element<'static, Message> {
    let tool_row = {
        let draw_active   = pr.tool == PianoRollTool::Draw;
        let select_active = pr.tool == PianoRollTool::Select;
        let erase_active  = pr.tool == PianoRollTool::Erase;
        row![
            button(text("✏ Draw").size(10)).on_press(Message::SetPianoTool(PianoRollTool::Draw))
                .style(move |_,_| tab_style(draw_active)),
            button(text("↖ Select").size(10)).on_press(Message::SetPianoTool(PianoRollTool::Select))
                .style(move |_,_| tab_style(select_active)),
            button(text("✕ Erase").size(10)).on_press(Message::SetPianoTool(PianoRollTool::Erase))
                .style(move |_,_| tab_style(erase_active)),
        ].spacing(4)
    };

    let snap_options: Vec<(&str, f64)> = vec![
        ("1/4", 0.25), ("1/8", 0.125), ("1/16", 0.0625), ("1/32", 0.03125),
    ];
    let snap_row = snap_options.iter().fold(row![].spacing(4), |r, (label, val)| {
        let active = (pr.snap_beats - val).abs() < 0.001;
        let v = *val;
        r.push(
            button(text(*label).size(10))
                .on_press(Message::SetPianoSnap(v))
                .style(move |_,_| tab_style(active))
        )
    });

    let toolbar = container(
        row![
            button(text("← Back").size(11)).on_press(Message::ClosePianoRoll)
                .style(|_,_| btn_style(false, false)),
            text(clip.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| "No clip".to_string())).size(12)
                .color(Color::WHITE),
            tool_row,
            snap_row,
        ].spacing(8).padding(6),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    });

    // Piano keyboard on the left
    let keys: Vec<Element<Message>> = (0..88u8).rev().map(|k| {
        let midi = k + 21; // A0 = 21
        let is_black = matches!(midi % 12, 1 | 3 | 6 | 8 | 10);
        let h = pr.zoom_y;
        let bg = if is_black {
            Color::from_rgb(0.08, 0.08, 0.1)
        } else {
            Color::from_rgb(0.85, 0.87, 0.9)
        };
        let pitch = midi;
        container(
            button(iced::widget::horizontal_space())
                .on_press(Message::PianoKeyPress(pitch))
                .style(move |_, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border { color: Color::from_rgb(0.1,0.1,0.15), width: 0.5, radius: 0.0.into() },
                    ..Default::default()
                })
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(h))
        ).into()
    }).collect();

    let key_col: Element<Message> = scrollable(column(keys).spacing(0))
        .height(Length::Fill)
        .into();

    // Note grid canvas
    let notes = clip.as_ref().map(|c| c.notes.clone()).unwrap_or_default();
    let clip_id = clip.as_ref().map(|c| c.id);
    let note_canvas: Element<Message> = canvas::Canvas::new(PianoRollCanvas {
        notes,
        zoom_x: pr.zoom_x,
        zoom_y: pr.zoom_y,
        snap_beats: pr.snap_beats,
        scroll_y: pr.scroll_y,
        selected_ids: pr.selected_note_ids.clone(),
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let grid_row = row![key_col, note_canvas].height(Length::Fill);
    column![toolbar, grid_row].height(Length::Fill).into()
}

struct PianoRollCanvas {
    notes: Vec<MidiNote>,
    zoom_x: f32,
    zoom_y: f32,
    snap_beats: f64,
    scroll_y: f32,
    selected_ids: Vec<u64>,
}

// ── Piano Roll canvas interaction state ──────────────────────────────────────

#[derive(Default, Clone)]
struct PianoRollCanvasState {
    dragging: bool,
    last_beat: f64,
    last_pitch: u8,
}

impl canvas::Program<Message> for PianoRollCanvas {
    type State = PianoRollCanvasState;

    fn update(
        &self,
        state: &mut PianoRollCanvasState,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let bw = self.zoom_x;
        let kh = self.zoom_y;
        let total_keys = 128usize;

        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => return (canvas::event::Status::Ignored, None),
        };

        let beat = (pos.x / bw) as f64;
        let key_row = (pos.y / kh) as usize;
        let pitch = (total_keys.saturating_sub(1 + key_row)) as u8;

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                state.dragging = true;
                state.last_beat = beat;
                state.last_pitch = pitch;
                (canvas::event::Status::Captured,
                 Some(Message::PianoRollClick { beat, pitch }))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if state.dragging => {
                state.last_beat = beat;
                state.last_pitch = pitch;
                (canvas::event::Status::Captured,
                 Some(Message::PianoRollDrag { beat, pitch }))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                state.dragging = false;
                (canvas::event::Status::Captured, Some(Message::PianoRollRelease))
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self, _: &PianoRollCanvasState, renderer: &iced::Renderer, _: &Theme,
        bounds: iced::Rectangle, _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let bw = self.zoom_x;
        let kh = self.zoom_y;
        let total_keys = 128;

        // Background
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.04,0.08,0.13));

        // Horizontal key rows
        for k in 0..total_keys {
            let midi = total_keys - 1 - k;
            let is_black = matches!(midi % 12, 1 | 3 | 6 | 8 | 10);
            let y = k as f32 * kh;
            if y > bounds.height { break; }
            let bg = if is_black {
                Color::from_rgb(0.035, 0.07, 0.11)
            } else {
                Color::from_rgb(0.045, 0.09, 0.14)
            };
            frame.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(bounds.width, kh), bg);
            // Row separator
            frame.fill_rectangle(iced::Point::new(0.0, y + kh - 0.5), iced::Size::new(bounds.width, 0.5),
                Color::from_rgb(0.06,0.1,0.16));
        }

        // Vertical beat lines
        let total_beats = 64;
        for i in 0..total_beats {
            let x = i as f32 * bw;
            if x > bounds.width { break; }
            let is_bar = i % 4 == 0;
            frame.fill_rectangle(iced::Point::new(x, 0.0), iced::Size::new(1.0, bounds.height),
                if is_bar { Color::from_rgb(0.1,0.2,0.32) } else { Color::from_rgb(0.06,0.1,0.16) });
        }

        // Notes
        for note in &self.notes {
            let x  = note.beat as f32 * bw;
            let y  = (total_keys - 1 - note.pitch as usize) as f32 * kh;
            let nw = (note.duration as f32 * bw).max(4.0);
            let selected = self.selected_ids.contains(&note.id);

            let fill_color = if selected {
                Color::from_rgb(0.0, 0.9, 0.63)
            } else {
                Color::from_rgb(0.3, 0.72, 1.0)
            };

            frame.fill_rectangle(
                iced::Point::new(x + 1.0, y + 1.0),
                iced::Size::new(nw - 2.0, kh - 2.0),
                fill_color,
            );
            // Note border
            let stroke = canvas::Stroke::default()
                .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                .with_width(0.5);
            frame.stroke(
                &canvas::Path::rectangle(iced::Point::new(x + 1.0, y + 1.0), iced::Size::new(nw - 2.0, kh - 2.0)),
                stroke,
            );
        }

        vec![frame.into_geometry()]
    }
}

// ── Mixer view ────────────────────────────────────────────────────────────────

fn mixer_view_el(tracks: Vec<crate::app_state::Track>) -> Element<'static, Message> {
    let strips: Vec<Element<Message>> = tracks.iter().enumerate().map(|(i, t)| {
        let c     = color_from_u32(t.color);
        let name  = t.name.clone();
        let vol   = t.volume;
        let muted = t.muted;
        let solo  = t.solo;
        let vol_pct = (vol * 100.0) as u32;

        container(
            column![
                // Color bar at top
                container(iced::widget::horizontal_space())
                    .width(Length::Fill).height(Length::Fixed(4.0))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(c)),
                        ..Default::default()
                    }),
                // Track name
                text(name).size(10).color(c),
                // VU meter placeholder
                container(iced::widget::vertical_space())
                    .width(Length::Fixed(48.0)).height(Length::Fixed(80.0))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.04,0.08,0.13))),
                        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 3.0.into() },
                        ..Default::default()
                    }),
                // Fader
                container(iced::widget::vertical_space())
                    .width(Length::Fixed(20.0)).height(Length::Fixed(120.0))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.04,0.08,0.13))),
                        border: iced::Border { color: Color::from_rgba(c.r, c.g, c.b, 0.4), width: 1.0, radius: 3.0.into() },
                        ..Default::default()
                    }),
                text(format!("{}%", vol_pct)).size(9).color(Color::from_rgb(0.28,0.4,0.52)),
                // Mute / Solo
                row![
                    button(text("M").size(9))
                        .on_press(Message::SetTrackMute { track_idx: i })
                        .style(move |_,_| btn_style(muted, true)),
                    button(text("S").size(9))
                        .on_press(Message::SetTrackSolo { track_idx: i })
                        .style(move |_,_| btn_style(solo, false)),
                ].spacing(3),
            ]
            .spacing(4)
            .align_x(Alignment::Center)
            .padding(6),
        )
        .width(Length::Fixed(72.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 6.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    container(
        scrollable(row(strips).spacing(6).padding(12))
            .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.02,0.05,0.08))),
        ..Default::default()
    })
    .into()
}

fn placeholder_view(msg: &str) -> Element<'static, Message> {
    let msg = msg.to_string();
    container(text(msg).size(14).color(Color::from_rgb(0.28,0.4,0.52)))
        .width(Length::Fill).height(Length::Fill)
        .center_x(Length::Fill).center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.02,0.05,0.08))),
            ..Default::default()
        })
        .into()
}

// ── Style helpers ─────────────────────────────────────────────────────────────

fn tab_style(active: bool) -> iced::widget::button::Style {
    if active {
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.3,0.72,1.0,0.12))),
            border: iced::Border { color: Color::from_rgba(0.3,0.72,1.0,0.3), width: 1.0, radius: 5.0.into() },
            text_color: Color::from_rgb(0.3,0.72,1.0),
            ..Default::default()
        }
    } else {
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border { color: Color::TRANSPARENT, width: 0.0, radius: 5.0.into() },
            text_color: Color::from_rgb(0.28,0.4,0.52),
            ..Default::default()
        }
    }
}

fn btn_style(active: bool, danger: bool) -> iced::widget::button::Style {
    let (bg, border, tc) = if active && danger {
        (Color::from_rgba(0.94,0.33,0.31,0.18), Color::from_rgba(0.94,0.33,0.31,0.4), Color::from_rgb(0.94,0.33,0.31))
    } else if active {
        (Color::from_rgba(0.3,0.72,1.0,0.12), Color::from_rgba(0.3,0.72,1.0,0.3), Color::from_rgb(0.0,0.9,0.63))
    } else {
        (Color::TRANSPARENT, Color::TRANSPARENT, Color::from_rgb(0.28,0.4,0.52))
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border { color: border, width: 1.0, radius: 5.0.into() },
        text_color: tc,
        ..Default::default()
    }
}

fn color_from_u32(c: u32) -> Color {
    Color::from_rgba(
        ((c >> 24) & 0xff) as f32 / 255.0,
        ((c >> 16) & 0xff) as f32 / 255.0,
        ((c >>  8) & 0xff) as f32 / 255.0,
        ( c        & 0xff) as f32 / 255.0,
    )
}

// ── Instrument Panel ──────────────────────────────────────────────────────────

fn instrument_panel_el(track: &crate::app_state::Track, track_idx: usize) -> Element<'static, Message> {
    let p = track.instrument.clone();
    let c = color_from_u32(track.color);
    let name = track.name.clone();

    let waveforms = ["Sine", "Saw", "Square", "Tri"];
    let wave_row = waveforms.iter().enumerate().fold(row![].spacing(3), |r, (i, label)| {
        let active = (p.waveform - i as f32).abs() < 0.1;
        let idx = i;
        r.push(
            button(text(*label).size(9))
                .on_press(Message::SetTrackInstrumentParam {
                    track_idx,
                    param: InstrumentParam::Waveform(idx as f32),
                })
                .style(move |_,_| tab_style(active))
        )
    });

    let knob_row = row![
        knob_col("ATK",  p.attack,   0.001, 4.0,  track_idx, |v| InstrumentParam::Attack(v)),
        knob_col("DEC",  p.decay,    0.001, 4.0,  track_idx, |v| InstrumentParam::Decay(v)),
        knob_col("SUS",  p.sustain,  0.0,   1.0,  track_idx, |v| InstrumentParam::Sustain(v)),
        knob_col("REL",  p.release,  0.001, 4.0,  track_idx, |v| InstrumentParam::Release(v)),
        knob_col("CUT",  p.cutoff / 20000.0, 0.0, 1.0, track_idx, |v| InstrumentParam::Cutoff(v * 20000.0)),
        knob_col("RES",  p.resonance / 4.0, 0.0, 1.0, track_idx, |v| InstrumentParam::Resonance(v * 4.0)),
        knob_col("GAIN", p.gain,     0.0,   1.0,  track_idx, |v| InstrumentParam::Gain(v)),
    ].spacing(8);

    container(
        column![
            row![
                container(iced::widget::horizontal_space()).width(4).height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(c)),
                        ..Default::default()
                    }),
                column![
                    row![
                        text(name).size(11).color(c),
                        iced::widget::horizontal_space(),
                        button(text("✕").size(10)).on_press(Message::CloseInstrumentPanel)
                            .style(|_,_| btn_style(false, false)),
                    ].spacing(8).align_y(Alignment::Center),
                    wave_row,
                    knob_row,
                ].spacing(6).padding([4, 8]),
            ].height(Length::Fill),
        ],
    )
    .width(Length::Fill)
    .height(Length::Fixed(110.0))
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03, 0.06, 0.1))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into()
}

fn knob_col<F>(label: &'static str, value: f32, min: f32, max: f32, track_idx: usize, make_param: F) -> Element<'static, Message>
where
    F: Fn(f32) -> InstrumentParam + 'static,
{
    let norm = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let pct = (norm * 100.0) as u32;
    column![
        text(label).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
        // Knob as a vertical slider (iced doesn't have a rotary knob widget yet)
        container(
            iced::widget::slider(0.0..=1.0_f32, norm, move |v| {
                let real = min + v * (max - min);
                Message::SetTrackInstrumentParam { track_idx, param: make_param(real) }
            })
            .step(0.001)
        )
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(60.0)),
        text(format!("{}", pct)).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
    ]
    .spacing(2)
    .align_x(Alignment::Center)
    .into()
}

// ── Effects bar ───────────────────────────────────────────────────────────────

fn effects_bar_el(track: &crate::app_state::Track, track_idx: usize) -> Element<'static, Message> {
    use crate::app_state::EffectType;

    let effect_types = [
        (EffectType::Eq,         "EQ",     Color::from_rgb(0.3, 0.72, 1.0)),
        (EffectType::Compressor, "Comp",   Color::from_rgb(0.65, 0.55, 0.98)),
        (EffectType::Reverb,     "Reverb", Color::from_rgb(0.2, 0.83, 0.6)),
        (EffectType::Delay,      "Delay",  Color::from_rgb(0.98, 0.75, 0.14)),
        (EffectType::Filter,     "Filter", Color::from_rgb(0.98, 0.45, 0.09)),
    ];

    let add_buttons = effect_types.iter().fold(row![].spacing(4), |r, (et, label, col)| {
        let et2 = et.clone();
        let c = *col;
        r.push(
            button(text(format!("+ {}", label)).size(9))
                .on_press(Message::AddEffect { track_idx, effect_type: et2 })
                .style(move |_,_| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color { a: 0.08, ..c })),
                    border: iced::Border { color: Color { a: 0.25, ..c }, width: 1.0, radius: 4.0.into() },
                    text_color: c,
                    ..Default::default()
                })
        )
    });

    let fx_chips: Vec<Element<Message>> = track.effects.iter().map(|fx| {
        let label = fx.effect_type.label().to_string();
        let enabled = fx.enabled;
        let fid = fx.id;
        let col = match fx.effect_type {
            EffectType::Eq         => Color::from_rgb(0.3, 0.72, 1.0),
            EffectType::Compressor => Color::from_rgb(0.65, 0.55, 0.98),
            EffectType::Reverb     => Color::from_rgb(0.2, 0.83, 0.6),
            EffectType::Delay      => Color::from_rgb(0.98, 0.75, 0.14),
            EffectType::Filter     => Color::from_rgb(0.98, 0.45, 0.09),
        };
        let alpha = if enabled { 0.7 } else { 0.25 };
        container(
            row![
                button(text(label).size(9))
                    .on_press(Message::ToggleEffect { track_idx, effect_id: fid })
                    .style(move |_,_| iced::widget::button::Style {
                        background: Some(iced::Background::Color(Color { a: if enabled { 0.15 } else { 0.05 }, ..col })),
                        border: iced::Border { color: Color { a: alpha, ..col }, width: 1.0, radius: 4.0.into() },
                        text_color: Color { a: alpha, ..col },
                        ..Default::default()
                    }),
                button(text("✕").size(8))
                    .on_press(Message::RemoveEffect { track_idx, effect_id: fid })
                    .style(|_,_| btn_style(false, false)),
            ].spacing(2)
        ).into()
    }).collect();

    let track_name = track.name.clone();
    let track_color = color_from_u32(track.color);
    let tid = track.id;

    container(
        row![
            text(format!("FX: {}", track_name)).size(10).color(track_color),
            button(text("🎹 Synth").size(9))
                .on_press(Message::OpenInstrumentPanel(tid))
                .style(|_,_| btn_style(false, false)),
            iced::widget::vertical_rule(1),
            add_buttons,
            iced::widget::vertical_rule(1),
        ]
        .push(row(fx_chips).spacing(4))
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([4, 10]),
    )
    .width(Length::Fill)
    .height(Length::Fixed(36.0))
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03, 0.06, 0.1))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into()
}
