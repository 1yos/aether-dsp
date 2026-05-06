// placeholder
//! DawApp — Production-ready DAW application.
//! Sprint 1: Full Song view, Piano Roll, Mixer, Transport.
use iced::{
    widget::{button, canvas, column, container, row, scrollable, slider, text, text_input},
    Alignment, Color, Element, Length, Task, Theme, time,
};
use std::time::{Duration, Instant};
use crate::app_state::{
    AppState, ActiveView, Clip, MidiNote, PianoRollTool, SongTool,
    EffectType,
};
use crate::instrument::{MidiEvent, InstrumentPreset};

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    // Transport
    SetView(ActiveView),
    Play, Stop, ToggleRecord,
    SetBpm(f32), TapTempoClick,
    ToggleLoop, ToggleMetronome,
    Tick(Instant),

    // Song view — canvas interactions
    SongMouseDown { x: f32, y: f32, right: bool },
    SongMouseMove { x: f32, y: f32 },
    SongMouseUp   { x: f32, y: f32 },
    SongScroll    { delta_x: f32, delta_y: f32, ctrl: bool },
    SongZoom(f32),

    // Song view — toolbar
    AddTrack,
    SetSongTool(SongTool),
    SetSongSnap(f64),

    // Song view — clip operations
    OpenPianoRoll   { clip_id: u64, track_id: u64 },
    DeleteClip(u64),
    DuplicateClip(u64),
    SplitClip       { clip_id: u64, at_beat: f64 },
    RenameClip      { clip_id: u64, name: String },
    SetClipColor    { clip_id: u64, color: u32 },

    // Song view — track operations
    RenameTrack     { track_id: u64, name: String },
    SetTrackColor   { track_id: u64, color: u32 },
    RemoveTrack(u64),
    MoveTrackUp(u64),
    MoveTrackDown(u64),
    ResizeTrack     { track_id: u64, height: f32 },
    SetTrackVolume  { track_idx: usize, volume: f32 },
    SetTrackPan     { track_idx: usize, pan: f32 },
    SetTrackMute    { track_idx: usize },
    SetTrackSolo    { track_idx: usize },
    ArmTrack        { track_idx: usize },

    // Context menu
    ShowContextMenu { x: f32, y: f32, kind: ContextMenuKind },
    HideContextMenu,
    ContextAction(ContextAction),

    // Track rename inline
    StartRenameTrack(u64),
    CommitRenameTrack,
    RenameInput(String),

    // Piano Roll — canvas interactions
    PianoRollMouseDown { x: f32, y: f32, right: bool },
    PianoRollMouseMove { x: f32, y: f32 },
    PianoRollMouseUp   { x: f32, y: f32 },
    PianoRollScroll    { delta_x: f32, delta_y: f32, ctrl: bool },

    // Piano Roll — toolbar
    SetPianoTool(PianoRollTool),
    SetPianoSnap(f64),
    QuantizeNotes,
    TransposeUp, TransposeDown,
    TransposeOctaveUp, TransposeOctaveDown,
    SetScale(Scale),
    ClosePianoRoll,

    // Piano Roll — keyboard
    PianoKeyPress(u8),
    PianoKeyRelease(u8),

    // Velocity lane
    VelocityMouseDown { note_id: u64, y: f32 },
    VelocityMouseMove { y: f32 },
    VelocityMouseUp,

    // Mixer
    MixerFaderMove  { track_idx: usize, value: f32 },
    MixerPanMove    { track_idx: usize, value: f32 },

    // Instrument panel
    OpenInstrumentPanel(u64),
    CloseInstrumentPanel,
    SetInstrumentParam { track_idx: usize, param: InstrumentParam },

    // Effects
    AddEffect       { track_idx: usize, effect_type: EffectType },
    RemoveEffect    { track_idx: usize, effect_id: u64 },
    ToggleEffect    { track_idx: usize, effect_id: u64 },
    SetEffectParam  { track_idx: usize, effect_id: u64, param: EffectParam },

    // Keyboard shortcuts
    Undo, Redo,
    Copy, Paste, Duplicate, Cut,
    DeleteSelected, SelectAll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuKind {
    Clip(u64),
    Track(u64),
    Timeline { beat: f64, track_idx: usize },
}

#[derive(Debug, Clone)]
pub enum ContextAction {
    DeleteClip(u64),
    DuplicateClip(u64),
    SplitClip { clip_id: u64, at_beat: f64 },
    RenameClip(u64),
    OpenPianoRoll { clip_id: u64, track_id: u64 },
    DeleteTrack(u64),
    RenameTrack(u64),
    MoveTrackUp(u64),
    MoveTrackDown(u64),
    AddInstrumentTrack,
    AddAudioTrack,
}

#[derive(Debug, Clone)]
pub enum InstrumentParam {
    Waveform(f32), Attack(f32), Decay(f32), Sustain(f32), Release(f32),
    Cutoff(f32), Resonance(f32), Gain(f32),
    LfoRate(f32), LfoDepth(f32), LfoWaveform(f32),
    Detune(f32), Unison(u8),
}

#[derive(Debug, Clone)]
pub enum EffectParam {
    // EQ
    EqLowGain(f32), EqMidGain(f32), EqHighGain(f32), EqMidFreq(f32),
    // Compressor
    CompThreshold(f32), CompRatio(f32), CompAttack(f32), CompRelease(f32), CompMakeup(f32),
    // Reverb
    ReverbRoom(f32), ReverbDamp(f32), ReverbWet(f32),
    // Delay
    DelayTime(f32), DelayFeedback(f32), DelayWet(f32),
    // Filter
    FilterCutoff(f32), FilterResonance(f32), FilterMode(u8),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    Dorian,
    Mixolydian,
    Pentatonic,
    Blues,
}

impl Scale {
    pub fn semitones(&self) -> &'static [u8] {
        match self {
            Scale::Chromatic    => &[0,1,2,3,4,5,6,7,8,9,10,11],
            Scale::Major        => &[0,2,4,5,7,9,11],
            Scale::Minor        => &[0,2,3,5,7,8,10],
            Scale::Dorian       => &[0,2,3,5,7,9,10],
            Scale::Mixolydian   => &[0,2,4,5,7,9,10],
            Scale::Pentatonic   => &[0,2,4,7,9],
            Scale::Blues        => &[0,3,5,6,7,10],
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Scale::Chromatic  => "Chromatic",
            Scale::Major      => "Major",
            Scale::Minor      => "Minor",
            Scale::Dorian     => "Dorian",
            Scale::Mixolydian => "Mixolydian",
            Scale::Pentatonic => "Pentatonic",
            Scale::Blues      => "Blues",
        }
    }
}

// ── Song canvas interaction state ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum SongDragMode {
    None,
    CreatingClip { clip_id: u64, track_idx: usize },
    MovingClip   { clip_id: u64, offset_beat: f64 },
    ResizingClip { clip_id: u64, original_len: f64 },
    ResizingTrack { track_id: u64, start_y: f32, start_h: f32 },
    RubberBand   { start_beat: f64, start_track: usize, end_beat: f64, end_track: usize },
    SeekingPlayhead,
}

// ── Piano Roll canvas interaction state ───────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum PrDragMode {
    None,
    DrawingNote  { note_id: u64, start_beat: f64 },
    MovingNote   { note_id: u64, start_beat: f64, start_pitch: u8 },
    ResizingNote { note_id: u64, original_len: f64 },
    RubberBand   { start_beat: f64, start_pitch: u8 },
    EditVelocity { note_id: u64, start_y: f32, start_vel: u8 },
}

// ── Context menu ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ContextMenu {
    x: f32,
    y: f32,
    kind: ContextMenuKind,
}

// ── DawApp ────────────────────────────────────────────────────────────────────

pub struct DawApp {
    pub state: AppState,
    last_tick: Instant,

    // Song view interaction
    song_drag: SongDragMode,
    song_scroll_x: f32,   // pixel offset
    song_zoom_x: f32,     // pixels per beat

    // Piano Roll interaction
    pr_drag: PrDragMode,
    pr_scroll_x: f32,
    pr_scroll_y: f32,
    pr_zoom_x: f32,
    pr_zoom_y: f32,
    pr_scale: Scale,
    pr_velocity_drag_note: Option<u64>,
    pr_velocity_drag_start_y: f32,
    pr_velocity_drag_start_vel: u8,

    // Context menu
    context_menu: Option<ContextMenu>,

    // Inline rename
    renaming_track_id: Option<u64>,
    rename_value: String,

    // VU meter levels (updated from audio thread via shared state)
    vu_levels: Vec<f32>,

    // Metronome
    metronome_on: bool,
}

