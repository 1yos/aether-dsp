//! AppState — shared state between the audio engine and the UI.

use std::sync::{Arc, Mutex};
use std::time::Instant;
use aether_core::scheduler::Scheduler;
use crate::instrument::{MasterEngine, MidiEvent, InstrumentPreset};

// ── Transport ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Transport {
    pub bpm: f32,
    pub is_playing: bool,
    pub is_recording: bool,
    pub playhead_beat: f64,
    pub loop_start: f64,
    pub loop_end: f64,
    pub loop_enabled: bool,
    pub time_sig_num: u8,
    pub time_sig_den: u8,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            is_playing: false,
            is_recording: false,
            playhead_beat: 0.0,
            loop_start: 0.0,
            loop_end: 16.0,
            loop_enabled: false,
            time_sig_num: 4,
            time_sig_den: 4,
        }
    }
}

// ── Track / Clip / Note ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub id: u64,
    pub pitch: u8,
    pub beat: f64,
    pub duration: f64,
    pub velocity: u8,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub id: u64,
    pub track_id: u64,
    pub name: String,
    pub start_beat: f64,
    pub length_beats: f64,
    pub color: u32,
    pub notes: Vec<MidiNote>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackType {
    Instrument,
    Audio,
    Bus,
    Master,
}

#[derive(Debug, Clone)]
pub struct TrackEffect {
    pub id: u64,
    pub effect_type: EffectType,
    pub enabled: bool,
    // Effect params
    pub params: EffectParams,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    Eq,
    Compressor,
    Reverb,
    Delay,
    Filter,
}

impl EffectType {
    pub fn label(&self) -> &'static str {
        match self {
            EffectType::Eq         => "EQ",
            EffectType::Compressor => "Comp",
            EffectType::Reverb     => "Reverb",
            EffectType::Delay      => "Delay",
            EffectType::Filter     => "Filter",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectParams {
    // EQ
    pub eq_low_gain: f32,   // dB -12..+12
    pub eq_mid_gain: f32,
    pub eq_high_gain: f32,
    pub eq_mid_freq: f32,   // Hz
    // Compressor
    pub comp_threshold: f32, // dB
    pub comp_ratio: f32,
    pub comp_attack: f32,    // ms
    pub comp_release: f32,   // ms
    pub comp_makeup: f32,    // dB
    // Reverb
    pub reverb_room: f32,
    pub reverb_damp: f32,
    pub reverb_wet: f32,
    // Delay
    pub delay_time: f32,     // seconds
    pub delay_feedback: f32,
    pub delay_wet: f32,
    // Filter
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub filter_mode: u8,     // 0=LP 1=HP 2=BP
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            eq_low_gain: 0.0, eq_mid_gain: 0.0, eq_high_gain: 0.0, eq_mid_freq: 1000.0,
            comp_threshold: -20.0, comp_ratio: 4.0, comp_attack: 5.0, comp_release: 100.0, comp_makeup: 0.0,
            reverb_room: 0.5, reverb_damp: 0.5, reverb_wet: 0.3,
            delay_time: 0.25, delay_feedback: 0.4, delay_wet: 0.3,
            filter_cutoff: 2000.0, filter_resonance: 1.0, filter_mode: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub track_type: TrackType,
    pub color: u32,
    pub muted: bool,
    pub solo: bool,
    pub armed: bool,
    pub volume: f32,
    pub pan: f32,
    pub height: f32,
    pub clips: Vec<Clip>,
    pub effects: Vec<TrackEffect>,
    // Instrument preset (for instrument tracks)
    pub instrument: InstrumentPreset,
}

// ── Mixer ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MixerChannel {
    pub id: u64,
    pub name: String,
    pub color: u32,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
}

// ── Engine status ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    Starting,
    Running,
    Error(String),
}

// ── Active view ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ActiveView {
    Song,
    PianoRoll,
    Mixer,
    Patcher,
    Perform,
}

