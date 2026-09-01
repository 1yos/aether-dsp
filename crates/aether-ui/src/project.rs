//! Project data model — tracks, clips, notes.

use iced::Color;

// ── IDs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipId(pub u32);

// ── Track ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub color: Color,
    pub track_type: TrackType,
    pub tuning: TuningSystem,
    pub volume: f32,
    pub muted: bool,
    pub soloed: bool,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Melodic,
    Drum,
}

// ── Tuning ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuningSystem {
    // Ethiopian
    EthiopianTizitaMajor,
    EthiopianTizitaMinor,
    EthiopianBatiMinor,
    EthiopianBatiMajor,
    EthiopianAmbassel,
    EthiopianAnchihoye,
    // Arabic
    ArabicRast,
    ArabicBayati,
    ArabicHijaz,
    // Indian
    IndianYaman,
    // Gamelan
    GamelanSlendro,
    GamelanPelog,
    // Western
    EqualTemperament,
    JustIntonation,
}

impl TuningSystem {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::EthiopianTizitaMajor => "Tizita (Major)",
            Self::EthiopianTizitaMinor => "Tizita (Minor)",
            Self::EthiopianBatiMinor   => "Bati (Minor)",
            Self::EthiopianBatiMajor   => "Bati (Major)",
            Self::EthiopianAmbassel    => "Ambassel",
            Self::EthiopianAnchihoye   => "Anchihoye",
            Self::ArabicRast           => "Maqam Rast",
            Self::ArabicBayati         => "Maqam Bayati",
            Self::ArabicHijaz          => "Maqam Hijaz",
            Self::IndianYaman          => "Raga Yaman",
            Self::GamelanSlendro       => "Gamelan Slendro",
            Self::GamelanPelog         => "Gamelan Pelog",
            Self::EqualTemperament     => "12-TET",
            Self::JustIntonation       => "Just Intonation",
        }
    }

    pub fn category(self) -> &'static str {
        match self {
            Self::EthiopianTizitaMajor
            | Self::EthiopianTizitaMinor
            | Self::EthiopianBatiMinor
            | Self::EthiopianBatiMajor
            | Self::EthiopianAmbassel
            | Self::EthiopianAnchihoye => "Ethiopian",
            Self::ArabicRast | Self::ArabicBayati | Self::ArabicHijaz => "Arabic",
            Self::IndianYaman => "Indian",
            Self::GamelanSlendro | Self::GamelanPelog => "Gamelan",
            Self::EqualTemperament | Self::JustIntonation => "Western",
        }
    }

    /// All tuning systems in display order (Ethiopian first).
    pub fn all() -> &'static [TuningSystem] {
        &[
            Self::EthiopianTizitaMajor,
            Self::EthiopianTizitaMinor,
            Self::EthiopianBatiMinor,
            Self::EthiopianBatiMajor,
            Self::EthiopianAmbassel,
            Self::EthiopianAnchihoye,
            Self::ArabicRast,
            Self::ArabicBayati,
            Self::ArabicHijaz,
            Self::IndianYaman,
            Self::GamelanSlendro,
            Self::GamelanPelog,
            Self::EqualTemperament,
            Self::JustIntonation,
        ]
    }
}

// ── Clips ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Clip {
    pub id: ClipId,
    pub name: String,
    pub start_bar: f32,
    pub length_bars: f32,
    pub content: ClipContent,
}

#[derive(Debug, Clone)]
pub enum ClipContent {
    MidiNotes(Vec<MidiNote>),
    DrumPattern(DrumPattern),
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub pitch: u8,
    pub start_beat: f32,
    pub length_beats: f32,
    pub velocity: u8,
}

#[derive(Debug, Clone)]
pub struct DrumPattern {
    pub steps: u8,
    pub rows: Vec<DrumRow>,
}

#[derive(Debug, Clone)]
pub struct DrumRow {
    pub name: String,
    pub note: u8,
    pub steps: Vec<bool>,
}

impl DrumPattern {
    pub fn default_kit() -> Self {
        Self {
            steps: 16,
            rows: vec![
                DrumRow { name: "Kick".into(),    note: 36, steps: vec![false; 16] },
                DrumRow { name: "Snare".into(),   note: 38, steps: vec![false; 16] },
                DrumRow { name: "Hi-hat".into(),  note: 42, steps: vec![false; 16] },
                DrumRow { name: "Open HH".into(), note: 46, steps: vec![false; 16] },
            ],
        }
    }
}

// ── Project ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub bpm: f32,
    pub tracks: Vec<Track>,
    next_track_id: u32,
    next_clip_id: u32,
}

impl Project {
    pub fn new(name: impl Into<String>, bpm: f32) -> Self {
        Self {
            name: name.into(),
            bpm,
            tracks: Vec::new(),
            next_track_id: 0,
            next_clip_id: 0,
        }
    }

    pub fn add_track(&mut self, name: impl Into<String>, track_type: TrackType, tuning: TuningSystem, color: Color) -> TrackId {
        let id = TrackId(self.next_track_id);
        self.next_track_id += 1;
        self.tracks.push(Track {
            id,
            name: name.into(),
            color,
            track_type,
            tuning,
            volume: 0.75,
            muted: false,
            soloed: false,
            clips: Vec::new(),
        });
        id
    }

    pub fn add_clip(&mut self, track_id: TrackId, start_bar: f32, length_bars: f32) -> Option<ClipId> {
        let id = ClipId(self.next_clip_id);
        self.next_clip_id += 1;
        let track = self.tracks.iter_mut().find(|t| t.id == track_id)?;
        let content = match track.track_type {
            TrackType::Melodic => ClipContent::MidiNotes(Vec::new()),
            TrackType::Drum    => ClipContent::DrumPattern(DrumPattern::default_kit()),
        };
        track.clips.push(Clip {
            id,
            name: format!("Clip {}", id.0 + 1),
            start_bar,
            length_bars,
            content,
        });
        Some(id)
    }

    pub fn find_track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    pub fn find_track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }
}