impl DawApp {
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        let track_count = state.lock().unwrap().tracks.len();
        let app = Self {
            state,
            last_tick: Instant::now(),
            song_drag: SongDragMode::None,
            song_scroll_x: 0.0,
            song_zoom_x: 32.0,
            pr_drag: PrDragMode::None,
            pr_scroll_x: 0.0,
            pr_scroll_y: 48.0 * 14.0, // start at C3
            pr_zoom_x: 80.0,
            pr_zoom_y: 14.0,
            pr_scale: Scale::Chromatic,
            pr_velocity_drag_note: None,
            pr_velocity_drag_start_y: 0.0,
            pr_velocity_drag_start_vel: 100,
            context_menu: None,
            renaming_track_id: None,
            rename_value: String::new(),
            vu_levels: vec![0.0; track_count],
            metronome_on: false,
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
                keyboard::Key::Character("x") if modifiers.control() => Some(Message::Cut),
                keyboard::Key::Character("a") if modifiers.control() => Some(Message::SelectAll),
                keyboard::Key::Named(Named::Delete) | keyboard::Key::Named(Named::Backspace) => {
                    Some(Message::DeleteSelected)
                }
                keyboard::Key::Named(Named::ArrowUp) if modifiers.shift() => Some(Message::TransposeUp),
                keyboard::Key::Named(Named::ArrowDown) if modifiers.shift() => Some(Message::TransposeDown),
                keyboard::Key::Named(Named::ArrowUp) if modifiers.control() => Some(Message::TransposeOctaveUp),
                keyboard::Key::Named(Named::ArrowDown) if modifiers.control() => Some(Message::TransposeOctaveDown),
                keyboard::Key::Character("q") if modifiers.control() => Some(Message::QuantizeNotes),
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
            // ── Tick ─────────────────────────────────────────────────────
            Message::Tick(now) => {
                let delta = now.duration_since(self.last_tick).as_secs_f64();
                self.last_tick = now;
                self.state.lock().unwrap().tick_transport(delta);
            }

            // ── Transport ─────────────────────────────────────────────────
            Message::SetView(v) => {
                self.context_menu = None;
                self.state.lock().unwrap().active_view = v;
            }
            Message::Play => {
                let mut s = self.state.lock().unwrap();
                if s.transport.is_playing {
                    // Second press = stop and return to start
                    s.transport.is_playing = false;
                    s.transport.playhead_beat = 0.0;
                } else {
                    s.transport.is_playing = true;
                }
            }
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
            Message::TapTempoClick => {
                let mut s = self.state.lock().unwrap();
                if let Some(bpm) = s.tap_tempo.tap() {
                    s.transport.bpm = bpm.clamp(20.0, 300.0);
                }
            }
            Message::ToggleLoop => {
                let mut s = self.state.lock().unwrap();
                s.transport.loop_enabled = !s.transport.loop_enabled;
            }
            Message::ToggleMetronome => {
                self.metronome_on = !self.metronome_on;
            }

            // ── Song view — canvas ────────────────────────────────────────
            Message::SongMouseDown { x, y, right } => {
                self.context_menu = None;
                self.renaming_track_id = None;
                let s = self.state.lock().unwrap();
                let snap = s.song_view.snap_beats;
                let tool = s.song_view.tool.clone();
                let tracks = s.tracks.clone();
                drop(s);

                let ruler_h = 28.0f32;
                let header_w = 180.0f32;
                let bw = self.song_zoom_x;

                // Click on ruler = seek
                if y < ruler_h && !right {
                    let beat = ((x + self.song_scroll_x) / bw) as f64;
                    self.state.lock().unwrap().transport.playhead_beat = beat.max(0.0);
                    self.song_drag = SongDragMode::SeekingPlayhead;
                    return Task::none();
                }

                // Click in track area
                let beat = ((x + self.song_scroll_x) / bw) as f64;
                let track_idx = self.track_at_y(&tracks, y - ruler_h);

                if right {
                    // Right click = context menu
                    if let Some(idx) = track_idx {
                        let track = &tracks[idx];
                        // Check if clicking on a clip
                        let clip_id = track.clips.iter()
                            .find(|c| beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                            .map(|c| c.id);
                        let kind = if let Some(cid) = clip_id {
                            ContextMenuKind::Clip(cid)
                        } else {
                            ContextMenuKind::Timeline { beat, track_idx: idx }
                        };
                        self.context_menu = Some(ContextMenu { x, y, kind });
                    }
                    return Task::none();
                }

                if let Some(idx) = track_idx {
                    let track = &tracks[idx];
                    let snapped = snap_beat(beat, snap);

                    match tool {
                        SongTool::Draw => {
                            // Check if clicking near right edge of existing clip (resize)
                            let resize_clip = track.clips.iter().find(|c| {
                                let end_x = (c.start_beat + c.length_beats) as f32 * bw - self.song_scroll_x;
                                (x - end_x).abs() < 8.0 && beat >= c.start_beat && beat <= c.start_beat + c.length_beats + 0.5
                            }).map(|c| (c.id, c.length_beats));

                            if let Some((cid, orig_len)) = resize_clip {
                                self.song_drag = SongDragMode::ResizingClip { clip_id: cid, original_len: orig_len };
                            } else {
                                // Check if clicking on existing clip (move)
                                let existing = track.clips.iter()
                                    .find(|c| beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                                    .map(|c| (c.id, c.start_beat));

                                if let Some((cid, cstart)) = existing {
                                    self.song_drag = SongDragMode::MovingClip {
                                        clip_id: cid,
                                        offset_beat: beat - cstart,
                                    };
                                    let mut s = self.state.lock().unwrap();
                                    s.selected_clip_id = Some(cid);
                                    s.song_view.selected_clip_ids = vec![cid];
                                } else {
                                    // Create new clip
                                    let mut s = self.state.lock().unwrap();
                                    s.push_undo("Create Clip");
                                    let id = s.next_id();
                                    let track_id = s.tracks[idx].id;
                                    let color = s.tracks[idx].color;
                                    s.tracks[idx].clips.push(Clip {
                                        id, track_id,
                                        name: format!("Clip {}", id),
                                        start_beat: snapped,
                                        length_beats: snap * 4.0,
                                        color, notes: Vec::new(),
                                    });
                                    s.selected_clip_id = Some(id);
                                    s.song_view.selected_clip_ids = vec![id];
                                    self.song_drag = SongDragMode::CreatingClip { clip_id: id, track_idx: idx };
                                }
                            }
                        }
                        SongTool::Select => {
                            let existing = track.clips.iter()
                                .find(|c| beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                                .map(|c| c.id);
                            if let Some(cid) = existing {
                                let mut s = self.state.lock().unwrap();
                                s.selected_clip_id = Some(cid);
                                s.song_view.selected_clip_ids = vec![cid];
                                let cstart = s.find_clip(cid).map(|c| c.start_beat).unwrap_or(0.0);
                                drop(s);
                                self.song_drag = SongDragMode::MovingClip {
                                    clip_id: cid,
                                    offset_beat: beat - cstart,
                                };
                            } else {
                                // Start rubber-band
                                self.song_drag = SongDragMode::RubberBand {
                                    start_beat: beat,
                                    start_track: idx,
                                    end_beat: beat,
                                    end_track: idx,
                                };
                                self.state.lock().unwrap().song_view.selected_clip_ids.clear();
                            }
                        }
                        SongTool::Erase => {
                            let mut s = self.state.lock().unwrap();
                            s.push_undo("Erase Clip");
                            s.tracks[idx].clips.retain(|c| {
                                !(beat >= c.start_beat && beat < c.start_beat + c.length_beats)
                            });
                        }
                    }
                }
            }

            Message::SongMouseMove { x, y } => {
                let bw = self.song_zoom_x;
                let beat = ((x + self.song_scroll_x) / bw) as f64;
                let snap = self.state.lock().unwrap().song_view.snap_beats;

                match &self.song_drag.clone() {
                    SongDragMode::SeekingPlayhead => {
                        self.state.lock().unwrap().transport.playhead_beat = beat.max(0.0);
                    }
                    SongDragMode::CreatingClip { clip_id, track_idx } => {
                        let cid = *clip_id;
                        let mut s = self.state.lock().unwrap();
                        if let Some(clip) = s.find_clip_mut(cid) {
                            let new_end = snap_beat(beat, snap).max(clip.start_beat + snap);
                            clip.length_beats = (new_end - clip.start_beat).max(snap);
                        }
                    }
                    SongDragMode::MovingClip { clip_id, offset_beat } => {
                        let cid = *clip_id;
                        let off = *offset_beat;
                        let mut s = self.state.lock().unwrap();
                        if let Some(clip) = s.find_clip_mut(cid) {
                            clip.start_beat = snap_beat(beat - off, snap).max(0.0);
                        }
                    }
                    SongDragMode::ResizingClip { clip_id, .. } => {
                        let cid = *clip_id;
                        let mut s = self.state.lock().unwrap();
                        if let Some(clip) = s.find_clip_mut(cid) {
                            let new_end = snap_beat(beat, snap).max(clip.start_beat + snap);
                            clip.length_beats = (new_end - clip.start_beat).max(snap);
                        }
                    }
                    SongDragMode::RubberBand { start_beat, start_track, .. } => {
                        let sb = *start_beat;
                        let st = *start_track;
                        let tracks = self.state.lock().unwrap().tracks.clone();
                        let end_track = self.track_at_y(&tracks, y - 28.0).unwrap_or(st);
                        self.song_drag = SongDragMode::RubberBand {
                            start_beat: sb, start_track: st,
                            end_beat: beat, end_track,
                        };
                        // Update selection
                        let min_beat = sb.min(beat);
                        let max_beat = sb.max(beat);
                        let min_track = st.min(end_track);
                        let max_track = st.max(end_track);
                        let mut s = self.state.lock().unwrap();
                        s.song_view.selected_clip_ids = s.tracks.iter()
                            .enumerate()
                            .filter(|(i, _)| *i >= min_track && *i <= max_track)
                            .flat_map(|(_, t)| t.clips.iter())
                            .filter(|c| c.start_beat < max_beat && c.start_beat + c.length_beats > min_beat)
                            .map(|c| c.id)
                            .collect();
                    }
                    _ => {}
                }
            }

            Message::SongMouseUp { .. } => {
                if let SongDragMode::MovingClip { .. } | SongDragMode::CreatingClip { .. } | SongDragMode::ResizingClip { .. } = &self.song_drag {
                    self.state.lock().unwrap().push_undo("Move/Resize Clip");
                }
                self.song_drag = SongDragMode::None;
            }

            Message::SongScroll { delta_x, delta_y, ctrl } => {
                if ctrl {
                    // Zoom
                    self.song_zoom_x = (self.song_zoom_x * (1.0 + delta_y * 0.1)).clamp(8.0, 256.0);
                } else {
                    self.song_scroll_x = (self.song_scroll_x + delta_x * 32.0).max(0.0);
                }
            }
            Message::SongZoom(z) => {
                self.song_zoom_x = z.clamp(8.0, 256.0);
            }

            // ── Song view — toolbar ───────────────────────────────────────
            Message::AddTrack => {
                let mut s = self.state.lock().unwrap();
                s.push_undo("Add Track");
                let id = s.next_id();
                let idx = s.tracks.len();
                let colors = [0x4db8ffff_u32,0xa78bfaff,0x34d399ff,0xf97316ff,
                              0xf43f5eff,0xfbbf24ff,0x06b6d4ff,0x8b5cf6ff];
                let presets = [
                    InstrumentPreset::kick(), InstrumentPreset::bass(),
                    InstrumentPreset::lead(), InstrumentPreset::pad(),
                ];
                s.tracks.push(crate::app_state::Track {
                    id, name: format!("Track {}", idx + 1),
                    track_type: crate::app_state::TrackType::Instrument,
                    color: colors[idx % colors.len()],
                    muted: false, solo: false, armed: false,
                    volume: 0.8, pan: 0.0, height: 72.0,
                    clips: Vec::new(), effects: Vec::new(),
                    instrument: presets[idx % presets.len()].clone(),
                });
                let sched_arc = s.scheduler.clone();
                if let Ok(mut sched) = sched_arc.try_lock() {
                    if let Some(ref mut engine) = s.master_engine {
                        engine.ensure_track(&mut sched, idx);
                    }
                };
            }
            Message::SetSongTool(t) => { self.state.lock().unwrap().song_view.tool = t; }
            Message::SetSongSnap(sn) => { self.state.lock().unwrap().song_view.snap_beats = sn; }

            // ── Song view — clip operations ───────────────────────────────
            Message::OpenPianoRoll { clip_id, track_id } => {
                let mut s = self.state.lock().unwrap();
                s.piano_roll.open_clip_id = Some(clip_id);
                s.piano_roll.open_track_id = Some(track_id);
                s.active_view = ActiveView::PianoRoll;
            }
            Message::DeleteClip(id) => {
                let mut s = self.state.lock().unwrap();
                s.push_undo("Delete Clip");
                for track in &mut s.tracks { track.clips.retain(|c| c.id != id); }
            }
            Message::DuplicateClip(id) => {
                let mut s = self.state.lock().unwrap();
                s.push_undo("Duplicate Clip");
                let clip = s.tracks.iter().flat_map(|t| t.clips.iter())
                    .find(|c| c.id == id).cloned();
                if let Some(clip) = clip {
                    let new_id = s.next_id();
                    let new_start = clip.start_beat + clip.length_beats;
                    if let Some(track) = s.tracks.iter_mut().find(|t| t.id == clip.track_id) {
                        track.clips.push(Clip { id: new_id, start_beat: new_start, ..clip });
                    }
                }
            }
            Message::SplitClip { clip_id, at_beat } => {
                let mut s = self.state.lock().unwrap();
                s.push_undo("Split Clip");
                let clip = s.tracks.iter().flat_map(|t| t.clips.iter())
                    .find(|c| c.id == clip_id).cloned();
                if let Some(clip) = clip {
                    if at_beat > clip.start_beat && at_beat < clip.start_beat + clip.length_beats {
                        let new_id = s.next_id();
                        let left_len = at_beat - clip.start_beat;
                        let right_start = at_beat;
                        let right_len = clip.length_beats - left_len;
                        // Split notes
                        let left_notes: Vec<MidiNote> = clip.notes.iter()
                            .filter(|n| n.beat < left_len)
                            .cloned().collect();
                        let right_notes: Vec<MidiNote> = clip.notes.iter()
                            .filter(|n| n.beat >= left_len)
                            .map(|n| MidiNote { beat: n.beat - left_len, ..n.clone() })
                            .collect();
                        if let Some(track) = s.tracks.iter_mut().find(|t| t.id == clip.track_id) {
                            if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                                c.length_beats = left_len;
                                c.notes = left_notes;
                            }
                            track.clips.push(Clip {
                                id: new_id, track_id: clip.track_id,
                                name: clip.name.clone(),
                                start_beat: right_start, length_beats: right_len,
                                color: clip.color, notes: right_notes,
                            });
                        }
                    }
                }
            }
            Message::RenameClip { clip_id, name } => {
                let mut s = self.state.lock().unwrap();
                if let Some(clip) = s.find_clip_mut(clip_id) { clip.name = name; }
            }
            Message::SetClipColor { clip_id, color } => {
                let mut s = self.state.lock().unwrap();
                if let Some(clip) = s.find_clip_mut(clip_id) { clip.color = color; }
            }

            // ── Song view — track operations ──────────────────────────────
            Message::RenameTrack { track_id, name } => {
                let mut s = self.state.lock().unwrap();
                if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) { t.name = name; }
            }
            Message::SetTrackColor { track_id, color } => {
                let mut s = self.state.lock().unwrap();
                if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) { t.color = color; }
            }
            Message::RemoveTrack(id) => {
                let mut s = self.state.lock().unwrap();
                s.push_undo("Remove Track");
                s.tracks.retain(|t| t.id != id);
            }
            Message::MoveTrackUp(id) => {
                let mut s = self.state.lock().unwrap();
                if let Some(i) = s.tracks.iter().position(|t| t.id == id) {
                    if i > 0 { s.tracks.swap(i, i - 1); }
                }
            }
            Message::MoveTrackDown(id) => {
                let mut s = self.state.lock().unwrap();
                let len = s.tracks.len();
                if let Some(i) = s.tracks.iter().position(|t| t.id == id) {
                    if i < len - 1 { s.tracks.swap(i, i + 1); }
                }
            }
            Message::ResizeTrack { track_id, height } => {
                let mut s = self.state.lock().unwrap();
                if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                    t.height = height.clamp(48.0, 200.0);
                }
            }
            Message::SetTrackVolume { track_idx, volume } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.tracks[track_idx].volume = volume;
                    let sched_arc = s.scheduler.clone();
                    if let Ok(mut sched) = sched_arc.try_lock() {
                        if let Some(ref mut engine) = s.master_engine {
                            if let Some(slot) = engine.tracks.get_mut(track_idx) {
                                if let Some(ref mut te) = slot { te.set_volume(&mut sched, volume); }
                            }
                        }
                    };
                }
            }
            Message::SetTrackPan { track_idx, pan } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() { s.tracks[track_idx].pan = pan; }
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
            Message::ArmTrack { track_idx } => {
                let mut s = self.state.lock().unwrap();
                if track_idx < s.tracks.len() {
                    s.tracks[track_idx].armed = !s.tracks[track_idx].armed;
                }
            }

            // ── Context menu ──────────────────────────────────────────────
            Message::ShowContextMenu { x, y, kind } => {
                self.context_menu = Some(ContextMenu { x, y, kind });
            }
            Message::HideContextMenu => { self.context_menu = None; }
            Message::ContextAction(action) => {
                self.context_menu = None;
                match action {
                    ContextAction::DeleteClip(id) => {
                        let mut s = self.state.lock().unwrap();
                        s.push_undo("Delete Clip");
                        for t in &mut s.tracks { t.clips.retain(|c| c.id != id); }
                    }
                    ContextAction::DuplicateClip(id) => {
                        return self.update(Message::DuplicateClip(id));
                    }
                    ContextAction::SplitClip { clip_id, at_beat } => {
                        return self.update(Message::SplitClip { clip_id, at_beat });
                    }
                    ContextAction::RenameClip(id) => {
                        // TODO: inline rename
                    }
                    ContextAction::OpenPianoRoll { clip_id, track_id } => {
                        return self.update(Message::OpenPianoRoll { clip_id, track_id });
                    }
                    ContextAction::DeleteTrack(id) => {
                        return self.update(Message::RemoveTrack(id));
                    }
                    ContextAction::RenameTrack(id) => {
                        let name = self.state.lock().unwrap().tracks.iter()
                            .find(|t| t.id == id).map(|t| t.name.clone()).unwrap_or_default();
                        self.renaming_track_id = Some(id);
                        self.rename_value = name;
                    }
                    ContextAction::MoveTrackUp(id) => {
                        return self.update(Message::MoveTrackUp(id));
                    }
                    ContextAction::MoveTrackDown(id) => {
                        return self.update(Message::MoveTrackDown(id));
                    }
                    ContextAction::AddInstrumentTrack => {
                        return self.update(Message::AddTrack);
                    }
                    ContextAction::AddAudioTrack => {
                        return self.update(Message::AddTrack);
                    }
                }
            }

            // ── Track rename ──────────────────────────────────────────────
            Message::StartRenameTrack(id) => {
                let name = self.state.lock().unwrap().tracks.iter()
                    .find(|t| t.id == id).map(|t| t.name.clone()).unwrap_or_default();
                self.renaming_track_id = Some(id);
                self.rename_value = name;
            }
            Message::CommitRenameTrack => {
                if let Some(id) = self.renaming_track_id.take() {
                    let name = self.rename_value.clone();
                    let mut s = self.state.lock().unwrap();
                    if let Some(t) = s.tracks.iter_mut().find(|t| t.id == id) { t.name = name; }
                }
            }
            Message::RenameInput(v) => { self.rename_value = v; }

            // ── Piano Roll — canvas ───────────────────────────────────────
            Message::PianoRollMouseDown { x, y, right } => {
                let s = self.state.lock().unwrap();
                let snap = s.piano_roll.snap_beats;
                let default_len = s.piano_roll.default_note_len;
                let tool = s.piano_roll.tool.clone();
                let clip_id = s.piano_roll.open_clip_id;
                let track_id = s.piano_roll.open_track_id;
                drop(s);

                let bw = self.pr_zoom_x;
                let kh = self.pr_zoom_y;
                let beat = ((x + self.pr_scroll_x) / bw) as f64;
                let key_row = ((y + self.pr_scroll_y) / kh) as usize;
                let pitch = (127usize.saturating_sub(key_row)) as u8;
                let snapped = snap_beat(beat, snap);

                if right {
                    // Right-click = erase note
                    if let Some(cid) = clip_id {
                        let mut s = self.state.lock().unwrap();
                        s.push_undo("Erase Note");
                        if let Some(clip) = s.find_clip_mut(cid) {
                            clip.notes.retain(|n| {
                                !(n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration)
                            });
                        }
                    }
                    return Task::none();
                }

                match tool {
                    PianoRollTool::Draw => {
                        if let Some(cid) = clip_id {
                            // Check if clicking on existing note (resize from right edge)
                            let resize_note = {
                                let s = self.state.lock().unwrap();
                                s.find_clip(cid).and_then(|clip| {
                                    clip.notes.iter().find(|n| {
                                        let end_x = (n.beat + n.duration) as f32 * bw - self.pr_scroll_x;
                                        n.pitch == pitch && (x - end_x).abs() < 6.0
                                    }).map(|n| (n.id, n.duration))
                                })
                            };

                            if let Some((nid, orig_len)) = resize_note {
                                self.pr_drag = PrDragMode::ResizingNote { note_id: nid, original_len: orig_len };
                            } else {
                                // Check if clicking on existing note body (move)
                                let existing = {
                                    let s = self.state.lock().unwrap();
                                    s.find_clip(cid).and_then(|clip| {
                                        clip.notes.iter().find(|n| {
                                            n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration
                                        }).map(|n| (n.id, n.beat))
                                    })
                                };

                                if let Some((nid, nbeat)) = existing {
                                    self.pr_drag = PrDragMode::MovingNote {
                                        note_id: nid, start_beat: beat, start_pitch: pitch,
                                    };
                                    let mut s = self.state.lock().unwrap();
                                    s.piano_roll.selected_note_ids = vec![nid];
                                } else {
                                    // Draw new note
                                    let mut s = self.state.lock().unwrap();
                                    s.push_undo("Draw Note");
                                    let note_id = s.next_id();
                                    if let Some(clip) = s.find_clip_mut(cid) {
                                        clip.notes.push(MidiNote {
                                            id: note_id, pitch,
                                            beat: snapped, duration: default_len, velocity: 100,
                                        });
                                    }
                                    s.piano_roll.selected_note_ids = vec![note_id];
                                    drop(s);
                                    self.pr_drag = PrDragMode::DrawingNote { note_id, start_beat: snapped };
                                    // Preview sound
                                    if let Some(tid) = track_id {
                                        let mut s = self.state.lock().unwrap();
                                        let track_idx = s.tracks.iter().position(|t| t.id == tid).unwrap_or(0);
                                        s.midi_queue.push(crate::app_state::PendingMidi {
                                            track_idx, event: MidiEvent::NoteOn { pitch, velocity: 100 },
                                        });
                                        s.flush_midi();
                                    }
                                }
                            }
                        }
                    }
                    PianoRollTool::Select => {
                        if let Some(cid) = clip_id {
                            let existing = {
                                let s = self.state.lock().unwrap();
                                s.find_clip(cid).and_then(|clip| {
                                    clip.notes.iter().find(|n| {
                                        n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration
                                    }).map(|n| n.id)
                                })
                            };
                            if let Some(nid) = existing {
                                let mut s = self.state.lock().unwrap();
                                if !s.piano_roll.selected_note_ids.contains(&nid) {
                                    s.piano_roll.selected_note_ids = vec![nid];
                                }
                                drop(s);
                                self.pr_drag = PrDragMode::MovingNote {
                                    note_id: nid, start_beat: beat, start_pitch: pitch,
                                };
                            } else {
                                self.pr_drag = PrDragMode::RubberBand { start_beat: beat, start_pitch: pitch };
                                self.state.lock().unwrap().piano_roll.selected_note_ids.clear();
                            }
                        }
                    }
                    PianoRollTool::Erase => {
                        if let Some(cid) = clip_id {
                            let mut s = self.state.lock().unwrap();
                            s.push_undo("Erase Note");
                            if let Some(clip) = s.find_clip_mut(cid) {
                                clip.notes.retain(|n| {
                                    !(n.pitch == pitch && beat >= n.beat && beat < n.beat + n.duration)
                                });
                            }
                        }
                    }
                }
            }

            Message::PianoRollMouseMove { x, y } => {
                let bw = self.pr_zoom_x;
                let kh = self.pr_zoom_y;
                let beat = ((x + self.pr_scroll_x) / bw) as f64;
                let key_row = ((y + self.pr_scroll_y) / kh) as usize;
                let pitch = (127usize.saturating_sub(key_row)) as u8;
                let snap = self.state.lock().unwrap().piano_roll.snap_beats;

                match &self.pr_drag.clone() {
                    PrDragMode::DrawingNote { note_id, start_beat } => {
                        let nid = *note_id;
                        let sb = *start_beat;
                        let mut s = self.state.lock().unwrap();
                        if let Some(cid) = s.piano_roll.open_clip_id {
                            if let Some(clip) = s.find_clip_mut(cid) {
                                if let Some(note) = clip.notes.iter_mut().find(|n| n.id == nid) {
                                    let new_end = snap_beat(beat, snap).max(sb + snap);
                                    note.duration = (new_end - sb).max(snap);
                                }
                            }
                        }
                    }
                    PrDragMode::MovingNote { note_id, start_beat, start_pitch } => {
                        let nid = *note_id;
                        let db = beat - start_beat;
                        let dp = pitch as i16 - *start_pitch as i16;
                        let mut s = self.state.lock().unwrap();
                        if let Some(cid) = s.piano_roll.open_clip_id {
                            // Clone selected IDs before mutable borrow
                            let selected = s.piano_roll.selected_note_ids.clone();
                            for track in &mut s.tracks {
                                if let Some(clip) = track.clips.iter_mut().find(|c| c.id == cid) {
                                    for note in &mut clip.notes {
                                        if selected.contains(&note.id) {
                                            note.beat = snap_beat((note.beat + db).max(0.0), snap);
                                            note.pitch = (note.pitch as i16 + dp).clamp(0, 127) as u8;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        self.pr_drag = PrDragMode::MovingNote { note_id: nid, start_beat: beat, start_pitch: pitch };
                    }
                    PrDragMode::ResizingNote { note_id, .. } => {
                        let nid = *note_id;
                        let mut s = self.state.lock().unwrap();
                        if let Some(cid) = s.piano_roll.open_clip_id {
                            if let Some(clip) = s.find_clip_mut(cid) {
                                if let Some(note) = clip.notes.iter_mut().find(|n| n.id == nid) {
                                    let new_end = snap_beat(beat, snap).max(note.beat + snap);
                                    note.duration = (new_end - note.beat).max(snap);
                                }
                            }
                        }
                    }
                    PrDragMode::RubberBand { start_beat, start_pitch } => {
                        let sb = *start_beat;
                        let sp = *start_pitch;
                        let min_beat = sb.min(beat);
                        let max_beat = sb.max(beat);
                        let min_pitch = sp.min(pitch);
                        let max_pitch = sp.max(pitch);
                        let mut s = self.state.lock().unwrap();
                        if let Some(cid) = s.piano_roll.open_clip_id {
                            if let Some(clip) = s.find_clip(cid) {
                                s.piano_roll.selected_note_ids = clip.notes.iter()
                                    .filter(|n| {
                                        n.pitch >= min_pitch && n.pitch <= max_pitch &&
                                        n.beat < max_beat && n.beat + n.duration > min_beat
                                    })
                                    .map(|n| n.id)
                                    .collect();
                            }
                        }
                    }
                    _ => {}
                }
            }

            Message::PianoRollMouseUp { .. } => {
                if let PrDragMode::DrawingNote { note_id, .. } | PrDragMode::MovingNote { note_id, .. } = &self.pr_drag {
                    let nid = *note_id;
                    let s = self.state.lock().unwrap();
                    if let Some(tid) = s.piano_roll.open_track_id {
                        let track_idx = s.tracks.iter().position(|t| t.id == tid).unwrap_or(0);
                        let pitch = s.piano_roll.open_clip_id
                            .and_then(|cid| s.find_clip(cid))
                            .and_then(|clip| clip.notes.iter().find(|n| n.id == nid))
                            .map(|n| n.pitch).unwrap_or(60);
                        drop(s);
                        let mut s = self.state.lock().unwrap();
                        s.midi_queue.push(crate::app_state::PendingMidi {
                            track_idx, event: MidiEvent::NoteOff { pitch },
                        });
                        s.flush_midi();
                    }
                }
                self.pr_drag = PrDragMode::None;
            }

            Message::PianoRollScroll { delta_x, delta_y, ctrl } => {
                if ctrl {
                    self.pr_zoom_x = (self.pr_zoom_x * (1.0 + delta_y * 0.1)).clamp(20.0, 400.0);
                } else {
                    self.pr_scroll_x = (self.pr_scroll_x + delta_x * 32.0).max(0.0);
                    self.pr_scroll_y = (self.pr_scroll_y + delta_y * 32.0).max(0.0);
                }
            }

            // ── Piano Roll — toolbar ──────────────────────────────────────
            Message::SetPianoTool(t) => { self.state.lock().unwrap().piano_roll.tool = t; }
            Message::SetPianoSnap(sn) => { self.state.lock().unwrap().piano_roll.snap_beats = sn; }
            Message::QuantizeNotes => {
                let mut s = self.state.lock().unwrap();
                let snap = s.piano_roll.snap_beats;
                let selected = s.piano_roll.selected_note_ids.clone();
                if let Some(cid) = s.piano_roll.open_clip_id {
                    s.push_undo("Quantize");
                    if let Some(clip) = s.find_clip_mut(cid) {
                        for note in &mut clip.notes {
                            if selected.is_empty() || selected.contains(&note.id) {
                                note.beat = snap_beat(note.beat, snap);
                            }
                        }
                    }
                }
            }
            Message::TransposeUp => {
                let mut s = self.state.lock().unwrap();
                let selected = s.piano_roll.selected_note_ids.clone();
                if let Some(cid) = s.piano_roll.open_clip_id {
                    s.push_undo("Transpose");
                    if let Some(clip) = s.find_clip_mut(cid) {
                        for note in &mut clip.notes {
                            if selected.is_empty() || selected.contains(&note.id) {
                                note.pitch = (note.pitch as i16 + 1).clamp(0, 127) as u8;
                            }
                        }
                    }
                }
            }
            Message::TransposeDown => {
                let mut s = self.state.lock().unwrap();
                let selected = s.piano_roll.selected_note_ids.clone();
                if let Some(cid) = s.piano_roll.open_clip_id {
                    s.push_undo("Transpose");
                    if let Some(clip) = s.find_clip_mut(cid) {
                        for note in &mut clip.notes {
                            if selected.is_empty() || selected.contains(&note.id) {
                                note.pitch = (note.pitch as i16 - 1).clamp(0, 127) as u8;
                            }
                        }
                    }
                }
            }
            Message::TransposeOctaveUp => {
                let mut s = self.state.lock().unwrap();
                let selected = s.piano_roll.selected_note_ids.clone();
                if let Some(cid) = s.piano_roll.open_clip_id {
                    s.push_undo("Transpose Octave");
                    if let Some(clip) = s.find_clip_mut(cid) {
                        for note in &mut clip.notes {
                            if selected.is_empty() || selected.contains(&note.id) {
                                note.pitch = (note.pitch as i16 + 12).clamp(0, 127) as u8;
                            }
                        }
                    }
                }
            }
            Message::TransposeOctaveDown => {
                let mut s = self.state.lock().unwrap();
                let selected = s.piano_roll.selected_note_ids.clone();
                if let Some(cid) = s.piano_roll.open_clip_id {
                    s.push_undo("Transpose Octave");
                    if let Some(clip) = s.find_clip_mut(cid) {
                        for note in &mut clip.notes {
                            if selected.is_empty() || selected.contains(&note.id) {
                                note.pitch = (note.pitch as i16 - 12).clamp(0, 127) as u8;
                            }
                        }
                    }
                }
            }
            Message::SetScale(scale) => { self.pr_scale = scale; }
            Message::ClosePianoRoll => {
                let mut s = self.state.lock().unwrap();
                s.piano_roll.open_clip_id = None;
                s.active_view = ActiveView::Song;
            }

            // ── Piano Roll — keyboard ─────────────────────────────────────
            Message::PianoKeyPress(pitch) => {
                let mut s = self.state.lock().unwrap();
                if let Some(tid) = s.piano_roll.open_track_id {
                    let track_idx = s.tracks.iter().position(|t| t.id == tid).unwrap_or(0);
                    s.midi_queue.push(crate::app_state::PendingMidi {
                        track_idx, event: MidiEvent::NoteOn { pitch, velocity: 100 },
                    });
                    s.flush_midi();
                }
            }
            Message::PianoKeyRelease(pitch) => {
                let mut s = self.state.lock().unwrap();
                if let Some(tid) = s.piano_roll.open_track_id {
                    let track_idx = s.tracks.iter().position(|t| t.id == tid).unwrap_or(0);
                    s.midi_queue.push(crate::app_state::PendingMidi {
                        track_idx, event: MidiEvent::NoteOff { pitch },
                    });
                    s.flush_midi();
                }
            }

            // ── Velocity lane ─────────────────────────────────────────────
            Message::VelocityMouseDown { note_id, y } => {
                let s = self.state.lock().unwrap();
                let vel = s.piano_roll.open_clip_id
                    .and_then(|cid| s.find_clip(cid))
                    .and_then(|clip| clip.notes.iter().find(|n| n.id == note_id))
                    .map(|n| n.velocity).unwrap_or(100);
                drop(s);
                self.pr_velocity_drag_note = Some(note_id);
                self.pr_velocity_drag_start_y = y;
                self.pr_velocity_drag_start_vel = vel;
            }
            Message::VelocityMouseMove { y } => {
                if let Some(nid) = self.pr_velocity_drag_note {
                    let delta = (self.pr_velocity_drag_start_y - y) as i16;
                    let new_vel = (self.pr_velocity_drag_start_vel as i16 + delta).clamp(1, 127) as u8;
                    let mut s = self.state.lock().unwrap();
                    if let Some(cid) = s.piano_roll.open_clip_id {
                        if let Some(clip) = s.find_clip_mut(cid) {
                            if let Some(note) = clip.notes.iter_mut().find(|n| n.id == nid) {
                                note.velocity = new_vel;
                            }
                        }
                    }
                }
            }
            Message::VelocityMouseUp => {
                self.pr_velocity_drag_note = None;
            }

            // ── Mixer ─────────────────────────────────────────────────────
            Message::MixerFaderMove { track_idx, value } => {
                return self.update(Message::SetTrackVolume { track_idx, volume: value });
            }
            Message::MixerPanMove { track_idx, value } => {
                return self.update(Message::SetTrackPan { track_idx, pan: value });
            }

            // ── Instrument panel ──────────────────────────────────────────
            Message::OpenInstrumentPanel(id) => {
                self.state.lock().unwrap().instrument_panel_track = Some(id);
            }
            Message::CloseInstrumentPanel => {
                self.state.lock().unwrap().instrument_panel_track = None;
            }
            Message::SetInstrumentParam { track_idx, param } => {
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
                        _ => {}
                    }
                    let preset = s.tracks[track_idx].instrument.clone();
                    let sched_arc = s.scheduler.clone();
                    if let Ok(mut sched) = sched_arc.try_lock() {
                        if let Some(ref mut engine) = s.master_engine {
                            if let Some(slot) = engine.tracks.get_mut(track_idx) {
                                if let Some(ref mut te) = slot { te.update_preset(&mut sched, preset); }
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
                        id, effect_type, enabled: true,
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
                            EffectParam::ReverbRoom(v)      => fx.params.reverb_room = v,
                            EffectParam::ReverbDamp(v)      => fx.params.reverb_damp = v,
                            EffectParam::ReverbWet(v)       => fx.params.reverb_wet = v,
                            EffectParam::DelayTime(v)       => fx.params.delay_time = v,
                            EffectParam::DelayFeedback(v)   => fx.params.delay_feedback = v,
                            EffectParam::DelayWet(v)        => fx.params.delay_wet = v,
                            EffectParam::CompThreshold(v)   => fx.params.comp_threshold = v,
                            EffectParam::CompRatio(v)       => fx.params.comp_ratio = v,
                            EffectParam::FilterCutoff(v)    => fx.params.filter_cutoff = v,
                            EffectParam::FilterResonance(v) => fx.params.filter_resonance = v,
                            _ => {}
                        }
                    }
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
            Message::Cut => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                match view {
                    ActiveView::Song => {
                        s.copy_selected_clips();
                        s.delete_selected_clips();
                    }
                    ActiveView::PianoRoll => {
                        s.copy_selected_notes();
                        s.delete_selected_notes();
                    }
                    _ => {}
                }
            }
            Message::Duplicate => {
                let mut s = self.state.lock().unwrap();
                let view = s.active_view;
                if view == ActiveView::Song { s.duplicate_selected_clips(); }
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
        }
        Task::none()
    }

    // ── Helper: find track index at a y position ──────────────────────────────
    fn track_at_y(&self, tracks: &[crate::app_state::Track], y: f32) -> Option<usize> {
        let mut ty = 0.0f32;
        for (i, track) in tracks.iter().enumerate() {
            if y >= ty && y < ty + track.height { return Some(i); }
            ty += track.height;
        }
        None
    }

    pub fn view(&self) -> Element<Message> {
        let s = self.state.lock().unwrap();
        let active        = s.active_view;
        let transport     = s.transport.clone();
        let tracks        = s.tracks.clone();
        let engine_ok     = s.engine_status == crate::app_state::EngineStatus::Running;
        let song_st       = s.song_view.clone();
        let pr_st         = s.piano_roll.clone();
        let open_clip     = pr_st.open_clip_id.and_then(|id| s.find_clip(id)).cloned();
        let instr_track   = s.instrument_panel_track;
        let sel_track_id  = s.selected_track_id;
        let sel_track_idx = sel_track_id.and_then(|id| s.tracks.iter().position(|t| t.id == id));
        drop(s);

        let top_bar = self.build_top_bar(active, &transport, engine_ok);

        let workspace: Element<Message> = match active {
            ActiveView::Song => self.song_view_el(tracks.clone(), &transport, &song_st),
            ActiveView::PianoRoll => self.piano_roll_el(open_clip, &pr_st),
            ActiveView::Mixer => self.mixer_view_el(tracks.clone()),
            _ => placeholder_view(match active {
                ActiveView::Patcher => "Patcher — node graph (coming soon)",
                ActiveView::Perform => "Perform — live mixer (coming soon)",
                _ => "",
            }),
        };

        // Context menu overlay
        let ctx_menu: Option<Element<Message>> = self.context_menu.as_ref().map(|cm| {
            self.context_menu_el(cm, &tracks)
        });

        // Instrument panel
        let instr_panel: Option<Element<Message>> = instr_track.and_then(|tid| {
            let idx = tracks.iter().position(|t| t.id == tid)?;
            Some(instrument_panel_el(&tracks[idx], idx))
        });

        // Effects bar for selected track (Song view only)
        let fx_bar: Option<Element<Message>> = if active == ActiveView::Song {
            sel_track_idx.map(|idx| effects_bar_el(&tracks[idx], idx))
        } else {
            None
        };

        // Rename input overlay
        let rename_overlay: Option<Element<Message>> = self.renaming_track_id.map(|_| {
            container(
                row![
                    text_input("Track name...", &self.rename_value)
                        .on_input(Message::RenameInput)
                        .on_submit(Message::CommitRenameTrack)
                        .size(12)
                        .width(Length::Fixed(160.0)),
                    button(text("✓").size(11)).on_press(Message::CommitRenameTrack)
                        .style(|_,_| btn_style(true, false)),
                ].spacing(4).padding(6)
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.04, 0.08, 0.14))),
                border: iced::Border { color: Color::from_rgb(0.3, 0.72, 1.0), width: 1.0, radius: 4.0.into() },
                ..Default::default()
            })
            .into()
        });

        let mut main_col = column![top_bar, workspace];
        if let Some(fx) = fx_bar { main_col = main_col.push(fx); }
        if let Some(panel) = instr_panel { main_col = main_col.push(panel); }

        let base: Element<Message> = main_col.into();

        // Stack context menu on top if open
        if let Some(menu) = ctx_menu {
            // Use a simple column — proper overlay would need iced's layer system
            column![base, menu].into()
        } else {
            base
        }
    }
}