// ── Pending MIDI events (UI → engine) ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PendingMidi {
    pub track_idx: usize,
    pub event: MidiEvent,
}

// ── Piano Roll edit state ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PianoRollTool {
    Draw,
    Select,
    Erase,
}

#[derive(Debug, Clone)]
pub struct PianoRollState {
    pub open_clip_id: Option<u64>,
    pub open_track_id: Option<u64>,
    pub tool: PianoRollTool,
    pub zoom_x: f32,   // beats per pixel
    pub zoom_y: f32,   // pixels per semitone
    pub scroll_x: f32, // beat offset
    pub scroll_y: f32, // semitone offset (0=C8, 127=C-1)
    pub snap_beats: f64, // quantize grid (0.25 = 1/16)
    pub default_note_len: f64,
    pub selected_note_ids: Vec<u64>,
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self {
            open_clip_id: None,
            open_track_id: None,
            tool: PianoRollTool::Draw,
            zoom_x: 80.0,
            zoom_y: 14.0,
            scroll_x: 0.0,
            scroll_y: 48.0, // start around C3
            snap_beats: 0.25,
            default_note_len: 0.25,
            selected_note_ids: Vec::new(),
        }
    }
}

// ── Song view interaction state ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SongTool {
    Draw,
    Select,
    Erase,
}

#[derive(Debug, Clone)]
pub struct SongViewState {
    pub tool: SongTool,
    pub snap_beats: f64,
    pub zoom_x: f32,   // pixels per beat
    pub scroll_x: f32, // beat offset
    pub selected_clip_ids: Vec<u64>,
    pub drag_clip_id: Option<u64>,
    pub drag_start_beat: f64,
    pub drag_offset_beat: f64,
}

impl Default for SongViewState {
    fn default() -> Self {
        Self {
            tool: SongTool::Draw,
            snap_beats: 1.0,
            zoom_x: 32.0,
            scroll_x: 0.0,
            selected_clip_ids: Vec::new(),
            drag_clip_id: None,
            drag_start_beat: 0.0,
            drag_offset_beat: 0.0,
        }
    }
}

// ── Undo/Redo ─────────────────────────────────────────────────────────────────

/// A snapshot of the session tracks for undo/redo.
/// We snapshot only tracks (clips + notes) — not engine state.
#[derive(Debug, Clone)]
pub struct UndoSnapshot {
    pub tracks: Vec<Track>,
    pub label: String,
}

// ── Clipboard ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Clipboard {
    Clips(Vec<Clip>),
    Notes(Vec<MidiNote>),
    Empty,
}

// ── Tap tempo ─────────────────────────────────────────────────────────────────

pub struct TapTempo {
    pub taps: Vec<Instant>,
}

impl TapTempo {
    pub fn new() -> Self { Self { taps: Vec::new() } }

    pub fn tap(&mut self) -> Option<f32> {
        let now = Instant::now();
        // Discard taps older than 2 seconds
        self.taps.retain(|t| now.duration_since(*t).as_secs_f32() < 2.0);
        self.taps.push(now);
        if self.taps.len() < 2 { return None; }
        // Average interval between taps
        let intervals: Vec<f32> = self.taps.windows(2)
            .map(|w| w[1].duration_since(w[0]).as_secs_f32())
            .collect();
        let avg = intervals.iter().sum::<f32>() / intervals.len() as f32;
        Some(60.0 / avg)
    }
}

// ── AppState ──────────────────────────────────────────────────────────────────

pub struct AppStateInner {
    // Engine
    pub scheduler: Arc<Mutex<Scheduler>>,
    pub master_engine: Option<MasterEngine>,
    pub engine_status: EngineStatus,
    pub audio_active: bool,

    // Pending MIDI events — UI pushes, engine drains
    pub midi_queue: Vec<PendingMidi>,

    // Transport
    pub transport: Transport,
    pub beat_accumulator: f64,

    // Session
    pub tracks: Vec<Track>,
    pub channels: Vec<MixerChannel>,
    pub selected_track_id: Option<u64>,
    pub selected_clip_id: Option<u64>,

