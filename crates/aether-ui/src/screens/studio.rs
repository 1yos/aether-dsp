//! Main studio screen — transport, track list, arrangement, editors, mixer.

use iced::widget::{column, container, row};
use iced::{Element, Length};

use crate::components::{
    add_track,
    arrangement, mixer, piano_roll, step_seq, track_list, transport,
    track_strip,
};
use crate::engine::Engine;
use crate::project::{ClipContent, Project, TrackId};
use crate::theme;

#[derive(Debug, Clone)]
pub enum Message {
    Transport(transport::Message),
    TrackList(track_list::Message),
    Arrangement(arrangement::Message),
    AddTrack(add_track::Message),
    PianoRoll(piano_roll::Message),
    StepSeq(step_seq::Message),
    Mixer(mixer::Message),
}

struct OpenClip {
    track_id: TrackId,
    clip_idx: usize,
}

pub struct Studio {
    pub project: Project,
    transport: transport::Transport,
    mixer: mixer::Mixer,
    selected_track: Option<TrackId>,
    show_add_track: bool,
    open_clip: Option<OpenClip>,
    next_color_idx: usize,
    pub playhead_bar: f32,
}

impl Studio {
    pub fn new(project: Project) -> Self {
        let bpm = project.bpm;
        Self {
            project,
            transport: transport::Transport {
                bpm,
                bpm_str: format!("{}", bpm as u32),
                ..Default::default()
            },
            mixer: mixer::Mixer::default(),
            selected_track: None,
            show_add_track: false,
            open_clip: None,
            next_color_idx: 0,
            playhead_bar: 0.0,
        }
    }

