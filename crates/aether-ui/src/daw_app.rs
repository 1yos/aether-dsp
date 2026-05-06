//! DawApp — the iced Application struct.
use iced::{
    widget::{button, canvas, column, container, row, scrollable, text},
    Alignment, Color, Element, Length, Task, Theme,
};
use crate::app_state::{AppState, ActiveView};

#[derive(Debug, Clone)]
pub enum Message {
    SetView(ActiveView),
    Play, Stop, ToggleRecord, AddTrack, Tick,
}

pub struct DawApp { pub state: AppState }

impl DawApp {
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        (Self { state }, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetView(v) => { self.state.lock().unwrap().active_view = v; }
            Message::Play => { self.state.lock().unwrap().transport.is_playing = true; }
            Message::Stop => { let mut s = self.state.lock().unwrap(); s.transport.is_playing = false; s.transport.playhead_beat = 0.0; }
            Message::ToggleRecord => { let mut s = self.state.lock().unwrap(); s.transport.is_recording = !s.transport.is_recording; }
            Message::AddTrack => {
                let mut s = self.state.lock().unwrap();
                let id = s.next_id();
                let idx = s.tracks.len();
                let colors = [0x4db8ffff_u32,0xa78bfaff,0x34d399ff,0xf97316ff,0xf43f5eff,0xfbbf24ff,0x06b6d4ff,0x8b5cf6ff];
                s.tracks.push(crate::app_state::Track {
                    id, name: format!("Track {}", idx+1),
                    track_type: crate::app_state::TrackType::Instrument,
                    color: colors[idx % colors.len()],
                    muted: false, solo: false, armed: false,
                    volume: 0.8, pan: 0.0, height: 72.0,
                    clips: Vec::new(), effects: Vec::new(),
                });
            }
            Message::Tick => {
                let mut s = self.state.lock().unwrap();
                if s.transport.is_playing {
                    s.transport.playhead_beat += s.transport.bpm as f64 / 60.0 / 60.0;
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let state = self.state.lock().unwrap();
        let active = state.active_view;
        let transport = state.transport.clone();
        let tracks = state.tracks.clone();
        let channels = state.channels.clone();
        let engine_ok = state.engine_status == crate::app_state::EngineStatus::Running;
        drop(state);

        let views = [
            ("≡ Song", ActiveView::Song),
            ("♩ Piano Roll", ActiveView::PianoRoll),
            ("⊟ Mixer", ActiveView::Mixer),
            ("⬡ Patcher", ActiveView::Patcher),
            ("🎚 Perform", ActiveView::Perform),
        ];

        let tabs = views.iter().fold(
            row![].spacing(2),
            |r, (label, view)| {
                let is_active = active == *view;
                let msg = Message::SetView(*view);
                let lbl = label.to_string();
                r.push(
                    button(text(lbl).size(12))
                        .on_press(msg)
                        .style(move |_, _| tab_style(is_active))
                )
            }
        );

        let is_playing = transport.is_playing;
        let is_recording = transport.is_recording;
        let bpm = transport.bpm;

        let top_bar = container(
            row![
                text("Aether Studio").size(15).color(Color::from_rgb(0.3, 0.72, 1.0)),
                tabs,
                button(text("■").size(12)).on_press(Message::Stop).style(|_,_| btn_style(false, false)),
                button(text(if is_playing {"▶ Playing"} else {"▶ Play"}).size(12)).on_press(Message::Play).style(move |_,_| btn_style(is_playing, false)),
                button(text("● Rec").size(12)).on_press(Message::ToggleRecord).style(move |_,_| btn_style(is_recording, true)),
                text(format!("BPM {:.0}", bpm)).size(12).color(Color::WHITE),
                iced::widget::horizontal_space(),
                text(if engine_ok {"● Live"} else {"● Offline"}).size(11)
                    .color(if engine_ok {Color::from_rgb(0.0,0.9,0.63)} else {Color::from_rgb(0.94,0.33,0.31)}),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .padding(8),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.04,0.07,0.13))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        });

        let workspace: Element<Message> = match active {
            ActiveView::Song => song_view_el(tracks, transport),
            ActiveView::Mixer => mixer_view_el(channels),
            _ => placeholder_view(match active {
                ActiveView::PianoRoll => "Piano Roll — double-click a clip in Song view",
                ActiveView::Patcher   => "Patcher — node graph (GPU rendering)",
                ActiveView::Perform   => "Perform — live mixer",
                _ => "",
            }),
        };

        column![top_bar, workspace].into()
    }
}

