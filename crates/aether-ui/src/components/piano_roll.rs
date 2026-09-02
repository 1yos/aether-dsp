//! Piano roll — draw MIDI notes on a pitch × time grid.

use iced::widget::{button, canvas, column, container, row, text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme as IcedTheme};

use crate::project::{MidiNote, TuningSystem};
use crate::theme::Theme;

const NOTE_HEIGHT: f32 = 16.0;
const BEAT_WIDTH: f32 = 60.0;
const KEY_WIDTH: f32 = 48.0;
const NUM_NOTES: usize = 60; // 5 octaves
const LOWEST_NOTE: u8 = 36; // C2

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    AddNote { pitch: u8, start_beat: f32, length_beats: f32 },
    RemoveNote(usize),
}

pub struct PianoRoll {
    pub notes: Vec<MidiNote>,
    pub tuning: TuningSystem,
    pub title: String,
}

impl Default for PianoRoll {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            tuning: TuningSystem::EqualTemperament,
            title: "Clip".to_string(),
        }
    }
}

impl PianoRoll {
    pub fn view(self) -> Element<'static, Message> {
        let title_text = self.title.clone();
        let title_bar = container(
            row![
                text(title_text).size(14).style(|_| text::Style {
                    color: Some(Theme::TEXT_PRIMARY),
                }),
                iced::widget::Space::with_width(Length::Fill),
                button(text("✕ Close").size(13).style(|_| text::Style {
                    color: Some(Theme::TEXT_SECONDARY),
                }))
                .on_press(Message::Close)
                .padding([4, 12])
                .style(|_, _| button::Style {
                    background: Some(iced::Background::Color(Theme::SURFACE)),
                    border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
                    ..Default::default()
                }),
            ]
            .align_y(iced::Alignment::Center)
            .padding([0, 16]),
        )
        .height(40)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
            ..Default::default()
        });

        let roll = PianoRollCanvas { notes: self.notes };

        let canvas_area = canvas(roll).width(Length::Fill).height(Length::Fill);

        let help = text(
            "Left-click: draw note  ·  Right-click: delete note",
        )
        .size(11)
        .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

        let footer = container(help).padding([6, 16]).style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            ..Default::default()
        });

        column![title_bar, canvas_area, footer].into()
    }
}

/// Canvas program that owns its note data.
struct PianoRollCanvas {
    notes: Vec<MidiNote>,
}

#[derive(Default)]
pub struct DrawState {
    pending: Option<(u8, f32)>,
}

impl canvas::Program<Message> for PianoRollCanvas {
    type State = DrawState;

    fn draw(
        &self,
        _state: &DrawState,
        renderer: &Renderer,
        _theme: &IcedTheme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), canvas::Fill::from(Theme::APP_BG));

        for row in 0..NUM_NOTES {
            let note_num = LOWEST_NOTE as usize + (NUM_NOTES - 1 - row);
            let y = row as f32 * NOTE_HEIGHT;
            let pitch_class = note_num % 12;
            let is_black = matches!(pitch_class, 1 | 3 | 6 | 8 | 10);

            // Piano key
            let key_col = if is_black {
                Color { r: 0.08, g: 0.08, b: 0.08, a: 1.0 }
            } else {
                Color { r: 0.18, g: 0.18, b: 0.18, a: 1.0 }
            };
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(KEY_WIDTH, NOTE_HEIGHT),
                canvas::Fill::from(key_col),
            );

            // C note labels
            if pitch_class == 0 {
                let octave = note_num / 12 - 1;
                frame.fill_text(canvas::Text {
                    content: format!("C{}", octave),
                    position: Point::new(4.0, y + 3.0),
                    color: Theme::TEXT_SECONDARY,
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });
            }

            // Roll background
            let roll_bg = if is_black {
                Color { r: 0.06, g: 0.06, b: 0.06, a: 1.0 }
            } else {
                Theme::APP_BG
            };
            frame.fill_rectangle(
                Point::new(KEY_WIDTH, y),
                Size::new(bounds.width - KEY_WIDTH, NOTE_HEIGHT),
                canvas::Fill::from(roll_bg),
            );

            // Horizontal line
            let mut hline = canvas::path::Builder::new();
            hline.move_to(Point::new(KEY_WIDTH, y + NOTE_HEIGHT));
            hline.line_to(Point::new(bounds.width, y + NOTE_HEIGHT));
            frame.stroke(
                &hline.build(),
                canvas::Stroke::default()
                    .with_color(Color { a: 0.3, ..Theme::BORDER })
                    .with_width(0.5),
            );
        }

        // Beat grid
        let num_beats = ((bounds.width - KEY_WIDTH) / BEAT_WIDTH) as usize + 2;
        for beat in 0..num_beats {
            let x = KEY_WIDTH + beat as f32 * BEAT_WIDTH;
            let is_bar = beat % 4 == 0;
            let mut vline = canvas::path::Builder::new();
            vline.move_to(Point::new(x, 0.0));
            vline.line_to(Point::new(x, bounds.height));
            frame.stroke(
                &vline.build(),
                canvas::Stroke::default()
                    .with_color(if is_bar {
                        Theme::BORDER
                    } else {
                        Color { a: 0.25, ..Theme::BORDER }
                    })
                    .with_width(if is_bar { 1.0 } else { 0.5 }),
            );
        }

        // Notes
        for note in &self.notes {
            let row =
                (LOWEST_NOTE as i32 + NUM_NOTES as i32 - 1 - note.pitch as i32) as f32;
            if row < 0.0 || row >= NUM_NOTES as f32 {
                continue;
            }
            let y = row * NOTE_HEIGHT;
            let x = KEY_WIDTH + note.start_beat * BEAT_WIDTH;
            let w = (note.length_beats * BEAT_WIDTH - 2.0).max(4.0);

            frame.fill_rectangle(
                Point::new(x, y + 1.5),
                Size::new(w, NOTE_HEIGHT - 3.0),
                canvas::Fill::from(Theme::ACCENT),
            );
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut DrawState,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if pos.x > KEY_WIDTH {
                        let row = (pos.y / NOTE_HEIGHT) as usize;
                        if row < NUM_NOTES {
                            let pitch = LOWEST_NOTE + (NUM_NOTES - 1 - row) as u8;
                            let beat = ((pos.x - KEY_WIDTH) / BEAT_WIDTH).floor();
                            state.pending = Some((pitch, beat));
                            return (canvas::event::Status::Captured, None);
                        }
                    }
                }
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )) => {
                if let Some((pitch, start)) = state.pending.take() {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::AddNote {
                            pitch,
                            start_beat: start,
                            length_beats: 1.0,
                        }),
                    );
                }
            }
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Right,
            )) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if pos.x > KEY_WIDTH {
                        let row = (pos.y / NOTE_HEIGHT) as usize;
                        if row < NUM_NOTES {
                            let pitch = LOWEST_NOTE + (NUM_NOTES - 1 - row) as u8;
                            let beat = (pos.x - KEY_WIDTH) / BEAT_WIDTH;
                            for (i, note) in self.notes.iter().enumerate() {
                                if note.pitch == pitch
                                    && beat >= note.start_beat
                                    && beat <= note.start_beat + note.length_beats
                                {
                                    return (
                                        canvas::event::Status::Captured,
                                        Some(Message::RemoveNote(i)),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }
}