// ── Top bar ───────────────────────────────────────────────────────────────────

impl DawApp {
    fn build_top_bar(&self, active: ActiveView, transport: &crate::app_state::Transport, engine_ok: bool) -> Element<'static, Message> {
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
            r.push(button(text(lbl).size(12)).on_press(msg).style(move |_,_| tab_style(is_active)))
        });

        let is_playing   = transport.is_playing;
        let is_recording = transport.is_recording;
        let bpm          = transport.bpm;
        let beat         = transport.playhead_beat;
        let loop_on      = transport.loop_enabled;
        let bar          = (beat / transport.time_sig_num as f64) as u32 + 1;
        let beat_in_bar  = (beat % transport.time_sig_num as f64) as u32 + 1;
        let metronome_on = self.metronome_on;

        container(
            row![
                text("Aether").size(13).color(Color::from_rgb(0.3, 0.72, 1.0)),
                tabs,
                // Transport controls
                button(text("■").size(12)).on_press(Message::Stop).style(|_,_| btn_style(false, false)),
                button(text(if is_playing { "▶▶" } else { "▶" }).size(12))
                    .on_press(Message::Play).style(move |_,_| btn_style(is_playing, false)),
                button(text("●").size(12)).on_press(Message::ToggleRecord)
                    .style(move |_,_| btn_style(is_recording, true)),
                // BPM
                text(format!("{:.0}", bpm)).size(13).color(Color::WHITE),
                button(text("TAP").size(9)).on_press(Message::TapTempoClick)
                    .style(|_,_| btn_style(false, false)),
                // Position
                text(format!("{:3}:{}", bar, beat_in_bar)).size(11)
                    .color(Color::from_rgb(0.3, 0.72, 1.0)),
                // Loop + metronome
                button(text("⟳").size(11)).on_press(Message::ToggleLoop)
                    .style(move |_,_| tab_style(loop_on)),
                button(text("♩").size(11)).on_press(Message::ToggleMetronome)
                    .style(move |_,_| tab_style(metronome_on)),
                iced::widget::horizontal_space(),
                text(if engine_ok { "● Live" } else { "● Offline" }).size(10)
                    .color(if engine_ok { Color::from_rgb(0.0,0.9,0.63) } else { Color::from_rgb(0.94,0.33,0.31) }),
            ]
            .spacing(6).align_y(Alignment::Center).padding([4, 10]),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03, 0.06, 0.1))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }

    // ── Song view ─────────────────────────────────────────────────────────────

    fn song_view_el(&self, tracks: Vec<crate::app_state::Track>, transport: &crate::app_state::Transport, sv: &crate::app_state::SongViewState) -> Element<'static, Message> {
        let snap_opts: &[(&str, f64)] = &[("1 bar",1.0),("1/2",0.5),("1/4",0.25),("1/8",0.125),("1/16",0.0625)];
        let snap_row = snap_opts.iter().fold(row![].spacing(3), |r, (lbl, val)| {
            let active = (sv.snap_beats - val).abs() < 0.001;
            let v = *val;
            r.push(button(text(*lbl).size(9)).on_press(Message::SetSongSnap(v)).style(move |_,_| tab_style(active)))
        });

        let draw_a = sv.tool == SongTool::Draw;
        let sel_a  = sv.tool == SongTool::Select;
        let era_a  = sv.tool == SongTool::Erase;
        let tool_row = row![
            button(text("✏ Draw").size(9)).on_press(Message::SetSongTool(SongTool::Draw)).style(move |_,_| tab_style(draw_a)),
            button(text("↖ Sel").size(9)).on_press(Message::SetSongTool(SongTool::Select)).style(move |_,_| tab_style(sel_a)),
            button(text("✕ Era").size(9)).on_press(Message::SetSongTool(SongTool::Erase)).style(move |_,_| tab_style(era_a)),
        ].spacing(3);

        let zoom = self.song_zoom_x;
        let toolbar = container(
            row![
                button(text("+ Track").size(10)).on_press(Message::AddTrack).style(|_,_| btn_style(false, false)),
                tool_row, snap_row,
                text(format!("Zoom: {:.0}px/beat", zoom)).size(9).color(Color::from_rgb(0.28,0.4,0.52)),
            ].spacing(8).padding([4, 8]),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03,0.05,0.09))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        });

        // Track headers
        let renaming_id = self.renaming_track_id;
        let rename_val = self.rename_value.clone();
        let headers: Vec<Element<Message>> = tracks.iter().enumerate().map(|(i, t)| {
            let c     = color_from_u32(t.color);
            let muted = t.muted;
            let solo  = t.solo;
            let armed = t.armed;
            let tid   = t.id;
            let vol   = (t.volume * 100.0) as u32;

            let name_widget: Element<Message> = if renaming_id == Some(tid) {
                text_input("Name...", &rename_val)
                    .on_input(Message::RenameInput)
                    .on_submit(Message::CommitRenameTrack)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            } else {
                let name = t.name.clone();
                button(text(name).size(11).color(Color::WHITE))
                    .on_press(Message::StartRenameTrack(tid))
                    .style(|_,_| iced::widget::button::Style {
                        background: None, border: iced::Border::default(),
                        text_color: Color::WHITE, ..Default::default()
                    })
                    .into()
            };

            container(
                row![
                    container(iced::widget::vertical_space()).width(3).height(Length::Fill)
                        .style(move |_| container::Style { background: Some(iced::Background::Color(c)), ..Default::default() }),
                    column![
                        row![name_widget, iced::widget::horizontal_space()].spacing(4),
                        text(format!("{}%", vol)).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
                        row![
                            button(text("M").size(8)).on_press(Message::SetTrackMute { track_idx: i })
                                .style(move |_,_| btn_style(muted, true)),
                            button(text("S").size(8)).on_press(Message::SetTrackSolo { track_idx: i })
                                .style(move |_,_| btn_style(solo, false)),
                            button(text("●").size(8)).on_press(Message::ArmTrack { track_idx: i })
                                .style(move |_,_| btn_style(armed, true)),
                            button(text("🎹").size(8)).on_press(Message::OpenInstrumentPanel(tid))
                                .style(|_,_| btn_style(false, false)),
                        ].spacing(2),
                    ].spacing(2).padding([3, 6]),
                ].height(Length::Fixed(t.height)),
            )
            .width(Length::Fixed(180.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
                border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
                ..Default::default()
            })
            .into()
        }).collect();

        let header_col: Element<Message> = scrollable(column(headers).spacing(1))
            .height(Length::Fill).into();

        let playhead = transport.playhead_beat;
        let bpm = transport.bpm;
        let time_sig = transport.time_sig_num;
        let loop_start = transport.loop_start;
        let loop_end = transport.loop_end;
        let loop_on = transport.loop_enabled;
        let zoom_x = self.song_zoom_x;
        let scroll_x = self.song_scroll_x;
        let selected_ids = self.state.lock().unwrap().song_view.selected_clip_ids.clone();
        let rubber_band = if let SongDragMode::RubberBand { start_beat, start_track, end_beat, end_track } = &self.song_drag {
            Some((*start_beat, *start_track, *end_beat, *end_track))
        } else { None };

        let timeline: Element<Message> = canvas::Canvas::new(SongCanvas {
            tracks: tracks.clone(),
            playhead_beat: playhead,
            time_sig,
            zoom_x,
            scroll_x,
            loop_start,
            loop_end,
            loop_on,
            selected_ids,
            rubber_band,
        })
        .width(Length::Fill).height(Length::Fill).into();

        let main = row![header_col, timeline].height(Length::Fill);
        column![toolbar, main].height(Length::Fill).into()
    }

    // ── Piano Roll ────────────────────────────────────────────────────────────

    fn piano_roll_el(&self, clip: Option<Clip>, pr: &crate::app_state::PianoRollState) -> Element<'static, Message> {
        let draw_a = pr.tool == PianoRollTool::Draw;
        let sel_a  = pr.tool == PianoRollTool::Select;
        let era_a  = pr.tool == PianoRollTool::Erase;

        let snap_opts: &[(&str, f64)] = &[("1/4",0.25),("1/8",0.125),("1/16",0.0625),("1/32",0.03125)];
        let snap_row = snap_opts.iter().fold(row![].spacing(3), |r, (lbl, val)| {
            let active = (pr.snap_beats - val).abs() < 0.001;
            let v = *val;
            r.push(button(text(*lbl).size(9)).on_press(Message::SetPianoSnap(v)).style(move |_,_| tab_style(active)))
        });

        let scale_opts = [Scale::Chromatic, Scale::Major, Scale::Minor, Scale::Dorian,
                          Scale::Mixolydian, Scale::Pentatonic, Scale::Blues];
        let cur_scale = self.pr_scale;
        let scale_row = scale_opts.iter().fold(row![].spacing(3), |r, &sc| {
            let active = sc == cur_scale;
            r.push(button(text(sc.label()).size(9)).on_press(Message::SetScale(sc)).style(move |_,_| tab_style(active)))
        });

        let clip_name = clip.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| "No clip".to_string());

        let toolbar = container(
            column![
                row![
                    button(text("← Back").size(10)).on_press(Message::ClosePianoRoll).style(|_,_| btn_style(false, false)),
                    text(clip_name).size(11).color(Color::WHITE),
                    iced::widget::horizontal_space(),
                    button(text("✏").size(9)).on_press(Message::SetPianoTool(PianoRollTool::Draw)).style(move |_,_| tab_style(draw_a)),
                    button(text("↖").size(9)).on_press(Message::SetPianoTool(PianoRollTool::Select)).style(move |_,_| tab_style(sel_a)),
                    button(text("✕").size(9)).on_press(Message::SetPianoTool(PianoRollTool::Erase)).style(move |_,_| tab_style(era_a)),
                    snap_row,
                    button(text("Q").size(9)).on_press(Message::QuantizeNotes).style(|_,_| btn_style(false, false)),
                    button(text("↑").size(9)).on_press(Message::TransposeUp).style(|_,_| btn_style(false, false)),
                    button(text("↓").size(9)).on_press(Message::TransposeDown).style(|_,_| btn_style(false, false)),
                ].spacing(4).align_y(Alignment::Center),
                row![text("Scale:").size(9).color(Color::from_rgb(0.28,0.4,0.52)), scale_row].spacing(4),
            ].spacing(3).padding([4, 8]),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03,0.05,0.09))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        });

        // Piano keyboard
        let kh = self.pr_zoom_y;
        let keys: Vec<Element<Message>> = (0..88u8).rev().map(|k| {
            let midi = k + 21;
            let is_black = matches!(midi % 12, 1|3|6|8|10);
            let pitch = midi;
            let bg = if is_black { Color::from_rgb(0.1,0.1,0.12) } else { Color::from_rgb(0.82,0.85,0.88) };
            let note_name = if midi % 12 == 0 { format!("C{}", midi / 12 - 1) } else { String::new() };
            container(
                row![
                    button(iced::widget::horizontal_space())
                        .on_press(Message::PianoKeyPress(pitch))
                        .style(move |_,_| iced::widget::button::Style {
                            background: Some(iced::Background::Color(bg)),
                            border: iced::Border { color: Color::from_rgb(0.15,0.15,0.2), width: 0.5, radius: 0.0.into() },
                            ..Default::default()
                        })
                        .width(Length::Fixed(36.0)).height(Length::Fixed(kh)),
                    if !note_name.is_empty() {
                        let e: Element<Message> = text(note_name).size(7).color(Color::from_rgb(0.4,0.5,0.6)).into();
                        e
                    } else {
                        let e: Element<Message> = iced::widget::horizontal_space().into();
                        e
                    },
                ]
            ).into()
        }).collect();

        let key_col: Element<Message> = scrollable(column(keys).spacing(0)).height(Length::Fill).into();

        // Note grid
        let notes = clip.as_ref().map(|c| c.notes.clone()).unwrap_or_default();
        let selected_ids = pr.selected_note_ids.clone();
        let scale = self.pr_scale;
        let note_canvas: Element<Message> = canvas::Canvas::new(PianoRollCanvas {
            notes: notes.clone(),
            zoom_x: self.pr_zoom_x,
            zoom_y: self.pr_zoom_y,
            scroll_x: self.pr_scroll_x,
            scroll_y: self.pr_scroll_y,
            snap_beats: pr.snap_beats,
            selected_ids: selected_ids.clone(),
            scale,
        })
        .width(Length::Fill).height(Length::Fill).into();

        // Velocity lane
        let vel_canvas: Element<Message> = canvas::Canvas::new(VelocityCanvas {
            notes,
            zoom_x: self.pr_zoom_x,
            scroll_x: self.pr_scroll_x,
            selected_ids,
        })
        .width(Length::Fill).height(Length::Fixed(60.0)).into();

        let grid_area = column![
            row![key_col, note_canvas].height(Length::Fill),
            vel_canvas,
        ];

        column![toolbar, grid_area].height(Length::Fill).into()
    }

    // ── Mixer ─────────────────────────────────────────────────────────────────

    fn mixer_view_el(&self, tracks: Vec<crate::app_state::Track>) -> Element<'static, Message> {
        let strips: Vec<Element<Message>> = tracks.iter().enumerate().map(|(i, t)| {
            let c     = color_from_u32(t.color);
            let name  = t.name.clone();
            let vol   = t.volume;
            let pan   = t.pan;
            let muted = t.muted;
            let solo  = t.solo;
            let vu    = self.vu_levels.get(i).copied().unwrap_or(0.0);
            let vu_h  = (vu * 80.0).clamp(0.0, 80.0);

            container(
                column![
                    // Color bar
                    container(iced::widget::horizontal_space()).width(Length::Fill).height(Length::Fixed(3.0))
                        .style(move |_| container::Style { background: Some(iced::Background::Color(c)), ..Default::default() }),
                    // Name
                    text(name).size(9).color(c),
                    // VU meter
                    container(
                        container(iced::widget::vertical_space())
                            .width(Length::Fixed(8.0)).height(Length::Fixed(vu_h))
                            .style(move |_| container::Style {
                                background: Some(iced::Background::Color(
                                    if vu > 0.9 { Color::from_rgb(0.94,0.33,0.31) }
                                    else if vu > 0.7 { Color::from_rgb(0.98,0.75,0.14) }
                                    else { Color::from_rgb(0.0,0.9,0.63) }
                                )),
                                ..Default::default()
                            })
                    )
                    .width(Length::Fixed(12.0)).height(Length::Fixed(80.0))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.02,0.04,0.07))),
                        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 2.0.into() },
                        ..Default::default()
                    }),
                    // Fader
                    slider(0.0..=1.0_f32, vol, move |v| Message::MixerFaderMove { track_idx: i, value: v })
                        .step(0.001)
                        .width(Length::Fixed(56.0)),
                    text(format!("{:.0}%", vol * 100.0)).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
                    // Pan
                    slider(-1.0..=1.0_f32, pan, move |v| Message::MixerPanMove { track_idx: i, value: v })
                        .step(0.01)
                        .width(Length::Fixed(56.0)),
                    text(if pan == 0.0 { "C".to_string() } else if pan > 0.0 { format!("R{:.0}", pan*100.0) } else { format!("L{:.0}", -pan*100.0) })
                        .size(8).color(Color::from_rgb(0.28,0.4,0.52)),
                    // M/S buttons
                    row![
                        button(text("M").size(8)).on_press(Message::SetTrackMute { track_idx: i }).style(move |_,_| btn_style(muted, true)),
                        button(text("S").size(8)).on_press(Message::SetTrackSolo { track_idx: i }).style(move |_,_| btn_style(solo, false)),
                    ].spacing(2),
                ]
                .spacing(3).align_x(Alignment::Center).padding(5),
            )
            .width(Length::Fixed(72.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
                border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 5.0.into() },
                ..Default::default()
            })
            .into()
        }).collect();

        container(
            scrollable(row(strips).spacing(6).padding(12))
                .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()))
        )
        .width(Length::Fill).height(Length::Fill)
        .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.02,0.04,0.07))), ..Default::default() })
        .into()
    }

    // ── Context menu ──────────────────────────────────────────────────────────

    fn context_menu_el(&self, cm: &ContextMenu, tracks: &[crate::app_state::Track]) -> Element<'static, Message> {
        let items: Vec<Element<Message>> = match &cm.kind {
            ContextMenuKind::Clip(cid) => {
                let cid = *cid;
                let track_id = tracks.iter().flat_map(|t| t.clips.iter())
                    .find(|c| c.id == cid).map(|c| c.track_id).unwrap_or(0);
                vec![
                    ctx_item("Open Piano Roll", Message::ContextAction(ContextAction::OpenPianoRoll { clip_id: cid, track_id })),
                    ctx_item("Duplicate", Message::ContextAction(ContextAction::DuplicateClip(cid))),
                    ctx_item("Delete", Message::ContextAction(ContextAction::DeleteClip(cid))),
                    ctx_item("Rename", Message::ContextAction(ContextAction::RenameClip(cid))),
                ]
            }
            ContextMenuKind::Track(tid) => {
                let tid = *tid;
                vec![
                    ctx_item("Rename Track", Message::ContextAction(ContextAction::RenameTrack(tid))),
                    ctx_item("Move Up", Message::ContextAction(ContextAction::MoveTrackUp(tid))),
                    ctx_item("Move Down", Message::ContextAction(ContextAction::MoveTrackDown(tid))),
                    ctx_item("Delete Track", Message::ContextAction(ContextAction::DeleteTrack(tid))),
                ]
            }
            ContextMenuKind::Timeline { beat, track_idx } => {
                let b = *beat;
                let _ti = *track_idx;
                vec![
                    ctx_item("+ Instrument Track", Message::ContextAction(ContextAction::AddInstrumentTrack)),
                    ctx_item("+ Audio Track", Message::ContextAction(ContextAction::AddAudioTrack)),
                ]
            }
        };

        container(
            column(items).spacing(1).padding(4)
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.06,0.1,0.16))),
            border: iced::Border { color: Color::from_rgb(0.15,0.28,0.45), width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    }
}