// ── Song view ─────────────────────────────────────────────────────────────────

fn song_view_el(tracks: Vec<crate::app_state::Track>, transport: crate::app_state::Transport) -> Element<'static, Message> {
    let toolbar = container(
        row![
            button(text("+ Instrument").size(11)).on_press(Message::AddTrack).style(|_,_| btn_style(false, false)),
            button(text("+ Audio").size(11)).style(|_,_| btn_style(false, false)),
        ].spacing(6).padding(6),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
        border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
        ..Default::default()
    });

    let headers: Vec<Element<Message>> = tracks.iter().map(|t| {
        let c = color_from_u32(t.color);
        let name = t.name.clone();
        let vol = (t.volume * 100.0) as u32;
        container(
            row![
                container(iced::widget::vertical_space()).width(3).height(Length::Fill)
                    .style(move |_| container::Style { background: Some(iced::Background::Color(c)), ..Default::default() }),
                column![
                    text(name).size(11).color(Color::WHITE),
                    text(format!("{}%", vol)).size(9).color(Color::from_rgb(0.28,0.4,0.52)),
                ].spacing(2).padding([4,8]),
            ].height(Length::Fixed(72.0)),
        )
        .width(Length::Fixed(200.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))),
            border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    let header_col: Element<Message> = if headers.is_empty() {
        container(text("No tracks").size(12).color(Color::from_rgb(0.28,0.4,0.52)))
            .width(Length::Fixed(200.0)).height(Length::Fill).into()
    } else {
        scrollable(column(headers).spacing(1)).height(Length::Fill).into()
    };

    let time_sig = transport.time_sig_num;
    let playhead = transport.playhead_beat;
    let bpm = transport.bpm;

    let timeline: Element<Message> = canvas::Canvas::new(TimelineCanvas { tracks, playhead_beat: playhead, bpm, time_sig })
        .width(Length::Fill).height(Length::Fill).into();

    let main = row![header_col, timeline].height(Length::Fill);
    column![toolbar, main].height(Length::Fill).into()
}

struct TimelineCanvas {
    tracks: Vec<crate::app_state::Track>,
    playhead_beat: f64,
    bpm: f32,
    time_sig: u8,
}

impl<Message> canvas::Program<Message> for TimelineCanvas {
    type State = ();
    fn draw(&self, _: &(), renderer: &iced::Renderer, _: &Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let beat_w = 32.0f32;
        let ruler_h = 28.0f32;
        let total_beats = 128usize;

        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), Color::from_rgb(0.04,0.08,0.13));
        frame.fill_rectangle(iced::Point::ORIGIN, iced::Size::new(bounds.width, ruler_h), Color::from_rgb(0.03,0.06,0.1));

        for i in 0..total_beats {
            let is_bar = i % self.time_sig as usize == 0;
            let x = i as f32 * beat_w;
            let lh = if is_bar { ruler_h } else { ruler_h * 0.45 };
            frame.fill_rectangle(iced::Point::new(x, ruler_h - lh), iced::Size::new(1.0, lh),
                if is_bar { Color::from_rgb(0.12,0.23,0.37) } else { Color::from_rgb(0.04,0.08,0.13) });
            if is_bar {
                frame.fill_text(canvas::Text {
                    content: (i / self.time_sig as usize + 1).to_string(),
                    position: iced::Point::new(x + 3.0, 4.0),
                    color: Color::from_rgb(0.3,0.72,1.0),
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });
            }
        }

        let mut ty = ruler_h;
        for track in &self.tracks {
            let th = track.height;
            let tc = color_from_u32(track.color);
            frame.fill_rectangle(iced::Point::new(0.0, ty), iced::Size::new(bounds.width, th), Color::from_rgb(0.04,0.08,0.13));
            for i in 0..total_beats {
                let is_bar = i % self.time_sig as usize == 0;
                frame.fill_rectangle(iced::Point::new(i as f32 * beat_w, ty), iced::Size::new(1.0, th),
                    if is_bar { Color::from_rgb(0.06,0.12,0.18) } else { Color::from_rgb(0.03,0.06,0.1) });
            }
            for clip in &track.clips {
                let cx = clip.start_beat as f32 * beat_w + 1.0;
                let cw = (clip.length_beats as f32 * beat_w - 2.0).max(8.0);
                frame.fill_rectangle(iced::Point::new(cx, ty+4.0), iced::Size::new(cw, th-8.0), Color { a: 0.15, ..tc });
                let stroke = canvas::Stroke::default().with_color(Color { a: 0.5, ..tc }).with_width(1.0);
                frame.stroke(&canvas::Path::rectangle(iced::Point::new(cx, ty+4.0), iced::Size::new(cw, th-8.0)), stroke);
            }
            frame.fill_rectangle(iced::Point::new(0.0, ty+th-1.0), iced::Size::new(bounds.width, 1.0), Color::from_rgb(0.06,0.12,0.18));
            ty += th;
        }

        let phx = self.playhead_beat as f32 * beat_w;
        frame.fill_rectangle(iced::Point::new(phx, 0.0), iced::Size::new(1.0, bounds.height), Color::from_rgb(0.3,0.72,1.0));
        let tri = canvas::Path::new(|p| { p.move_to(iced::Point::new(phx-5.0,0.0)); p.line_to(iced::Point::new(phx+5.0,0.0)); p.line_to(iced::Point::new(phx,10.0)); p.close(); });
        frame.fill(&tri, Color::from_rgb(0.3,0.72,1.0));

        vec![frame.into_geometry()]
    }
}