    pub fn update(&mut self, message: Message, engine: &mut Option<Engine>) {
        match message {
            // ── Transport ─────────────────────────────────────────────────────
            Message::Transport(msg) => match msg {
                transport::Message::Play => {
                    self.transport.is_playing = true;
                    if let Some(e) = engine {
                        // Collect all MIDI notes from all tracks into ScheduledNotes
                        let mut notes: Vec<crate::engine::ScheduledNote> = Vec::new();
                        for track in &self.project.tracks {
                            for clip in &track.clips {
                                if let crate::project::ClipContent::MidiNotes(ref midi) =
                                    clip.content
                                {
                                    for note in midi {
                                        // Offset note by clip start position
                                        let beats_per_bar = 4.0_f32;
                                        notes.push(crate::engine::ScheduledNote {
                                            track_id: track.id,
                                            pitch: note.pitch,
                                            start_beat: clip.start_bar * beats_per_bar
                                                + note.start_beat,
                                            length_beats: note.length_beats,
                                        });
                                    }
                                }
                            }
                        }
                        e.start_playback(notes, self.project.bpm);
                    }
                }
                transport::Message::Stop => {
                    self.transport.is_playing = false;
                    self.playhead_bar = 0.0;
                    if let Some(e) = engine {
                        e.stop_playback();
                        e.set_mute(true);
                    }
                }
                transport::Message::BpmChanged(s) => {
                    self.transport.bpm_str = s.clone();
                    if let Ok(bpm) = s.parse::<f32>() {
                        if bpm > 0.0 && bpm < 1000.0 {
                            self.project.bpm = bpm;
                            self.transport.bpm = bpm;
                        }
                    }
                }
            },

            // ── Track List ────────────────────────────────────────────────────
            Message::TrackList(msg) => match msg {
                track_list::Message::AddTrack => {
                    self.show_add_track = true;
                }
                track_list::Message::Strip(strip_msg) => match strip_msg {
                    track_strip::Message::Select(id) => {
                        self.selected_track = Some(id);
                        self.open_clip = None;
                    }
                    track_strip::Message::VolumeChanged(id, vol) => {
                        if let Some(t) = self.project.find_track_mut(id) {
                            t.volume = vol;
                        }
                        if let Some(e) = engine {
                            e.set_track_volume(id, vol);
                        }
                    }
                    track_strip::Message::ToggleMute(id) => {
                        if let Some(t) = self.project.find_track_mut(id) {
                            t.muted = !t.muted;
                            let m = t.muted;
                            if let Some(e) = engine {
                                e.set_track_mute(id, m);
                            }
                        }
                    }
                    track_strip::Message::ToggleSolo(id) => {
                        if let Some(t) = self.project.find_track_mut(id) {
                            t.soloed = !t.soloed;
                        }
                    }
                    track_strip::Message::Remove(id) => {
                        self.project.tracks.retain(|t| t.id != id);
                        if self.selected_track == Some(id) {
                            self.selected_track = None;
                            self.open_clip = None;
                        }
                        if let Some(e) = engine {
                            e.remove_track(id);
                        }
                    }
                },
            },

            // ── Arrangement ───────────────────────────────────────────────────
            Message::Arrangement(msg) => match msg {
                arrangement::Message::ClickBar(track_id, bar) => {
                    self.project.add_clip(track_id, bar, 4.0);
                    self.selected_track = Some(track_id);
                }
                arrangement::Message::OpenClip(track_id, clip_idx) => {
                    self.selected_track = Some(track_id);
                    self.open_clip = Some(OpenClip { track_id, clip_idx });
                }
            },

            // ── Add Track Panel ───────────────────────────────────────────────
            Message::AddTrack(msg) => match msg {
                add_track::Message::Close => {
                    self.show_add_track = false;
                }
                add_track::Message::SelectInstrument(template) => {
                    let color = theme::track_color(self.next_color_idx);
                    self.next_color_idx += 1;
                    let id = self.project.add_track(
                        &template.name,
                        template.track_type,
                        template.tuning,
                        color,
                    );
                    if let Some(e) = engine {
                        e.add_track(id, template.tuning, 0.75);
                    }
                    self.selected_track = Some(id);
                    self.show_add_track = false;
                }
            },

            // ── Piano Roll ────────────────────────────────────────────────────
            Message::PianoRoll(msg) => match msg {
                piano_roll::Message::Close => {
                    self.open_clip = None;
                }
                piano_roll::Message::AddNote { pitch, start_beat, length_beats } => {
                    if let Some(open) = &self.open_clip {
                        let (tid, cidx) = (open.track_id, open.clip_idx);
                        if let Some(track) = self.project.find_track_mut(tid) {
                            if let Some(clip) = track.clips.get_mut(cidx) {
                                if let ClipContent::MidiNotes(ref mut notes) = clip.content {
                                    notes.push(crate::project::MidiNote {
                                        pitch,
                                        start_beat,
                                        length_beats,
                                        velocity: 100,
                                    });
                                }
                            }
                        }
                    }
                }
                piano_roll::Message::RemoveNote(idx) => {
                    if let Some(open) = &self.open_clip {
                        let (tid, cidx) = (open.track_id, open.clip_idx);
                        if let Some(track) = self.project.find_track_mut(tid) {
                            if let Some(clip) = track.clips.get_mut(cidx) {
                                if let ClipContent::MidiNotes(ref mut notes) = clip.content {
                                    if idx < notes.len() {
                                        notes.remove(idx);
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ── Step Sequencer ────────────────────────────────────────────────
            Message::StepSeq(msg) => match msg {
                step_seq::Message::Close => {
                    self.open_clip = None;
                }
                step_seq::Message::ToggleStep(row, step) => {
                    if let Some(open) = &self.open_clip {
                        let (tid, cidx) = (open.track_id, open.clip_idx);
                        if let Some(track) = self.project.find_track_mut(tid) {
                            if let Some(clip) = track.clips.get_mut(cidx) {
                                if let ClipContent::DrumPattern(ref mut pat) = clip.content {
                                    if let Some(r) = pat.rows.get_mut(row) {
                                        if let Some(s) = r.steps.get_mut(step) {
                                            *s = !*s;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ── Mixer ─────────────────────────────────────────────────────────
            Message::Mixer(msg) => match msg {
                mixer::Message::ToggleExpand => {
                    self.mixer.expanded = !self.mixer.expanded;
                }
                mixer::Message::VolumeChanged(id, vol) => {
                    if let Some(t) = self.project.find_track_mut(id) {
                        t.volume = vol;
                    }
                    if let Some(e) = engine {
                        e.set_track_volume(id, vol);
                    }
                }
                mixer::Message::ToggleMute(id) => {
                    // u32::MAX is the master placeholder — ignore
                    if id.0 == u32::MAX {
                        return;
                    }
                    if let Some(t) = self.project.find_track_mut(id) {
                        t.muted = !t.muted;
                        let m = t.muted;
                        if let Some(e) = engine {
                            e.set_track_mute(id, m);
                        }
                    }
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let transport = self.transport.view().map(Message::Transport);

        let track_list =
            track_list::view(&self.project.tracks, self.selected_track).map(Message::TrackList);

        // Right: editor (open clip) or arrangement
        let main_area: Element<'_, Message> = if let Some(open) = &self.open_clip {
            self.view_editor(open)
        } else {
            arrangement::view(&self.project.tracks, self.selected_track, self.playhead_bar)
                .map(Message::Arrangement)
        };

        // Left panel: track list + optional add-track slide-in
        let left = if self.show_add_track {
            row![track_list, add_track::view().map(Message::AddTrack)]
                .height(Length::Fill)
                .into()
        } else {
            track_list
        };

        let workspace = row![left, main_area].height(Length::Fill);

        let mixer_bar = self.mixer.view(&self.project.tracks).map(Message::Mixer);

        container(column![transport, workspace, mixer_bar])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(crate::theme::Theme::APP_BG)),
                ..Default::default()
            })
            .into()
    }

    fn view_editor<'a>(&'a self, open: &OpenClip) -> Element<'a, Message> {
        let track = match self.project.find_track(open.track_id) {
            Some(t) => t,
            None => return iced::widget::Space::new(Length::Fill, Length::Fill).into(),
        };
        let clip = match track.clips.get(open.clip_idx) {
            Some(c) => c,
            None => return iced::widget::Space::new(Length::Fill, Length::Fill).into(),
        };

        match &clip.content {
            ClipContent::MidiNotes(notes) => {
                let roll = piano_roll::PianoRoll {
                    notes: notes.clone(),
                    tuning: track.tuning,
                    title: clip.name.clone(),
                };
                // view(self) consumes roll → Element<'static, Message>
                let elem: iced::Element<'static, piano_roll::Message> = roll.view();
                elem.map(Message::PianoRoll)
            }
            ClipContent::DrumPattern(pat) => {
                step_seq::view(pat, &clip.name).map(Message::StepSeq)
            }
        }
    }
}