fn ctx_item(label: &'static str, msg: Message) -> Element<'static, Message> {
    button(text(label).size(11).color(Color::from_rgb(0.8,0.88,0.95)))
        .on_press(msg)
        .style(|_, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => Color::from_rgba(0.3,0.72,1.0,0.1),
                _ => Color::TRANSPARENT,
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border::default(),
                text_color: Color::from_rgb(0.8,0.88,0.95),
                ..Default::default()
            }
        })
        .width(Length::Fixed(160.0))
        .into()
}

// ── Song canvas ───────────────────────────────────────────────────────────────

struct SongCanvas {
    tracks: Vec<crate::app_state::Track>,
    playhead_beat: f64,
    time_sig: u8,
    zoom_x: f32,
    scroll_x: f32,
    loop_start: f64,
    loop_end: f64,
    loop_on: bool,
    selected_ids: Vec<u64>,
    rubber_band: Option<(f64, usize, f64, usize)>,
}

#[derive(Default, Clone)]
struct SongCanvasState {
    last_x: f32,
    last_y: f32,
    pressed: bool,
    right_pressed: bool,
}

impl canvas::Program<Message> for SongCanvas {
    type State = SongCanvasState;

    fn update(&self, state: &mut SongCanvasState, event: canvas::Event, bounds: iced::Rectangle, cursor: iced::mouse::Cursor) -> (canvas::event::Status, Option<Message>) {
        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => return (canvas::event::Status::Ignored, None),
        };

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(btn)) => {
                let right = btn == iced::mouse::Button::Right;
                state.pressed = !right;
                state.right_pressed = right;
                state.last_x = pos.x;
                state.last_y = pos.y;
                (canvas::event::Status::Captured, Some(Message::SongMouseDown { x: pos.x, y: pos.y, right }))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if state.pressed || state.right_pressed => {
                state.last_x = pos.x;
                state.last_y = pos.y;
                (canvas::event::Status::Captured, Some(Message::SongMouseMove { x: pos.x, y: pos.y }))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
                state.pressed = false;
                state.right_pressed = false;
                (canvas::event::Status::Captured, Some(Message::SongMouseUp { x: pos.x, y: pos.y }))
            }
            canvas::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let (dx, dy, ctrl) = match delta {
                    iced::mouse::ScrollDelta::Lines { x, y } => (x, y, false),
                    iced::mouse::ScrollDelta::Pixels { x, y } => (x / 32.0, y / 32.0, false),
                };
                (canvas::event::Status::Captured, Some(Message::SongScroll { delta_x: dx, delta_y: dy, ctrl }))
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &SongCanvasState, renderer: &iced::Renderer, _: &Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let bw = self.zoom_x;
        let ruler_h = 28.0f32;
        let scroll = self.scroll_x;

        // Background
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.04,0.07,0.12));
        // Ruler bg
        frame.fill_rectangle(iced::Point::ORIGIN, iced::Size::new(bounds.width, ruler_h), Color::from_rgb(0.03,0.05,0.09));

        // Loop region
        if self.loop_on {
            let lx = self.loop_start as f32 * bw - scroll;
            let lw = (self.loop_end - self.loop_start) as f32 * bw;
            frame.fill_rectangle(iced::Point::new(lx, 0.0), iced::Size::new(lw, bounds.height),
                Color::from_rgba(0.98,0.75,0.14,0.06));
            frame.fill_rectangle(iced::Point::new(lx, 0.0), iced::Size::new(2.0, bounds.height),
                Color::from_rgba(0.98,0.75,0.14,0.5));
            frame.fill_rectangle(iced::Point::new(lx + lw - 2.0, 0.0), iced::Size::new(2.0, bounds.height),
                Color::from_rgba(0.98,0.75,0.14,0.5));
        }

        // Beat/bar lines
        let first_beat = (scroll / bw) as usize;
        let last_beat = ((scroll + bounds.width) / bw) as usize + 2;
        for i in first_beat..last_beat {
            let is_bar = i % self.time_sig as usize == 0;
            let x = i as f32 * bw - scroll;
            let lh = if is_bar { ruler_h } else { ruler_h * 0.4 };
            frame.fill_rectangle(iced::Point::new(x, ruler_h - lh), iced::Size::new(1.0, lh),
                if is_bar { Color::from_rgb(0.15,0.28,0.45) } else { Color::from_rgb(0.06,0.1,0.16) });
            if is_bar {
                let bar_num = i / self.time_sig as usize + 1;
                frame.fill_text(canvas::Text {
                    content: bar_num.to_string(),
                    position: iced::Point::new(x + 3.0, 4.0),
                    color: Color::from_rgb(0.3,0.72,1.0),
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

            // Track bg
            frame.fill_rectangle(iced::Point::new(0.0, ty), iced::Size::new(bounds.width, th),
                Color::from_rgb(0.04,0.07,0.12));

            // Grid lines
            for i in first_beat..last_beat {
                let is_bar = i % self.time_sig as usize == 0;
                let x = i as f32 * bw - scroll;
                frame.fill_rectangle(iced::Point::new(x, ty), iced::Size::new(1.0, th),
                    if is_bar { Color::from_rgb(0.07,0.13,0.2) } else { Color::from_rgb(0.04,0.07,0.12) });
            }

            // Clips
            for clip in &track.clips {
                let cx = clip.start_beat as f32 * bw - scroll + 1.0;
                let cw = (clip.length_beats as f32 * bw - 2.0).max(4.0);
                let selected = self.selected_ids.contains(&clip.id);

                // Clip fill
                frame.fill_rectangle(iced::Point::new(cx, ty + 3.0), iced::Size::new(cw, th - 6.0),
                    Color { a: if selected { 0.3 } else { 0.15 }, ..tc });

                // Clip border
                let border_color = if selected { Color { a: 1.0, ..tc } } else { Color { a: 0.6, ..tc } };
                let stroke = canvas::Stroke::default().with_color(border_color).with_width(if selected { 1.5 } else { 1.0 });
                frame.stroke(&canvas::Path::rectangle(iced::Point::new(cx, ty + 3.0), iced::Size::new(cw, th - 6.0)), stroke);

                // Clip name
                frame.fill_text(canvas::Text {
                    content: clip.name.clone(),
                    position: iced::Point::new(cx + 4.0, ty + 6.0),
                    color: Color { a: 0.9, ..tc },
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });

                // Mini note preview
                if !clip.notes.is_empty() {
                    let note_h = th - 18.0;
                    for note in &clip.notes {
                        let nx = cx + note.beat as f32 * bw;
                        let nw = (note.duration as f32 * bw).max(2.0);
                        let ny = ty + 10.0 + (1.0 - note.pitch as f32 / 127.0) * note_h;
                        frame.fill_rectangle(iced::Point::new(nx, ny), iced::Size::new(nw, 1.5),
                            Color { a: 0.85, ..tc });
                    }
                }

                // Resize handle (right edge indicator)
                let end_x = cx + cw;
                frame.fill_rectangle(iced::Point::new(end_x - 4.0, ty + 3.0), iced::Size::new(4.0, th - 6.0),
                    Color { a: 0.4, ..tc });
            }

            // Track separator
            frame.fill_rectangle(iced::Point::new(0.0, ty + th - 1.0), iced::Size::new(bounds.width, 1.0),
                Color::from_rgb(0.06,0.1,0.16));
            ty += th;
        }

        // Rubber band selection
        if let Some((sb, st, eb, et)) = self.rubber_band {
            let x1 = sb as f32 * bw - scroll;
            let x2 = eb as f32 * bw - scroll;
            let mut ty2 = ruler_h;
            let mut y1 = ruler_h;
            let mut y2 = ruler_h;
            for (i, track) in self.tracks.iter().enumerate() {
                if i == st.min(et) { y1 = ty2; }
                if i == st.max(et) { y2 = ty2 + track.height; }
                ty2 += track.height;
            }
            let rx = x1.min(x2);
            let rw = (x1 - x2).abs();
            let ry = y1;
            let rh = y2 - y1;
            frame.fill_rectangle(iced::Point::new(rx, ry), iced::Size::new(rw, rh),
                Color::from_rgba(0.3,0.72,1.0,0.08));
            let stroke = canvas::Stroke::default().with_color(Color::from_rgba(0.3,0.72,1.0,0.5)).with_width(1.0);
            frame.stroke(&canvas::Path::rectangle(iced::Point::new(rx, ry), iced::Size::new(rw, rh)), stroke);
        }

        // Playhead
        let phx = self.playhead_beat as f32 * bw - scroll;
        frame.fill_rectangle(iced::Point::new(phx, 0.0), iced::Size::new(2.0, bounds.height),
            Color::from_rgba(0.3,0.72,1.0,0.9));
        let tri = canvas::Path::new(|p| {
            p.move_to(iced::Point::new(phx - 5.0, 0.0));
            p.line_to(iced::Point::new(phx + 5.0, 0.0));
            p.line_to(iced::Point::new(phx, 10.0));
            p.close();
        });
        frame.fill(&tri, Color::from_rgb(0.3,0.72,1.0));

        vec![frame.into_geometry()]
    }
}

// ── Piano Roll canvas ─────────────────────────────────────────────────────────

struct PianoRollCanvas {
    notes: Vec<MidiNote>,
    zoom_x: f32,
    zoom_y: f32,
    scroll_x: f32,
    scroll_y: f32,
    snap_beats: f64,
    selected_ids: Vec<u64>,
    scale: Scale,
}

#[derive(Default, Clone)]
struct PrCanvasState { pressed: bool, right: bool }

impl canvas::Program<Message> for PianoRollCanvas {
    type State = PrCanvasState;

    fn update(&self, state: &mut PrCanvasState, event: canvas::Event, bounds: iced::Rectangle, cursor: iced::mouse::Cursor) -> (canvas::event::Status, Option<Message>) {
        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => return (canvas::event::Status::Ignored, None),
        };
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(btn)) => {
                let right = btn == iced::mouse::Button::Right;
                state.pressed = !right; state.right = right;
                (canvas::event::Status::Captured, Some(Message::PianoRollMouseDown { x: pos.x, y: pos.y, right }))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if state.pressed || state.right => {
                (canvas::event::Status::Captured, Some(Message::PianoRollMouseMove { x: pos.x, y: pos.y }))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
                state.pressed = false; state.right = false;
                (canvas::event::Status::Captured, Some(Message::PianoRollMouseUp { x: pos.x, y: pos.y }))
            }
            canvas::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let (dx, dy, ctrl) = match delta {
                    iced::mouse::ScrollDelta::Lines { x, y } => (x, y, false),
                    iced::mouse::ScrollDelta::Pixels { x, y } => (x / 32.0, y / 32.0, false),
                };
                (canvas::event::Status::Captured, Some(Message::PianoRollScroll { delta_x: dx, delta_y: dy, ctrl }))
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &PrCanvasState, renderer: &iced::Renderer, _: &Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let bw = self.zoom_x;
        let kh = self.zoom_y;
        let total_keys = 128usize;
        let scale_semitones = self.scale.semitones();

        // Background
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.04,0.07,0.12));

        // Key rows
        let first_key = (self.scroll_y / kh) as usize;
        let last_key = ((self.scroll_y + bounds.height) / kh) as usize + 2;
        for k in first_key..last_key.min(total_keys) {
            let midi = total_keys - 1 - k;
            let is_black = matches!(midi % 12, 1|3|6|8|10);
            let in_scale = scale_semitones.contains(&((midi % 12) as u8));
            let y = k as f32 * kh - self.scroll_y;
            let bg = if is_black {
                Color::from_rgb(0.03,0.06,0.1)
            } else if !in_scale && self.scale != Scale::Chromatic {
                Color::from_rgb(0.035,0.065,0.105) // slightly dimmed out-of-scale
            } else {
                Color::from_rgb(0.045,0.085,0.135)
            };
            frame.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(bounds.width, kh), bg);
            frame.fill_rectangle(iced::Point::new(0.0, y + kh - 0.5), iced::Size::new(bounds.width, 0.5),
                Color::from_rgb(0.06,0.1,0.16));
            // C note highlight
            if midi % 12 == 0 {
                frame.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(bounds.width, 1.0),
                    Color::from_rgba(0.3,0.72,1.0,0.15));
            }
        }

        // Beat lines
        let first_beat = (self.scroll_x / bw) as usize;
        let last_beat = ((self.scroll_x + bounds.width) / bw) as usize + 2;
        for i in first_beat..last_beat {
            let x = i as f32 * bw - self.scroll_x;
            let is_bar = i % 4 == 0;
            frame.fill_rectangle(iced::Point::new(x, 0.0), iced::Size::new(1.0, bounds.height),
                if is_bar { Color::from_rgb(0.1,0.2,0.32) } else { Color::from_rgb(0.06,0.1,0.16) });
        }

        // Notes
        for note in &self.notes {
            let x = note.beat as f32 * bw - self.scroll_x;
            let y = (total_keys - 1 - note.pitch as usize) as f32 * kh - self.scroll_y;
            let nw = (note.duration as f32 * bw).max(3.0);
            let selected = self.selected_ids.contains(&note.id);

            // Color by velocity
            let vel_t = note.velocity as f32 / 127.0;
            let note_color = if selected {
                Color::from_rgb(0.0, 0.9, 0.63)
            } else {
                // Gradient: low vel = dark blue, high vel = bright cyan
                Color::from_rgb(
                    0.1 + vel_t * 0.2,
                    0.4 + vel_t * 0.5,
                    0.8 + vel_t * 0.2,
                )
            };

            frame.fill_rectangle(iced::Point::new(x + 1.0, y + 1.0), iced::Size::new(nw - 2.0, kh - 2.0), note_color);

            // Note border
            let stroke = canvas::Stroke::default()
                .with_color(Color::from_rgba(1.0,1.0,1.0,0.25)).with_width(0.5);
            frame.stroke(&canvas::Path::rectangle(iced::Point::new(x + 1.0, y + 1.0), iced::Size::new(nw - 2.0, kh - 2.0)), stroke);

            // Resize handle
            frame.fill_rectangle(iced::Point::new(x + nw - 4.0, y + 1.0), iced::Size::new(3.0, kh - 2.0),
                Color::from_rgba(1.0,1.0,1.0,0.3));
        }

        vec![frame.into_geometry()]
    }
}

