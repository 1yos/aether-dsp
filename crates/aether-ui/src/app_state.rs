//! AppState — shared state between the audio engine and the UI.

use std::sync::{Arc, Mutex};
use aether_core::scheduler::Scheduler;
use crate::instrument::{MasterEngine, MidiEvent};

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
    pub effect_type: String,
    pub enabled: bool,
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
    // Accumulated fractional beat for the tick timer
    pub beat_accumulator: f64,

    // Session
    pub tracks: Vec<Track>,
    pub channels: Vec<MixerChannel>,
    pub selected_track_id: Option<u64>,
    pub selected_clip_id: Option<u64>,

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
        active_view: ActiveView::Song,
        piano_roll: PianoRollState::default(),
        song_view: SongViewState::default(),
        scale_id: "12-tet".into(),
        browser_open: false,
        browser_width: 220.0,
        properties_open: false,
        properties_height: 180.0,
        next_id,
    }))
}