// ── Mixer view ────────────────────────────────────────────────────────────────

fn mixer_view_el(channels: Vec<crate::app_state::MixerChannel>) -> Element<'static, Message> {
    let strips: Vec<Element<Message>> = channels.iter().map(|ch| {
        let c = color_from_u32(ch.color);
        let name = ch.name.clone();
        let vol = (ch.volume * 100.0) as u32;
        container(
            column![
                text(name).size(9).color(c),
                container(iced::widget::vertical_space()).width(Length::Fixed(48.0)).height(Length::Fixed(60.0))
                    .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.04,0.08,0.13))), border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 3.0.into() }, ..Default::default() }),
                container(iced::widget::vertical_space()).width(Length::Fixed(20.0)).height(Length::Fixed(100.0))
                    .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.04,0.08,0.13))), border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 3.0.into() }, ..Default::default() }),
                text(format!("{}%", vol)).size(8).color(Color::from_rgb(0.28,0.4,0.52)),
            ].spacing(4).align_x(Alignment::Center).padding(6),
        )
        .width(Length::Fixed(64.0))
        .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.03,0.06,0.1))), border: iced::Border { color: Color::from_rgb(0.06,0.12,0.18), width: 1.0, radius: 6.0.into() }, ..Default::default() })
        .into()
    }).collect();

    container(scrollable(row(strips).spacing(4).padding(12)).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())))
        .width(Length::Fill).height(Length::Fill)
        .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.02,0.05,0.08))), ..Default::default() })
        .into()
}

fn placeholder_view(msg: &str) -> Element<'static, Message> {
    let msg = msg.to_string();
    container(text(msg).size(14).color(Color::from_rgb(0.28,0.4,0.52)))
        .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
        .style(|_| container::Style { background: Some(iced::Background::Color(Color::from_rgb(0.02,0.05,0.08))), ..Default::default() })
        .into()
}

fn tab_style(active: bool) -> iced::widget::button::Style {
    if active {
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.3,0.72,1.0,0.1))),
            border: iced::Border { color: Color::from_rgba(0.3,0.72,1.0,0.25), width: 1.0, radius: 5.0.into() },
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
    iced::widget::button::Style { background: Some(iced::Background::Color(bg)), border: iced::Border { color: border, width: 1.0, radius: 5.0.into() }, text_color: tc, ..Default::default() }
}

fn color_from_u32(c: u32) -> Color {
    Color::from_rgba(((c>>24)&0xff) as f32/255.0, ((c>>16)&0xff) as f32/255.0, ((c>>8)&0xff) as f32/255.0, (c&0xff) as f32/255.0)
}