// ── Velocity canvas ───────────────────────────────────────────────────────────

struct VelocityCanvas {
    notes: Vec<MidiNote>,
    zoom_x: f32,
    scroll_x: f32,
    selected_ids: Vec<u64>,
}

#[derive(Default, Clone)]
struct VelCanvasState { pressed: bool, note_id: Option<u64> }

impl canvas::Program<Message> for VelocityCanvas {
    type State = VelCanvasState;

    fn update(&self, state: &mut VelCanvasState, event: canvas::Event, bounds: iced::Rectangle, cursor: iced::mouse::Cursor) -> (canvas::event::Status, Option<Message>) {
        let pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => return (canvas::event::Status::Ignored, None),
        };
        let bw = self.zoom_x;
        let beat = ((pos.x + self.scroll_x) / bw) as f64;

        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                // Find note under cursor
                let note_id = self.notes.iter()
                    .find(|n| beat >= n.beat && beat < n.beat + n.duration)
                    .map(|n| n.id);
                if let Some(nid) = note_id {
                    state.pressed = true;
                    state.note_id = Some(nid);
                    (canvas::event::Status::Captured, Some(Message::VelocityMouseDown { note_id: nid, y: pos.y }))
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) if state.pressed => {
                (canvas::event::Status::Captured, Some(Message::VelocityMouseMove { y: pos.y }))
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
                state.pressed = false;
                (canvas::event::Status::Captured, Some(Message::VelocityMouseUp))
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(&self, _: &VelCanvasState, renderer: &iced::Renderer, _: &Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let bw = self.zoom_x;

        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.03,0.05,0.09));
        // Separator line
        frame.fill_rectangle(iced::Point::ORIGIN, iced::Size::new(bounds.width, 1.0), Color::from_rgb(0.1,0.2,0.32));