    // Undo/Redo
    pub undo_stack: Vec<UndoSnapshot>,
    pub redo_stack: Vec<UndoSnapshot>,

    // Clipboard
    pub clipboard: Clipboard,

    // Tap tempo
    pub tap_tempo: TapTempo,

    // View states
    pub active_view: ActiveView,
    pub piano_roll: PianoRollState,
    pub song_view: SongViewState,

    // Misc
    pub scale_id: String,
    pub browser_open: bool,
    pub browser_width: f32,
    pub properties_open: bool,
    pub properties_height: f32,
    // Which track's instrument panel is open
    pub instrument_panel_track: Option<u64>,

    next_id: u64,
}

impl AppStateInner {
    pub fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Find a clip by id across all tracks
    pub fn find_clip_mut(&mut self, clip_id: u64) -> Option<&mut Clip> {
        for track in &mut self.tracks {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                return Some(clip);
            }
        }
        None
    }

    pub fn find_clip(&self, clip_id: u64) -> Option<&Clip> {
        for track in &self.tracks {
            if let Some(clip) = track.clips.iter().find(|c| c.id == clip_id) {
                return Some(clip);
            }
        }
        None
    }

    pub fn track_index_for_clip(&self, clip_id: u64) -> Option<usize> {
        for (i, track) in self.tracks.iter().enumerate() {
            if track.clips.iter().any(|c| c.id == clip_id) {
                return Some(i);
            }
        }
        None
    }

    /// Push current tracks state onto undo stack
    pub fn push_undo(&mut self, label: &str) {
        self.undo_stack.push(UndoSnapshot {
            tracks: self.tracks.clone(),
            label: label.to_string(),
        });
        // Cap undo history at 50 steps
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            self.redo_stack.push(UndoSnapshot {
                tracks: self.tracks.clone(),
                label: snap.label.clone(),
            });
            self.tracks = snap.tracks;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            self.undo_stack.push(UndoSnapshot {
                tracks: self.tracks.clone(),
                label: snap.label.clone(),
            });
            self.tracks = snap.tracks;
        }
    }

    /// Copy selected clips to clipboard
    pub fn copy_selected_clips(&mut self) {
        let clips: Vec<Clip> = self.tracks.iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| self.song_view.selected_clip_ids.contains(&c.id))
            .cloned()
            .collect();
        if !clips.is_empty() {
            self.clipboard = Clipboard::Clips(clips);
        }
    }

    /// Copy selected notes to clipboard
    pub fn copy_selected_notes(&mut self) {
        if let Some(clip_id) = self.piano_roll.open_clip_id {
            if let Some(clip) = self.find_clip(clip_id) {
                let notes: Vec<MidiNote> = clip.notes.iter()
                    .filter(|n| self.piano_roll.selected_note_ids.contains(&n.id))
                    .cloned()
                    .collect();
                if !notes.is_empty() {
                    self.clipboard = Clipboard::Notes(notes);
                }
            }
        }
    }

    /// Paste clips at playhead position
    pub fn paste_clips(&mut self) {
        if let Clipboard::Clips(clips) = self.clipboard.clone() {
            let min_beat = clips.iter().map(|c| c.start_beat).fold(f64::MAX, f64::min);
            let offset = self.transport.playhead_beat - min_beat;
            self.push_undo("Paste");
            for clip in clips {
                // Find the track by track_id
                if let Some(track) = self.tracks.iter_mut().find(|t| t.id == clip.track_id) {
                    let new_id = self.next_id;
                    self.next_id += 1;
                    track.clips.push(Clip {
                        id: new_id,
                        start_beat: (clip.start_beat + offset).max(0.0),
                        ..clip
                    });
                }
            }
        }
    }

    /// Paste notes at cursor position in piano roll
    pub fn paste_notes(&mut self) {
        if let Clipboard::Notes(notes) = self.clipboard.clone() {
            if let Some(clip_id) = self.piano_roll.open_clip_id {
                let min_beat = notes.iter().map(|n| n.beat).fold(f64::MAX, f64::min);
                self.push_undo("Paste Notes");
                if let Some(clip) = self.find_clip_mut(clip_id) {
                    for note in notes {
                        let new_id = clip.notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                        clip.notes.push(MidiNote {
                            id: new_id,
                            beat: note.beat - min_beat,
                            ..note
                        });
                    }
                }
            }
        }
    }

    /// Duplicate selected clips (place immediately after)
    pub fn duplicate_selected_clips(&mut self) {
        let selected: Vec<Clip> = self.tracks.iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| self.song_view.selected_clip_ids.contains(&c.id))
            .cloned()
            .collect();
        if selected.is_empty() { return; }
        self.push_undo("Duplicate");
        for clip in selected {
            let new_start = clip.start_beat + clip.length_beats;
            if let Some(track) = self.tracks.iter_mut().find(|t| t.id == clip.track_id) {
                let new_id = self.next_id;
                self.next_id += 1;
                track.clips.push(Clip {
                    id: new_id,
                    start_beat: new_start,
                    ..clip
                });
            }
        }
    }

    /// Delete selected clips
    pub fn delete_selected_clips(&mut self) {
        if self.song_view.selected_clip_ids.is_empty() { return; }
        self.push_undo("Delete Clips");
        let ids = self.song_view.selected_clip_ids.clone();
        for track in &mut self.tracks {
            track.clips.retain(|c| !ids.contains(&c.id));
        }
        self.song_view.selected_clip_ids.clear();
    }

    /// Delete selected notes in piano roll
    pub fn delete_selected_notes(&mut self) {
        if self.piano_roll.selected_note_ids.is_empty() { return; }
        self.push_undo("Delete Notes");
        let ids = self.piano_roll.selected_note_ids.clone();
        if let Some(clip_id) = self.piano_roll.open_clip_id {
            if let Some(clip) = self.find_clip_mut(clip_id) {
                clip.notes.retain(|n| !ids.contains(&n.id));
            }
        }
        self.piano_roll.selected_note_ids.clear();
    }

    /// Select all clips on all tracks
    pub fn select_all_clips(&mut self) {
        self.song_view.selected_clip_ids = self.tracks.iter()
            .flat_map(|t| t.clips.iter().map(|c| c.id))
            .collect();
    }

    /// Select all notes in open clip
    pub fn select_all_notes(&mut self) {
        if let Some(clip_id) = self.piano_roll.open_clip_id {
            if let Some(clip) = self.find_clip(clip_id) {
                self.piano_roll.selected_note_ids = clip.notes.iter().map(|n| n.id).collect();
            }
        }
    }

    /// Drain MIDI queue and send events to the master engine
    pub fn flush_midi(&mut self) {
        if self.master_engine.is_none() { return; }
        let events: Vec<PendingMidi> = self.midi_queue.drain(..).collect();
        // We hold &mut self (AppStateInner) so we can access both scheduler and master_engine.
        // Use try_lock on the scheduler — if the audio thread holds it, skip (events will be
        // retried next tick via the queue, but we've already drained it here so just process
        // directly by locking).
        if let Ok(mut sched) = self.scheduler.try_lock() {
            if let Some(ref mut engine) = self.master_engine {
                for pm in events {
                    engine.send_event(&mut sched, pm.track_idx, pm.event);
                }
            }
        }
        // If try_lock failed, events are lost this tick — acceptable for preview notes.
    }

    /// Advance playhead by delta_seconds and fire note events from clips
    pub fn tick_transport(&mut self, delta_secs: f64) {
        if !self.transport.is_playing { return; }

        let beats_per_sec = self.transport.bpm as f64 / 60.0;
        let delta_beats = delta_secs * beats_per_sec;
        let old_beat = self.transport.playhead_beat;
        let new_beat = old_beat + delta_beats;

        // Fire note-on/off events for notes that fall in [old_beat, new_beat)
        let tracks_snapshot: Vec<(usize, Vec<Clip>)> = self.tracks.iter()
            .enumerate()
            .map(|(i, t)| (i, t.clips.clone()))
            .collect();

        for (track_idx, clips) in &tracks_snapshot {
            for clip in clips {
                let clip_end = clip.start_beat + clip.length_beats;
                if clip.start_beat > new_beat || clip_end < old_beat { continue; }

                for note in &clip.notes {
                    let abs_start = clip.start_beat + note.beat;
                    let abs_end   = abs_start + note.duration;

                    // Note on
                    if abs_start >= old_beat && abs_start < new_beat {
                        self.midi_queue.push(PendingMidi {
                            track_idx: *track_idx,
                            event: MidiEvent::NoteOn { pitch: note.pitch, velocity: note.velocity },
                        });
                    }
                    // Note off
                    if abs_end >= old_beat && abs_end < new_beat {
                        self.midi_queue.push(PendingMidi {
                            track_idx: *track_idx,
                            event: MidiEvent::NoteOff { pitch: note.pitch },
                        });
                    }
                }
            }
        }

        self.transport.playhead_beat = new_beat;

        // Loop
        if self.transport.loop_enabled && new_beat >= self.transport.loop_end {
            self.transport.playhead_beat = self.transport.loop_start;
            // All notes off on loop wrap
            for i in 0..self.tracks.len() {
                self.midi_queue.push(PendingMidi {
                    track_idx: i,
                    event: MidiEvent::AllNotesOff,
                });
            }
        }

        self.flush_midi();
    }
}

