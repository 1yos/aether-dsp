//! AppState — shared state between the audio engine and the UI.
//!
//! This replaces the WebSocket + Zustand store from the React UI.
//! The UI reads from AppState directly — no serialization, no round trips.

use std::sync::{Arc, Mutex};
use aetherdsp_core::scheduler::Scheduler;

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
    pub color: u32, // RGBA packed
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

// ── AppState ──────────────────────────────────────────────────────────────────

pub struct AppStateInner {
    // Engine
    pub scheduler: Arc<Mutex<Scheduler>>,
    pub engine_status: EngineStatus,
    pub audio_active: bool,

    // Transport
    pub transport: Transport,

    // Session
    pub tracks: Vec<Track>,
    pub channels: Vec<MixerChannel>,
    pub selected_track_id: Option<u64>,
    pub selected_clip_id: Option<u64>,

    // Piano roll context
    pub piano_roll_clip_id: Option<u64>,
    pub piano_roll_track_id: Option<u64>,
    pub scale_id: String,
    pub rhythm_id: String,

    // Layout
    pub active_view: ActiveView,
    pub browser_open: bool,
    pub browser_width: f32,
    pub properties_open: bool,
    pub properties_height: f32,

    // ID counter
    next_id: u64,
}

impl AppStateInner {
    pub fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

pub type AppState = Arc<Mutex<AppStateInner>>;

const TRACK_COLORS: &[u32] = &[
    0x4db8ffff, 0xa78bfaff, 0x34d399ff, 0xf97316ff,
    0xf43f5eff, 0xfbbf24ff, 0x06b6d4ff, 0x8b5cf6ff,
];

pub fn new_app_state() -> AppState {
    let scheduler = Arc::new(Mutex::new(Scheduler::new(48_000.0)));

    // Default tracks
    let mut next_id = 0u64;
    let tracks: Vec<Track> = ["Kick", "Bass", "Melody", "Pad"]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            next_id += 1;
            Track {
                id: next_id,
                name: name.to_string(),
                track_type: TrackType::Instrument,
                color: TRACK_COLORS[i % TRACK_COLORS.len()],
                muted: false,
                solo: false,
                armed: false,
                volume: 0.8,
                pan: 0.0,
                height: 72.0,
                clips: Vec::new(),
                effects: Vec::new(),
            }
        })
        .collect();

    let channels: Vec<MixerChannel> = tracks
        .iter()
        .map(|t| {
            next_id += 1;
            MixerChannel {
                id: next_id,
                name: t.name.clone(),
                color: t.color,
                volume: 0.8,
                pan: 0.0,
                muted: false,
                solo: false,
            }
        })
        .collect();

    Arc::new(Mutex::new(AppStateInner {
        scheduler,
        engine_status: EngineStatus::Starting,
        audio_active: false,
        transport: Transport::default(),
        tracks,
        channels,
        selected_track_id: None,
        selected_clip_id: None,
        piano_roll_clip_id: None,
        piano_roll_track_id: None,
        scale_id: "12-tet".into(),
        rhythm_id: "4-4".into(),
        active_view: ActiveView::Song,
        browser_open: true,
        browser_width: 220.0,
        properties_open: true,
        properties_height: 180.0,
        next_id,
    }))
}

impl AppState {
    pub fn new() -> Self {
        new_app_state()
    }
}