        for note in &self.notes {
            let x = note.beat as f32 * bw - self.scroll_x;
            let bar_h = (note.velocity as f32 / 127.0) * (bounds.height - 4.0);
            let selected = self.selected_ids.contains(&note.id);
            let col = if selected { Color::from_rgb(0.0,0.9,0.63) } else { Color::from_rgb(0.3,0.72,1.0) };

            frame.fill_rectangle(
                iced::Point::new(x + 1.0, bounds.height - bar_h - 2.0),
                iced::Size::new((bw * note.duration as f32 - 2.0).max(2.0), bar_h),
                col,
            );
        }

        vec![frame.into_geometry()]
    }
}

// ── Instrument panel ──────────────────────────────────────────────────────────

fn instrument_panel_el(track: &crate::app_state::Track, track_idx: usize) -> Element<'static, Message> {
    let p = track.instrument.clone();
    let c = color_from_u32(track.color);
    let name = track.name.clone();

    let waveforms = [("Sine",0.0),("Saw",1.0),("Sqr",2.0),("Tri",3.0)];
    let wave_row = waveforms.iter().fold(row![].spacing(3), |r, (lbl, val)| {
        let active = (p.waveform - val).abs() < 0.1;
        let v = *val;
        r.push(button(text(*lbl).size(9)).on_press(Message::SetInstrumentParam {
            track_idx, param: InstrumentParam::Waveform(v),
        }).style(move |_,_| tab_style(active)))
    });

    let knobs = row![
        synth_knob("ATK",  p.attack,   0.001, 4.0,  track_idx, |v| InstrumentParam::Attack(v)),
        synth_knob("DEC",  p.decay,    0.001, 4.0,  track_idx, |v| InstrumentParam::Decay(v)),
        synth_knob("SUS",  p.sustain,  0.0,   1.0,  track_idx, |v| InstrumentParam::Sustain(v)),
        synth_knob("REL",  p.release,  0.001, 4.0,  track_idx, |v| InstrumentParam::Release(v)),
        synth_knob("CUT",  p.cutoff / 20000.0, 0.0, 1.0, track_idx, |v| InstrumentParam::Cutoff(v * 20000.0)),
        synth_knob("RES",  p.resonance / 4.0, 0.0, 1.0, track_idx, |v| InstrumentParam::Resonance(v * 4.0)),
        synth_knob("GAIN", p.gain,     0.0,   1.0,  track_idx, |v| InstrumentParam::Gain(v)),
    ].spacing(10);

    container(
        row![
            container(iced::widget::vertical_space()).width(3).height(Length::Fill)
                .style(move |_| container::Style { background: Some(iced::Background::Color(c)), ..Default::default() }),
            column![
                row![
                    text(name).size(11).color(c),
                    iced::widget::horizontal_space(),
                    button(text("✕").size(10)).on_press(Message::CloseInstrumentPanel)
                        .style(|_,_| btn_style(false, false)),
                ].spacing(8).align_y(Alignment::Center),
                wave_row,
                knobs,
            ].spacing(5).padding([4, 10]),
        ],
    )
    .width(Length::Fill).height(Length::Fixed(115.0))
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03,0.05,0.09))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into()
}