pub type AppState = Arc<Mutex<AppStateInner>>;

const TRACK_COLORS: &[u32] = &[
    0x4db8ffff, 0xa78bfaff, 0x34d399ff, 0xf97316ff,
    0xf43f5eff, 0xfbbf24ff, 0x06b6d4ff, 0x8b5cf6ff,
];

pub fn create_app_state() -> AppState {
    let scheduler = Arc::new(Mutex::new(Scheduler::new(48_000.0)));

    let mut next_id = 0u64;
    let track_names = ["Kick", "Bass", "Melody", "Pad"];
    let presets = [
        InstrumentPreset::kick(),
        InstrumentPreset::bass(),
        InstrumentPreset::lead(),
        InstrumentPreset::pad(),
    ];
    let tracks: Vec<Track> = track_names.iter().enumerate().map(|(i, name)| {
        next_id += 1;
        Track {
            id: next_id,
            name: name.to_string(),
            track_type: TrackType::Instrument,
            color: TRACK_COLORS[i % TRACK_COLORS.len()],
            muted: false, solo: false, armed: false,
            volume: 0.8, pan: 0.0, height: 72.0,
            clips: Vec::new(), effects: Vec::new(),
            instrument: presets[i].clone(),
        }
    }).collect();

    let channels: Vec<MixerChannel> = tracks.iter().map(|t| {
        next_id += 1;
        MixerChannel {
            id: next_id, name: t.name.clone(), color: t.color,
            volume: 0.8, pan: 0.0, muted: false, solo: false,
        }
    }).collect();

    Arc::new(Mutex::new(AppStateInner {
        scheduler,
        master_engine: None,
        engine_status: EngineStatus::Starting,
        audio_active: false,
        midi_queue: Vec::new(),
        transport: Transport::default(),
        beat_accumulator: 0.0,
        tracks,
        channels,
        selected_track_id: None,
        selected_clip_id: None,
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        clipboard: Clipboard::Empty,
        tap_tempo: TapTempo::new(),
        active_view: ActiveView::Song,
        piano_roll: PianoRollState::default(),
        song_view: SongViewState::default(),
        scale_id: "12-tet".into(),
        browser_open: false,
        browser_width: 220.0,
        properties_open: false,
        properties_height: 180.0,
        instrument_panel_track: None,
        next_id,
    }))
}