fn synth_knob<F>(label: &'static str, value: f32, min: f32, max: f32, track_idx: usize, make_param: F) -> Element<'static, Message>
where F: Fn(f32) -> InstrumentParam + 'static {
    let norm = ((value - min) / (max - min)).clamp(0.0, 1.0);
    column![
        text(label).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
        slider(0.0..=1.0_f32, norm, move |v| {
            Message::SetInstrumentParam { track_idx, param: make_param(min + v * (max - min)) }
        }).step(0.001).width(Length::Fixed(40.0)),
        text(format!("{:.2}", value)).size(7).color(Color::from_rgb(0.28,0.4,0.52)),
    ].spacing(2).align_x(Alignment::Center).into()
}

// ── Effects bar ───────────────────────────────────────────────────────────────

fn effects_bar_el(track: &crate::app_state::Track, track_idx: usize) -> Element<'static, Message> {
    let effect_defs = [
        (EffectType::Eq,         "EQ",     Color::from_rgb(0.3,0.72,1.0)),
        (EffectType::Compressor, "Comp",   Color::from_rgb(0.65,0.55,0.98)),
        (EffectType::Reverb,     "Reverb", Color::from_rgb(0.2,0.83,0.6)),
        (EffectType::Delay,      "Delay",  Color::from_rgb(0.98,0.75,0.14)),
        (EffectType::Filter,     "Filter", Color::from_rgb(0.98,0.45,0.09)),
    ];

    let add_btns = effect_defs.iter().fold(row![].spacing(3), |r, (et, lbl, col)| {
        let et2 = et.clone(); let c = *col;
        r.push(button(text(format!("+ {}", lbl)).size(9))
            .on_press(Message::AddEffect { track_idx, effect_type: et2 })
            .style(move |_,_| iced::widget::button::Style {
                background: Some(iced::Background::Color(Color { a: 0.08, ..c })),
                border: iced::Border { color: Color { a: 0.25, ..c }, width: 1.0, radius: 3.0.into() },
                text_color: c, ..Default::default()
            }))
    });

    let fx_chips: Vec<Element<Message>> = track.effects.iter().map(|fx| {
        let lbl = fx.effect_type.label().to_string();
        let enabled = fx.enabled;
        let fid = fx.id;
        let col = match fx.effect_type {
            EffectType::Eq         => Color::from_rgb(0.3,0.72,1.0),
            EffectType::Compressor => Color::from_rgb(0.65,0.55,0.98),
            EffectType::Reverb     => Color::from_rgb(0.2,0.83,0.6),
            EffectType::Delay      => Color::from_rgb(0.98,0.75,0.14),
            EffectType::Filter     => Color::from_rgb(0.98,0.45,0.09),
        };
        let a = if enabled { 0.7 } else { 0.25 };
        row![
            button(text(lbl).size(9))
                .on_press(Message::ToggleEffect { track_idx, effect_id: fid })
                .style(move |_,_| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color { a: if enabled { 0.15 } else { 0.05 }, ..col })),
                    border: iced::Border { color: Color { a, ..col }, width: 1.0, radius: 3.0.into() },
                    text_color: Color { a, ..col }, ..Default::default()
                }),
            button(text("✕").size(8))
                .on_press(Message::RemoveEffect { track_idx, effect_id: fid })
                .style(|_,_| btn_style(false, false)),
        ].spacing(1).into()
    }).collect();

    let tid = track.id;
    let tc = color_from_u32(track.color);
    let tname = track.name.clone();

    container(
        row![
            text(format!("FX: {}", tname)).size(9).color(tc),
            button(text("🎹 Synth").size(9)).on_press(Message::OpenInstrumentPanel(tid))
                .style(|_,_| btn_style(false, false)),
            iced::widget::vertical_rule(1),
            add_btns,
            iced::widget::vertical_rule(1),
        ]
        .push(row(fx_chips).spacing(4))
        .spacing(8).align_y(Alignment::Center).padding([3, 10]),
    )
    .width(Length::Fill).height(Length::Fixed(34.0))
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03,0.05,0.09))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into()
}

fn placeholder_view(msg: &str) -> Element<'static, Message> {
    let msg = msg.to_string();
    container(text(msg).size(14).color(Color::from_rgb(0.28,0.4,0.52)))
        .width(Length::Fill).height(Length::Fill)
        .center_x(Length::Fill).center_y(Length::Fill)
        .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.02,0.04,0.07))), ..Default::default() })
        .into()
}

// ── Style helpers ─────────────────────────────────────────────────────────────

fn tab_style(active: bool) -> iced::widget::button::Style {
    if active {
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.3,0.72,1.0,0.12))),
            border: iced::Border { color: Color::from_rgba(0.3,0.72,1.0,0.3), width: 1.0, radius: 4.0.into() },
            text_color: Color::from_rgb(0.3,0.72,1.0),
            ..Default::default()
        }
    } else {
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
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
        border: iced::Border { color: border, width: 1.0, radius: 4.0.into() },
        text_color: tc, ..Default::default()
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

fn snap_beat(beat: f64, snap: f64) -> f64 {
    if snap <= 0.0 { beat } else { (beat / snap).floor() * snap }
}
